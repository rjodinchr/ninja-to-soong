use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;
use crate::target::BuildTarget;

#[derive(Debug)]
struct SoongFile<'a> {
    content: String,
    sources: HashSet<String>,
    generated_headers: HashSet<String>,
    include_directories: HashSet<String>,
    targets_map: &'a HashMap<String, &'a BuildTarget>,
    src_root: &'a str,
    ndk_root: &'a str,
    build_root: &'a str,
    target_prefix: &'a str,
    host_targets: bool,
    input_ref_for_genrule: &'a str,
    dst_build_prefix: &'a str,
}

impl<'a> SoongFile<'a> {
    fn new(
        targets_map: &'a HashMap<String, &'a BuildTarget>,
        src_root: &'a str,
        ndk_root: &'a str,
        build_root: &'a str,
        target_prefix: &'a str,
        host_targets: bool,
        input_ref_for_genrule: &'a str,
        dst_build_prefix: &'a str,
    ) -> Self {
        SoongFile {
            content: String::new(),
            sources: HashSet::new(),
            generated_headers: HashSet::new(),
            include_directories: HashSet::new(),
            targets_map,
            src_root,
            ndk_root,
            build_root,
            target_prefix,
            host_targets,
            input_ref_for_genrule,
            dst_build_prefix,
        }
    }

    fn finish(self) -> (String, HashSet<String>, HashSet<String>, HashSet<String>) {
        (
            self.content,
            self.sources,
            self.generated_headers,
            self.include_directories,
        )
    }

