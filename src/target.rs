use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;

pub const COPY_TARGET: &str = "cmake_copy_if_different";

pub fn rework_target_name(target_name: &str, prefix: &str) -> String {
    let mut name = prefix.to_string() + target_name;
    name = name.strip_suffix(".so").unwrap_or(&name).to_string();
    name = name.strip_suffix(".a").unwrap_or(&name).to_string();
    return name.replace("/", "__").replace(".", "__");
}

pub fn rework_source_path(source: &str, source_root: &str) -> String {
    return source.replace(source_root, "");
}

pub fn create_map(targets: &Vec<BuildTarget>) -> HashMap<String, &BuildTarget> {
    let mut map: HashMap<String, &BuildTarget> = HashMap::new();
    for target in targets {
        for output in &target.outputs {
            map.insert(output.clone(), target);
        }
        for output in &target.implicit_outputs {
            map.insert(output.clone(), target);
        }
    }

    return map;
}

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
    pub fn get_outputs(&self) -> &Vec<String> {
        &self.outputs
    }
    pub fn get_name(&self) -> &String {
        return &self.outputs[0];
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
    pub fn get_link_flags(&self, source_root: &str) -> (String, HashSet<String>) {
        let mut link_flags: HashSet<String> = HashSet::new();
        let mut version_script = String::from("");
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if flag.contains("-Bsymbolic") {
                    link_flags.insert(flag.replace(source_root, ""));
                } else if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = rework_source_path(vs, source_root);
                }
            }
        }
        return (version_script, link_flags);
    }
    pub fn get_link_libraries(
        &self,
        native_lib_root: &str,
        prefix: &str,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<String>), String> {
        let mut static_libraries: HashSet<String> = HashSet::new();
        let mut shared_libraries: HashSet<String> = HashSet::new();
        let mut system_shared_libraries: HashSet<String> = HashSet::new();
        if let Some(libs) = self.variables.get("LINK_LIBRARIES") {
            for lib in libs.split(" ") {
                if lib.strip_prefix("-Wl").is_some()
                    || lib == "-lrt"
                    || lib == "-pthread"
                    || lib == "-latomic"
                    || lib == "-ldl"
                    || lib == "-lm"
                {
                    continue;
                } else if let Some(stripped_lib) = lib.strip_prefix("-l") {
                    system_shared_libraries.insert("lib".to_string() + stripped_lib);
                } else if lib.starts_with(native_lib_root) {
                    let new_lib = lib.split("/").last().unwrap();
                    if let Some(new_lib_stripped) = new_lib.strip_suffix(".a") {
                        static_libraries.insert(String::from(new_lib_stripped));
                    } else if let Some(new_lib_stripped) = new_lib.strip_suffix(".so") {
                        shared_libraries.insert(String::from(new_lib_stripped));
                    } else {
                        return error!(format!(
                            "unsupported library '{lib}' from target: {self:#?}"
                        ));
                    }
                } else {
                    let lib_name = rework_target_name(lib, prefix);
                    if lib.ends_with(".a") {
                        static_libraries.insert(lib_name);
                    } else if lib.ends_with(".so") {
                        shared_libraries.insert(lib_name);
                    } else if lib == "" {
                        continue;
                    } else {
                        return error!(format!(
                            "unsupported library '{lib}' from target: {self:#?}"
                        ));
                    }
                }
            }
        }
        return Ok((static_libraries, shared_libraries, system_shared_libraries));
    }
    fn get_defines(&self) -> HashSet<String> {
        let mut defines: HashSet<String> = HashSet::new();

        if let Some(defs) = self.variables.get("DEFINES") {
            for def in defs.trim().split("-D") {
                defines.insert(def.replace(" ", ""));
            }
        };
        return defines;
    }
    fn get_includes(&self, source_root: &str, build_root: &str) -> HashSet<String> {
        let mut includes: HashSet<String> = HashSet::new();
        if let Some(incs) = self.variables.get("INCLUDES") {
            for inc in incs.split(" ") {
                if inc.contains(build_root) {
                    continue;
                }
                if let Some(stripped_inc) = inc.strip_prefix("-I") {
                    includes.insert(rework_source_path(stripped_inc, source_root));
                }
            }
        }
        return includes;
    }
    pub fn get_command(&self) -> Result<Option<String>, String> {
        if self.rule != "CUSTOM_COMMAND" {
            return error!(format!(
                "Can only look for command in CUSTOM_COMMAND: {self:#?}"
            ));
        }
        let Some(command) = self.variables.get("COMMAND") else {
            return error!(format!("No command in CUSTOM_COMMAND: {self:#?}"));
        };
        let mut split = command.split(" && ");
        let split_count = split.clone().count();
        if split_count < 2 {
            return error!(format!(
                "Could not find enough split in command (expected at least 2, got {split_count}"
            ));
        }
        let command = split.nth(1).unwrap();
        return Ok(if command.contains("cmake -E copy_if_different") {
            Some(COPY_TARGET.to_string())
        } else if command.contains("/usr/bin/cmake") {
            None
        } else {
            Some(command.to_string())
        });
    }
    pub fn get_generated_headers(
        &self,
        targets_map: &HashMap<String, &BuildTarget>,
        prefix: &str,
    ) -> Result<HashSet<String>, String> {
        let mut generated_headers: HashSet<String> = HashSet::new();
        let mut target_seen: HashSet<String> = HashSet::new();
        let mut target_to_parse = vec![self.get_name().clone()];

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
                            generated_headers
                                .insert(rework_target_name(&target.get_name(), prefix));
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
        source_root: &str,
        build_root: &str,
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
        let mut defines = self.get_defines();
        for def in defines_no_assembly {
            defines.insert(def);
        }
        let includes = self.get_includes(source_root, build_root);
        let src = rework_source_path(&self.inputs[0], source_root);
        return Ok((src, includes, defines));
    }
}
