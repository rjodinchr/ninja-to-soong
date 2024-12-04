// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::project::Project;
use crate::utils::*;

#[derive(Debug)]
pub struct NinjaTarget {
    rule: String,
    outputs: Vec<String>,
    implicit_outputs: Vec<String>,
    inputs: Vec<String>,
    implicit_deps: Vec<String>,
    order_only_deps: Vec<String>,
    variables: HashMap<String, String>,
}

impl NinjaTarget {
    pub fn new(
        rule: String,
        outputs: Vec<String>,
        implicit_outputs: Vec<String>,
        inputs: Vec<String>,
        implicit_dependencies: Vec<String>,
        order_only_dependencies: Vec<String>,
        variables: HashMap<String, String>,
    ) -> Self {
        NinjaTarget {
            rule,
            outputs,
            implicit_outputs,
            inputs,
            implicit_deps: implicit_dependencies,
            order_only_deps: order_only_dependencies,
            variables,
        }
    }

    pub fn get_inputs(&self) -> &Vec<String> {
        &self.inputs
    }

    pub fn get_outputs(&self) -> &Vec<String> {
        &self.outputs
    }

    pub fn get_name(&self, prefix: &str) -> String {
        rework_name(prefix.to_string() + "_" + &self.outputs[0])
    }

    pub fn get_link_flags(
        &self,
        src_dir: &str,
        project: &dyn Project,
    ) -> (Option<String>, HashSet<String>) {
        let mut link_flags: HashSet<String> = HashSet::new();
        let mut version_script = None;
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = Some(vs.replace(&add_slash_suffix(src_dir), ""));
                } else if !project.ignore_link_flag(flag) {
                    link_flags.insert(flag.to_string());
                }
            }
        }
        (version_script, link_flags)
    }

    pub fn get_link_libraries(
        &self,
        ndk_dir: &str,
        project: &dyn Project,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<String>), String> {
        let mut static_libraries: HashSet<String> = HashSet::new();
        let mut shared_libraries: HashSet<String> = HashSet::new();
        let mut generated_libraries: HashSet<String> = HashSet::new();
        let Some(libs) = self.variables.get("LINK_LIBRARIES") else {
            return Ok((static_libraries, shared_libraries, generated_libraries));
        };
        for lib in libs.split(" ") {
            if lib.starts_with("-") || lib == "" {
                continue;
            } else {
                let lib_name = if lib.starts_with(ndk_dir) {
                    lib.split("/")
                        .last()
                        .unwrap()
                        .replace(".a", "")
                        .replace(".so", "")
                } else {
                    generated_libraries.insert(lib.to_string());
                    rework_name(project.get_library_name(lib))
                };
                if lib.ends_with(".a") {
                    static_libraries.insert(lib_name);
                } else if lib.ends_with(".so") {
                    shared_libraries.insert(lib_name);
                } else {
                    return error!(format!(
                        "unsupported library '{lib}' from target: {self:#?}"
                    ));
                }
            }
        }
        Ok((static_libraries, shared_libraries, generated_libraries))
    }

    pub fn get_defines(&self, project: &dyn Project) -> HashSet<String> {
        let mut defines: HashSet<String> = HashSet::new();

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

    pub fn get_includes(&self, src_dir: &str, project: &dyn Project) -> HashSet<String> {
        let mut includes: HashSet<String> = HashSet::new();
        let Some(incs) = self.variables.get("INCLUDES") else {
            return includes;
        };
        for include in incs.split(" ") {
            if project.ignore_include(include) {
                continue;
            }
            let inc = project
                .get_include(include.strip_prefix("-I").unwrap_or(include))
                .replace(&add_slash_suffix(src_dir), "")
                .replace(src_dir, "");

            if inc.is_empty() || inc == "isystem" {
                continue;
            }
            includes.insert(inc);
        }
        includes
    }

    pub fn get_gen_headers_and_gen_deps(
        &self,
        target_prefix: &str,
        targets_map: &NinjaTargetsMap,
        project: &dyn Project,
    ) -> Result<(HashSet<String>, HashSet<String>), String> {
        Ok(targets_map.traverse_from(
            vec![self.outputs[0].clone()],
            (HashSet::new(), HashSet::new()),
            |(gen_headers, gen_deps), rule, name, target| {
                if rule != "CUSTOM_COMMAND" || target.get_cmd()?.is_none() {
                    return Ok(());
                }
                if project.ignore_gen_header(&name) {
                    gen_deps.insert(name);
                } else {
                    gen_headers.insert(match targets_map.get(&name) {
                        Some(target_header) => target_header.get_name(target_prefix),
                        None => return error!(format!("Could not find target for '{name}'")),
                    });
                }
                Ok(())
            },
            |_target_name| false,
        )?)
    }

    pub fn get_cmd(&self) -> Result<Option<String>, String> {
        let Some(command) = self.variables.get("COMMAND") else {
            return error!(format!("No command in: {self:#?}"));
        };
        let mut split = command.split(" && ");
        let split_count = split.clone().count();
        if split_count < 2 {
            return error!(format!(
                "Could not find enough split in command (expected at least 2, got {split_count}"
            ));
        }
        let command = split.nth(1).unwrap();
        Ok(if command.contains("bin/cmake ") {
            None
        } else {
            Some(command.to_string())
        })
    }
}

pub struct NinjaTargetsMap<'a> {
    map: HashMap<String, &'a NinjaTarget>,
}

impl<'a> NinjaTargetsMap<'a> {
    pub fn new(targets: &'a Vec<NinjaTarget>) -> Self {
        let mut map: HashMap<String, &'a NinjaTarget> = HashMap::new();
        for target in targets {
            for output in &target.outputs {
                map.insert(output.clone(), target);
            }
            for output in &target.implicit_outputs {
                map.insert(output.clone(), target);
            }
        }
        NinjaTargetsMap { map }
    }
    pub fn get(&self, key: &str) -> Option<&&NinjaTarget> {
        self.map.get(key)
    }
    pub fn traverse_from<I, F, G>(
        &self,
        mut targets: Vec<String>,
        mut iterator: I,
        mut f: F,
        ignore_target: G,
    ) -> Result<I, String>
    where
        F: FnMut(&mut I, &str, String, &NinjaTarget) -> Result<(), String>,
        G: Fn(&String) -> bool,
    {
        let mut targets_seen: HashSet<String> = HashSet::new();
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