    fn generate_object(&mut self, name: &str, target: &BuildTarget) -> Result<String, String> {
        let mut module = crate::soongmodule::SoongModule::new(name);
        let copy_for_device = name == "cc_binary_host";
        let optimize_for_size = name == "cc_library_static";
        let name = if copy_for_device {
            "HOST_".to_string() + &target.get_name(self.target_prefix)
        } else {
            target.get_name(self.target_prefix)
        };
        module.add_str("name", name.clone());
        if name == "libOpenCL_so" {
            module.add_str("stem", "libclvk".to_string());
        }
        if optimize_for_size {
            module.add_bool("optimize_for_size", true);
        }
        module.add_bool("use_clang_lld", true);

        let mut includes: HashSet<String> = HashSet::new();
        let mut defines: HashSet<String> = HashSet::new();
        let mut srcs: HashSet<String> = HashSet::new();
        for input in target.get_inputs() {
            let Some(target) = self.targets_map.get(input) else {
                return error!(format!("unsupported input for library: {input}"));
            };
            let (src, src_includes, src_defines) = match target.get_compiler_target_info(
                self.src_root,
                self.build_root,
                self.dst_build_prefix,
            ) {
                Ok(return_values) => return_values,
                Err(err) => return Err(err),
            };
            for inc in src_includes {
                includes.insert(inc.clone());
                self.include_directories.insert(inc);
            }
            for def in src_defines {
                defines.insert(String::from("-D") + &def);
            }
            self.sources.insert(src.clone());
            srcs.insert(src);
        }
        module.add_set("srcs", srcs);
        module.add_set("local_include_dirs", includes);
        module.add_set("cflags", defines);

        let (version_script, link_flags) = target.get_link_flags(self.src_root);
        module.add_set("ldflags", link_flags);
        if let Some(vs) = version_script {
            self.sources.insert(vs.clone());
            module.add_str("version_script", vs);
        }

        let (static_libs, shared_libs) =
            match target.get_link_libraries(self.ndk_root, self.target_prefix) {
                Ok(return_values) => return_values,
                Err(err) => return Err(err),
            };
        module.add_set("static_libs", static_libs);
        module.add_set("shared_libs", shared_libs);

        let generated_headers =
            match target.get_generated_headers(self.targets_map, self.target_prefix) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
        module.add_set("generated_headers", generated_headers.clone());
        for header in generated_headers {
            self.generated_headers.insert(header);
        }

        let mut result = match module.print() {
            Ok(return_value) => return_value,
            Err(err) => return Err(err),
        };

        if copy_for_device {
            let mut copy_module = crate::soongmodule::SoongModule::new("genrule");
            copy_module.add_str("name", target.get_name(self.target_prefix));
            copy_module.add_set("tools", [name.clone()].into());
            copy_module.add_set("out", [target.get_name(self.target_prefix)].into());
            copy_module.add_str("cmd", "cp $(location ".to_string() + &name + ") $(out)");
            result += &match copy_module.print() {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
        }
        return Ok(result);
    }

    fn rework_output_path(output: &str) -> String {
        let rework_output = if let Some(split) = output.split_once("include/") {
            split.1
        } else if !output.contains("libclc") {
            output.split("/").last().unwrap()
        } else {
            output
        };
        return String::from(rework_output);
    }

    fn generate_simple_genrule(
        &mut self,
        target: &BuildTarget,
        command: &str,
        error_prefix: &str,
    ) -> Result<String, String> {
        let mut module = crate::soongmodule::SoongModule::new("cc_genrule");
        module.add_str("name", target.get_name(self.target_prefix));
        if self.host_targets {
            module.add_bool("host_supported", true);
        }

        let inputs = target.get_inputs();
        let outputs = target.get_outputs();
        if inputs.len() != 1 || outputs.len() != 1 {
            return error!(format!(
                "{0} with wrong number of input/output: {target:#?}",
                error_prefix,
            ));
        }
        let input = &inputs[0];
        let input_set: HashSet<String> = [if input == "bin/clang-20" {
            ":".to_string() + &crate::target::rework_target_name(input.clone(), self.target_prefix)
        } else {
            input.replace(self.src_root, "")
        }]
        .into();
        module.add_set("srcs", input_set);
        module.add_set("out", [Self::rework_output_path(&outputs[0])].into());
        module.add_str("cmd", command.to_string());
        return module.print();
    }

    fn replace_output_in_command(command: String, output: &String) -> String {
        let marker = "<output>";
        let space_and_marker = String::from(" ") + marker;
        let space_and_last_output = String::from(" ") + output.split("/").last().unwrap();
        let command = command.replace(output, marker);
        let command = command.replace(&space_and_last_output, &space_and_marker);
        let replace_output = String::from("$(location ") + &Self::rework_output_path(output) + ")";
        return command.replace(marker, &replace_output);
    }
    fn replace_input_in_command(&self, command: String, input: &String) -> String {
        let replace_input = String::from("$(location ") + &input.replace(self.src_root, "") + ")";
        return command.replace(input, &replace_input);
    }
    fn replace_dep_in_command(&self, command: String, tool: &String, prefix: &str) -> String {
        let replace_tool = "$(location :".to_string()
            + &crate::target::rework_target_name(tool.clone(), self.target_prefix)
            + ")";
        let tool_with_prefix = String::from(prefix) + tool;
        let command = command.replace(&tool_with_prefix, &replace_tool);
        return command.replace(tool, &replace_tool);
    }
    fn replace_source_root_in_command(
        &self,
        command: String,
        input_ref_for_genrule: &String,
    ) -> String {
        let replace_with = String::from("$$(dirname $(location ") + input_ref_for_genrule + "))/";
        return command.replace(self.src_root, &replace_with);
    }

    fn rework_command(
        &self,
        command: String,
        inputs: &HashSet<&String>,
        outputs: &Vec<String>,
        generated_deps: &HashSet<&String>,
    ) -> Result<(String, bool), String> {
        let mut command = command.replace("/usr/bin/python3 ", "");
        command = command.replace(self.build_root, "");
        for output in outputs {
            command = Self::replace_output_in_command(command, output);
        }
        for input in inputs.clone() {
            command = self.replace_input_in_command(command, input);
        }
        for dep in generated_deps {
            command = self.replace_dep_in_command(command, dep, self.target_prefix);
        }
        let previous_command = command.clone();
        command =
            self.replace_source_root_in_command(command, &(self.input_ref_for_genrule.to_string()));
        return Ok((command.clone(), previous_command != command));
    }

    fn generate_custom_command(
        &mut self,
        target: &BuildTarget,
        command: String,
    ) -> Result<String, String> {
        let mut module = crate::soongmodule::SoongModule::new("cc_genrule");
        module.add_str("name", target.get_name(self.target_prefix));
        if self.host_targets {
            module.add_bool("host_supported", true);
        }

        let mut filtered_inputs: HashSet<&String> = HashSet::new();
        let mut generated_deps: HashSet<&String> = HashSet::new();
        for input in target.get_inputs() {
            if input.starts_with(self.src_root) {
                filtered_inputs.insert(input);
            } else {
                generated_deps.insert(input);
            }
        }
        let outputs = target.get_outputs();
        let input_ref_for_genrule = self.input_ref_for_genrule.to_string();
        let command =
            &match self.rework_command(command, &filtered_inputs, outputs, &generated_deps) {
                Ok((command, add_input_ref_in_filtered_inputs)) => {
                    if add_input_ref_in_filtered_inputs {
                        filtered_inputs.insert(&input_ref_for_genrule);
                    }
                    command
                }
                Err(err) => return Err(err),
            };
        let mut srcs_set: HashSet<String> = HashSet::new();
        for input in filtered_inputs {
            srcs_set.insert(input.replace(self.src_root, ""));
        }
        for dep in generated_deps {
            srcs_set.insert(
                ":".to_string()
                    + &crate::target::rework_target_name(dep.clone(), self.target_prefix),
            );
        }
        module.add_set("srcs", srcs_set);
        let mut out_set: HashSet<String> = HashSet::new();
        for output in outputs {
            out_set.insert(Self::rework_output_path(output));
        }
        module.add_set("out", out_set);
        module.add_str("cmd", command.to_string());
        return module.print();
    }

    fn generate_target(&mut self, target: &BuildTarget) -> Result<(), String> {
        let rule = target.get_rule();
        let result = if rule.starts_with("CXX_SHARED_LIBRARY") {
            self.generate_object(
                if self.host_targets {
                    "cc_libary_host_shared"
                } else {
                    "cc_library_shared"
                },
                target,
            )
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            self.generate_object(
                if self.host_targets {
                    "cc_library_host_static"
                } else {
                    "cc_library_static"
                },
                target,
            )
        } else if rule.starts_with("CXX_EXECUTABLE") {
            self.generate_object(
                if self.host_targets {
                    "cc_binary_host"
                } else {
                    "cc_binary"
                },
                target,
            )
        } else if rule.starts_with("CMAKE_SYMLINK") {
            self.generate_simple_genrule(target, "ls -s $(in) $(out)", "Symlink")
        } else if rule.starts_with("CUSTOM_COMMAND") {
            let command = match target.get_command() {
                Ok(option) => match option {
                    Some(command) => command,
                    None => return Ok(()),
                },
                Err(err) => return Err(err),
            };
            if command == crate::target::COPY_TARGET {
                self.generate_simple_genrule(target, "cp $(in) $(out)", "Copy")
            } else {
                self.generate_custom_command(target, command)
            }
        } else if rule.starts_with("CXX_COMPILER")
            || rule.starts_with("C_COMPILER")
            || rule.starts_with("ASM_COMPILER")
            || rule == "phony"
        {
            return Ok(());
        } else {
            error!(format!("unsupported rule ({rule}) for target: {target:#?}"))
        };
        match result {
            Ok(package) => {
                self.content += &package;
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }
}

fn create_map(targets: &Vec<BuildTarget>) -> HashMap<String, &BuildTarget> {
    let mut map: HashMap<String, &BuildTarget> = HashMap::new();
    for target in targets {
        for output in &target.get_all_outputs() {
            map.insert(output.clone(), target);
        }
    }

    return map;
}

pub fn generate(
    entry_targets: Vec<&str>,
    targets: &Vec<BuildTarget>,
    src_root: &str,
    ndk_root: &str,
    build_root: &str,
    target_prefix: &str,
    host_targets: bool,
    input_ref_for_genrule: &str,
    dst_build_prefix: &str,
) -> Result<(String, HashSet<String>, HashSet<String>, HashSet<String>), String> {
    let mut target_seen: HashSet<String> = HashSet::new();
    let mut target_to_generate = entry_targets
        .into_iter()
        .fold(Vec::new(), |mut vec, element| {
            vec.push(element.to_string());
            vec
        });
    let targets_map = create_map(targets);
    let mut soong_file = SoongFile::new(
        &targets_map,
        src_root,
        ndk_root,
        build_root,
        target_prefix,
        host_targets,
        input_ref_for_genrule,
        dst_build_prefix,
    );

    while let Some(input) = target_to_generate.pop() {
        if target_seen.contains(&input) || input.contains("llvm/bin") {
            continue;
        }
        let Some(target) = targets_map.get(&input) else {
            continue;
        };

        target_to_generate.append(&mut target.get_all_inputs());
        for output in target.get_all_outputs() {
            target_seen.insert(output);
        }

        if let Err(err) = soong_file.generate_target(target) {
            return Err(err);
        }
    }
    // let _ = crate::filesystem::write_file(
    //     if host_targets {
    //         "host_targets.txt"
    //     } else {
    //         "targets.txt"
    //     },
    //     format!("{target_seen:#?}"),
    // );
    return Ok(soong_file.finish());
}
