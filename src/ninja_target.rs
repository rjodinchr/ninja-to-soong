// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::project::Project;
use crate::utils::*;

pub type NinjaTargetsMap<'a> = HashMap<String, &'a NinjaTarget>;

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

    pub fn get_rule(&self) -> &str {
        &self.rule
    }

    pub fn get_name(&self, prefix: &str) -> String {
        rework_name(prefix.to_string() + "_" + &self.outputs[0])
    }

    pub fn get_outputs(&self) -> &Vec<String> {
        &self.outputs
    }

    pub fn get_all_inputs(&self) -> Vec<String> {
        let mut inputs: Vec<String> = Vec::new();
        for input in &self.inputs {
            inputs.push(input.clone());
        }
        for input in &self.implicit_deps {
            inputs.push(input.clone());
        }
        for input in &self.order_only_deps {
            inputs.push(input.clone());
        }
        inputs
    }

    pub fn get_all_outputs(&self) -> Vec<String> {
        let mut outputs: Vec<String> = Vec::new();
        for output in &self.outputs {
            outputs.push(output.clone());
        }
        for output in &self.implicit_outputs {
            outputs.push(output.clone());
        }
        outputs
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
                } else {
                    project.update_link_flags(flag, &mut link_flags);
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
        let mut gen_headers: HashSet<String> = HashSet::new();
        let mut gen_deps: HashSet<String> = HashSet::new();
        let mut target_seen: HashSet<String> = HashSet::new();
        let mut target_to_parse = vec![self.outputs[0].clone()];

        while let Some(target_name) = target_to_parse.pop() {
            if target_seen.contains(&target_name) {
                continue;
            }
            let Some(target) = targets_map.get(&target_name) else {
                continue;
            };

            target_to_parse.append(&mut target.get_all_inputs());
            target_seen.extend(target.get_all_outputs());

            if target.rule != "CUSTOM_COMMAND" || target.get_cmd()?.is_none() {
                continue;
            }
            if project.ignore_gen_header(&target_name) {
                gen_deps.insert(target_name.clone());
            } else {
                gen_headers.insert(match targets_map.get(&target_name) {
                    Some(target_header) => target_header.get_name(target_prefix),
                    None => return error!(format!("Could not find target for '{target_name}'")),
                });
            }
        }
        Ok((gen_headers, gen_deps))
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
