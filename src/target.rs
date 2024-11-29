use std::collections::HashMap;
use std::collections::HashSet;

use crate::project::Project;
use crate::utils::*;

#[derive(Debug)]
pub struct BuildTarget {
    rule: String,
    outputs: Vec<String>,
    implicit_outputs: Vec<String>,
    inputs: Vec<String>,
    implicit_dependencies: Vec<String>,
    order_only_dependencies: Vec<String>,
    variables: HashMap<String, String>,
}

impl BuildTarget {
    pub fn new(
        rule: String,
        outputs: Vec<String>,
        implicit_outputs: Vec<String>,
        inputs: Vec<String>,
        implicit_dependencies: Vec<String>,
        order_only_dependencies: Vec<String>,
        variables: HashMap<String, String>,
    ) -> Self {
        BuildTarget {
            rule,
            outputs,
            implicit_outputs,
            inputs,
            implicit_dependencies,
            order_only_dependencies,
            variables,
        }
    }

    pub fn get_inputs(&self) -> &Vec<String> {
        &self.inputs
    }

    pub fn get_rule(&self) -> &String {
        &self.rule
    }

    pub fn get_name(&self, prefix: &str) -> String {
        return (prefix.to_string() + &self.outputs[0])
            .replace("/", "_")
            .replace(".", "_");
    }

    pub fn get_outputs(&self) -> &Vec<String> {
        return &self.outputs;
    }

    pub fn get_all_inputs(&self) -> Vec<String> {
        let mut inputs: Vec<String> = Vec::new();
        for input in &self.inputs {
            inputs.push(input.clone());
        }
        for input in &self.implicit_dependencies {
            inputs.push(input.clone());
        }
        for input in &self.order_only_dependencies {
            inputs.push(input.clone());
        }
        return inputs;
    }

    pub fn get_all_outputs(&self) -> Vec<String> {
        let mut outputs: Vec<String> = Vec::new();
        for output in &self.outputs {
            outputs.push(output.clone());
        }
        for output in &self.implicit_outputs {
            outputs.push(output.clone());
        }
        return outputs;
    }

    pub fn get_link_flags(
        &self,
        src_root: &str,
        project: &dyn Project,
    ) -> (String, HashSet<String>) {
        let mut link_flags: HashSet<String> = HashSet::new();
        let mut version_script = String::new();
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = vs.replace(&add_slash_suffix(src_root), "");
                } else {
                    project.handle_link_flag(flag, &mut link_flags);
                }
            }
        }
        return (version_script, link_flags);
    }

    pub fn get_link_libraries(
        &self,
        ndk_root: &str,
        project: &dyn Project,
    ) -> Result<(HashSet<String>, HashSet<String>), String> {
        let mut static_libraries: HashSet<String> = HashSet::new();
        let mut shared_libraries: HashSet<String> = HashSet::new();
        let Some(libs) = self.variables.get("LINK_LIBRARIES") else {
            return Ok((static_libraries, shared_libraries));
        };
        for lib in libs.split(" ") {
            if lib.starts_with("-") || lib == "" {
                continue;
            } else {
                let lib_name = if lib.starts_with(ndk_root) {
                    lib.split("/")
                        .last()
                        .unwrap()
                        .replace(".a", "")
                        .replace(".so", "")
                } else {
                    project.get_library_name(lib)
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
        return Ok((static_libraries, shared_libraries));
    }

    fn get_defines(&self, project: &dyn Project) -> HashSet<String> {
        let mut defines: HashSet<String> = HashSet::new();

        if let Some(defs) = self.variables.get("DEFINES") {
            for def in defs.split("-D") {
                let trim_def = def.trim();
                if project.ignore_define(trim_def) {
                    continue;
                }
                defines.insert(trim_def.to_string());
            }
        };
        defines.remove("");
        return defines;
    }

    fn get_includes(&self, src_root: &str, project: &dyn Project) -> HashSet<String> {
        let mut includes: HashSet<String> = HashSet::new();
        let Some(incs) = self.variables.get("INCLUDES") else {
            return includes;
        };
        for inc in incs.split(" ") {
            if project.ignore_include(inc) {
                continue;
            }
            let inc = project
                .rework_include(inc)
                .replace(&add_slash_suffix(src_root), "")
                .replace(src_root, "");

            if let Some(stripped_inc) = inc.strip_prefix("-I") {
                includes.insert(stripped_inc.to_string());
            } else if inc == "-isystem" {
                continue;
            } else {
                includes.insert(inc);
            }
        }
        return includes;
    }

    pub fn get_generated_headers(
        &self,
        targets_map: &HashMap<String, &BuildTarget>,
    ) -> Result<HashSet<String>, String> {
        let mut generated_headers: HashSet<String> = HashSet::new();
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
            for output in target.get_all_outputs() {
                target_seen.insert(output);
            }

            if target.rule == "CUSTOM_COMMAND" {
                match target.get_command() {
                    Ok(option) => match option {
                        Some(_) => {
                            generated_headers.insert(target_name);
                        }
                        None => continue,
                    },
                    Err(err) => return Err(err),
                }
            }
        }

        return Ok(generated_headers);
    }

    pub fn get_compiler_target_info(
        &self,
        src_root: &str,
        project: &dyn Project,
    ) -> Result<(String, HashSet<String>, HashSet<String>), String> {
        let mut defines_no_assembly: HashSet<String> = HashSet::new();
        if self.rule.starts_with("ASM_COMPILER") {
            defines_no_assembly.insert("BLAKE3_NO_AVX512".to_string());
            defines_no_assembly.insert("BLAKE3_NO_AVX2".to_string());
            defines_no_assembly.insert("BLAKE3_NO_SSE41".to_string());
            defines_no_assembly.insert("BLAKE3_NO_SSE2".to_string());
        } else if !self.rule.starts_with("CXX_COMPILER") && !self.rule.starts_with("C_COMPILER") {
            return error!(format!("unsupported input target for library: {self:#?}"));
        }
        if self.inputs.len() != 1 {
            return error!(format!(
                "Too many inputs in CXX_COMPILER input target for library: {self:#?}"
            ));
        }
        let mut defines = self.get_defines(project);
        for def in defines_no_assembly {
            defines.insert(def);
        }
        let includes = self.get_includes(src_root, project);
        let src = self.inputs[0].replace(&add_slash_suffix(src_root), "");
        return Ok((src, includes, defines));
    }

    pub fn get_command(&self) -> Result<Option<String>, String> {
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
        return Ok(if command.contains("/usr/bin/cmake") {
            None
        } else {
            Some(command.to_string())
        });
    }
}
