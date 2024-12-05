// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::project::Project;
use crate::utils::*;

#[derive(Debug)]
pub struct NinjaTarget {
    rule: String,
    outputs: Vec<PathBuf>,
    implicit_outputs: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    implicit_deps: Vec<PathBuf>,
    order_only_deps: Vec<PathBuf>,
    variables: HashMap<String, String>,
}

impl NinjaTarget {
    pub fn new(
        rule: String,
        outputs: Vec<PathBuf>,
        implicit_outputs: Vec<PathBuf>,
        inputs: Vec<PathBuf>,
        implicit_deps: Vec<PathBuf>,
        order_only_deps: Vec<PathBuf>,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            rule,
            outputs,
            implicit_outputs,
            inputs,
            implicit_deps,
            order_only_deps,
            variables,
        }
    }

    pub fn get_inputs(&self) -> &Vec<PathBuf> {
        &self.inputs
    }

    pub fn get_outputs(&self) -> &Vec<PathBuf> {
        &self.outputs
    }

    pub fn get_name(&self, prefix: &Path) -> String {
        path_to_id(prefix.join(&self.outputs[0]))
    }

    pub fn get_link_flags(
        &self,
        src_path: &Path,
        project: &dyn Project,
    ) -> (Option<PathBuf>, HashSet<String>) {
        let mut link_flags = HashSet::new();
        let mut version_script = None;
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = Some(strip_prefix(vs, src_path));
                } else if !project.ignore_link_flag(flag) {
                    link_flags.insert(flag.to_string());
                }
            }
        }
        (version_script, link_flags)
    }

    pub fn get_link_libraries(
        &self,
        ndk_path: &Path,
        project: &dyn Project,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<PathBuf>), String> {
        let mut static_libraries = HashSet::new();
        let mut shared_libraries = HashSet::new();
        let mut generated_libraries = HashSet::new();
        let Some(libs) = self.variables.get("LINK_LIBRARIES") else {
            return Ok((static_libraries, shared_libraries, generated_libraries));
        };
        for lib in libs.split(" ") {
            if lib.starts_with("-") || lib == "" {
                continue;
            } else {
                let lib_path = PathBuf::from(lib);
                let lib_name = if lib_path.starts_with(ndk_path) {
                    lib_path.file_stem().unwrap().to_str().unwrap().to_string()
                } else {
                    generated_libraries.insert(lib_path.clone());
                    path_to_id(project.get_library_name(&lib_path))
                };
                if lib.ends_with(".a") {
                    static_libraries.insert(lib_name);
                } else if lib.ends_with(".so") {
                    shared_libraries.insert(lib_name);
                } else {
                    return error!("unsupported library '{lib}' from target: {self:#?}");
                }
            }
        }
        Ok((static_libraries, shared_libraries, generated_libraries))
    }

    pub fn get_defines(&self, project: &dyn Project) -> HashSet<String> {
        let mut defines = HashSet::new();
        if let Some(defs) = self.variables.get("DEFINES") {
            for define in defs.split("-D") {
                if define.is_empty() || project.ignore_define(define) {
                    continue;
                }
                defines.insert("-D".to_string() + define.trim());
            }
        };
        defines
    }

    pub fn get_includes(&self, src_path: &Path, project: &dyn Project) -> HashSet<PathBuf> {
        let mut includes = HashSet::new();
        let Some(incs) = self.variables.get("INCLUDES") else {
            return includes;
        };
        for include in incs.split(" ") {
            let include_path = PathBuf::from(include.strip_prefix("-I").unwrap_or(include));
            if project.ignore_include(&include_path) {
                continue;
            }
            let inc = strip_prefix(project.get_include(&include_path), src_path);
            let include_string = path_to_string(&inc);
            if include_string.is_empty() || include_string == "isystem" {
                continue;
            }
            includes.insert(inc);
        }
        includes
    }

    pub fn get_gen_headers_and_gen_deps(
        &self,
        target_prefix: &Path,
        targets_map: &NinjaTargetsMap,
        project: &dyn Project,
    ) -> Result<(HashSet<String>, HashSet<PathBuf>), String> {
        Ok(targets_map.traverse_from(
            vec![self.outputs[0].clone()],
            (HashSet::new(), HashSet::new()),
            |(gen_headers, gen_deps), rule, name, target| {
                if rule != "CUSTOM_COMMAND" || target.get_cmd()?.is_none() {
                    return Ok(());
                }
                if project.ignore_gen_header(Path::new(&name)) {
                    gen_deps.insert(name);
                } else {
                    gen_headers.insert(match targets_map.get(&name) {
                        Some(target_header) => target_header.get_name(target_prefix),
                        None => return error!("Could not find target for {name:#?}"),
                    });
                }
                Ok(())
            },
            |_target_name| false,
        )?)
    }

    pub fn get_cmd(&self) -> Result<Option<String>, String> {
        let Some(command) = self.variables.get("COMMAND") else {
            return error!("No command in: {self:#?}");
        };
        let mut split = command.split(" && ");
        let split_count = split.clone().count();
        if split_count < 2 {
            return error!(
                "Could not find enough split in command (expected at least 2, got {split_count}"
            );
        }
        let command = split.nth(1).unwrap();
        Ok(if command.contains("bin/cmake ") {
            None
        } else {
            Some(command.to_string())
        })
    }
}

pub struct NinjaTargetsMap<'a>(HashMap<PathBuf, &'a NinjaTarget>);

impl<'a> NinjaTargetsMap<'a> {
    pub fn new(targets: &'a Vec<NinjaTarget>) -> Self {
        let mut map = HashMap::new();
        for target in targets {
            for output in &target.outputs {
                map.insert(output.clone(), target);
            }
            for output in &target.implicit_outputs {
                map.insert(output.clone(), target);
            }
        }
        Self(map)
    }
    pub fn get(&self, key: &Path) -> Option<&&NinjaTarget> {
        self.0.get(key)
    }
    pub fn traverse_from<I, F, G>(
        &self,
        mut targets: Vec<PathBuf>,
        mut iterator: I,
        mut f: F,
        ignore_target: G,
    ) -> Result<I, String>
    where
        F: FnMut(&mut I, &str, PathBuf, &NinjaTarget) -> Result<(), String>,
        G: Fn(&Path) -> bool,
    {
        let mut targets_seen = HashSet::new();
        while let Some(target_name) = targets.pop() {
            if targets_seen.contains(&target_name) || ignore_target(&target_name) {
                continue;
            }
            let Some(target) = self.get(&target_name) else {
                continue;
            };
            targets.append(&mut target.inputs.clone());
            targets.append(&mut target.implicit_deps.clone());
            targets.append(&mut target.order_only_deps.clone());
            targets_seen.extend(target.outputs.clone());
            targets_seen.extend(target.implicit_outputs.clone());
            f(&mut iterator, &target.rule, target_name, target)?;
        }
        Ok(iterator)
    }
}
