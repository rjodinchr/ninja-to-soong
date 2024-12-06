// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

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
    globals: Option<HashMap<String, String>>,
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
            globals: None,
        }
    }

    pub fn set_globals(&mut self, globals: HashMap<String, String>) {
        self.globals = Some(globals);
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

    pub fn get_link_flags(&self) -> (Option<PathBuf>, HashSet<String>) {
        let mut link_flags = HashSet::new();
        let mut version_script = None;
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = Some(PathBuf::from(vs));
                }
                link_flags.insert(flag.to_string());
            }
        }
        (version_script, link_flags)
    }

    pub fn get_link_libraries(&self) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String> {
        let mut static_libraries = HashSet::new();
        let mut shared_libraries = HashSet::new();
        let Some(libs) = self.variables.get("LINK_LIBRARIES") else {
            return Ok((static_libraries, shared_libraries));
        };
        for lib in libs.split(" ") {
            if lib.starts_with("-") || lib.is_empty() {
                continue;
            } else {
                let lib_path = PathBuf::from(lib);
                if lib.ends_with(".a") {
                    static_libraries.insert(lib_path);
                } else if lib.ends_with(".so") {
                    shared_libraries.insert(lib_path);
                } else {
                    return error!("unsupported library '{lib}' from target: {self:#?}");
                }
            }
        }
        Ok((static_libraries, shared_libraries))
    }

    pub fn get_defines(&self) -> HashSet<String> {
        let mut defines = HashSet::new();
        if let Some(defs) = self.variables.get("DEFINES") {
            for define in defs.split("-D") {
                if define.is_empty() {
                    continue;
                }
                defines.insert(define.trim().to_string());
            }
        };
        defines
    }

    pub fn get_includes(&self) -> HashSet<PathBuf> {
        let mut includes = HashSet::new();
        let Some(incs) = self.variables.get("INCLUDES") else {
            return includes;
        };
        for inc in incs.split(" ") {
            let include = inc.strip_prefix("-I").unwrap_or(inc);
            if include.is_empty() || include == "isystem" {
                continue;
            }
            includes.insert(PathBuf::from(include));
        }
        includes
    }

    pub fn get_gen_headers(
        &self,
        targets_map: &NinjaTargetsMap,
    ) -> Result<HashSet<PathBuf>, String> {
        Ok(targets_map.traverse_from(
            vec![self.outputs[0].clone()],
            HashSet::new(),
            |gen_headers, rule, name, target| {
                if rule != "CUSTOM_COMMAND" || target.get_cmd()?.is_none() {
                    return Ok(());
                }
                gen_headers.insert(name);
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
