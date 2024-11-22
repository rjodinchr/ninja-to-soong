use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;

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
    fn rework_target_name(target_name: String) -> String {
        let mut name = target_name;
        name = name.strip_suffix(".so").unwrap_or(&name).to_string();
        name = name.strip_suffix(".a").unwrap_or(&name).to_string();
        return name.replace("/", "_").replace(".", "_");
    }
    pub fn get_name(&self) -> String {
        return Self::rework_target_name(self.outputs[0].to_string());
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
    pub fn get_link_flags(&self, src_root: &str) -> (Option<String>, HashSet<String>) {
        let mut link_flags: HashSet<String> = HashSet::new();
        let mut version_script = None;
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if flag.contains("-Bsymbolic") {
                    link_flags.insert(flag.replace(src_root, ""));
                } else if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = Some(vs.replace(src_root, ""));
                }
            }
        }
        return (version_script, link_flags);
    }
    pub fn get_link_libraries(
        &self,
        ndk_root: &str,
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
                    Self::rework_target_name(lib.to_string())
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
    fn get_defines(&self) -> HashSet<String> {
        let mut defines: HashSet<String> = HashSet::new();

        if let Some(defs) = self.variables.get("DEFINES") {
            for def in defs.trim().split("-D") {
                defines.insert(def.replace(" ", ""));
            }
        };
        defines.remove("");
        return defines;
    }
    fn get_includes(
        &self,
        src_root: &str,
        build_root: &str,
        dst_build_prefix: &str,
    ) -> HashSet<String> {
        let mut includes: HashSet<String> = HashSet::new();
        let Some(incs) = self.variables.get("INCLUDES") else {
            return includes;
        };
        for inc in incs.split(" ") {
            let inc = inc
                .replace(build_root, dst_build_prefix)
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

            for output in target.get_all_outputs() {
                target_seen.insert(output);
            }

            if target.rule == "CUSTOM_COMMAND" {
                for output in target.get_all_outputs() {
                    if !(output.ends_with(".inc")
                        || output.ends_with(".h")
                        || output.ends_with(".hpp")
                        || output.ends_with(".def"))
                    {
                        continue;
                    }
                    generated_headers.insert(output.replace("${cmake_ninja_workdir}", ""));
                }
            } else {
                target_to_parse.append(&mut target.get_all_inputs());
            }
        }

        return Ok(generated_headers);
    }
    pub fn get_compiler_target_info(
        &self,
        src_root: &str,
        build_root: &str,
        dst_build_prefix: &str,
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
        let includes = self.get_includes(src_root, build_root, dst_build_prefix);
        let src = self.inputs[0].replace(src_root, "");
        return Ok((src, includes, defines));
    }
}
