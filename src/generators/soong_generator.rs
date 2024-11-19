use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;
use crate::target::BuildTarget;

#[derive(Debug)]
struct SoongPackage {
    name: String,
    single_string_map: HashMap<String, String>,
    list_string_map: HashMap<String, HashSet<String>>,
    optimize_for_size: bool,
    host_supported: bool,
}

impl SoongPackage {
    fn new(name: &str, optimize_for_size: bool, host_supported: bool) -> Self {
        Self {
            name: name.to_string(),
            single_string_map: HashMap::new(),
            list_string_map: HashMap::new(),
            optimize_for_size,
            host_supported,
        }
    }

    fn add_single_string(&mut self, key: &str, value: String) {
        self.single_string_map.insert(key.to_string(), value);
    }

    fn add_list_string(&mut self, key: &str, set: HashSet<String>) {
        self.list_string_map.insert(key.to_string(), set);
    }

    fn add_list_string_single(&mut self, key: &str, value: String) {
        let mut set: HashSet<String> = HashSet::new();
        set.insert(value);
        self.list_string_map.insert(key.to_string(), set);
    }

    fn print_single_string(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, value)) = self.single_string_map.remove_entry(entry) else {
            return result;
        };
        if value == "" {
            return result;
        }
        result += "    ";
        result += &key;
        result += ": \"";
        result += &value;
        result += "\",\n";

        return result;
    }

    fn print_list_string(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, mut set)) = self.list_string_map.remove_entry(entry) else {
            return result;
        };
        set.remove("");
        if set.len() == 0 {
            return result;
        }
        result += "    ";
        result += &key;
        result += ": ";

        result += "[\n";
        for value in set {
            result += "        \"";
            result += &value;
            result += "\",\n";
        }
        result += "    ],\n";
        return result;
    }

    fn print(mut self) -> Result<String, String> {
        let mut result = String::new();
        result += &self.name;
        result += " {\n";

        if !self.single_string_map.contains_key("name") {
            return error!(format!("no 'name' in soong package: '{self:#?}"));
        }
        result += &self.print_single_string("name");
        result += &self.print_list_string("srcs");
        result += &self.print_list_string("out");
        result += &self.print_list_string("tools");
        result += &self.print_list_string("cflags");
        result += &self.print_list_string("ldflags");
        result += &self.print_single_string("version_script");
        result += &self.print_list_string("system_shared_libs");
        result += &self.print_list_string("shared_libs");
        result += &self.print_list_string("static_libs");
        result += &self.print_list_string("local_include_dirs");
        result += &self.print_list_string("generated_headers");
        result += &self.print_single_string("cmd");

        if self.single_string_map.len() > 0 || self.list_string_map.len() > 0 {
            return error!(format!("entries not consumed in: '{self:#?}"));
        }
        if self.optimize_for_size {
            result += "    optimize_for_size: true,\n";
        }
        if self.host_supported {
            result += "    host_supported: true,\n";
        }

        result += "}\n\n";
        return Ok(result);
    }
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

fn replace_output_in_command(command: String, output: &String) -> String {
    let marker = "<output>";
    let space_and_marker = String::from(" ") + marker;
    let space_and_last_output = String::from(" ") + output.split("/").last().unwrap();

    let command = command.replace(output, marker);
    let command = command.replace(&space_and_last_output, &space_and_marker);

    let replace_output = String::from("$(location ") + &rework_output_path(output) + ")";
    return command.replace(marker, &replace_output);
}

fn replace_input_in_command(command: String, input: &String, source_root: &str) -> String {
    let replace_input =
        String::from("$(location ") + &crate::target::rework_source_path(input, source_root) + ")";
    return command.replace(input, &replace_input);
}

fn replace_dep_in_command(command: String, tool: &String, prefix: &str) -> String {
    let replace_tool =
        "$(location :".to_string() + &crate::target::rework_target_name(tool, prefix) + ")";
    let tool_with_prefix = String::from(prefix) + tool;
    let command = command.replace(&tool_with_prefix, &replace_tool);
    return command.replace(tool, &replace_tool);
}

fn replace_source_root_in_command(
    command: String,
    source_root: &str,
    input_ref_for_genrule: &String,
) -> String {
    let replace_with = String::from("$$(dirname $(location ") + input_ref_for_genrule + "))/";
    return command.replace(source_root, &replace_with);
}

fn rework_command<'a>(
    command: String,
    inputs: &mut HashSet<&'a String>,
    outputs: &Vec<String>,
    generated_deps: &HashSet<&String>,
    source_root: &str,
    build_root: &str,
    prefix: &str,
    input_ref_for_genrule: &'a String,
) -> Result<String, String> {
    let mut command = command.replace("/usr/bin/python3 ", "");
    command = command.replace(build_root, "");

    for output in outputs {
        command = replace_output_in_command(command, output);
    }
    for input in inputs.clone() {
        command = replace_input_in_command(command, input, source_root);
    }
    for dep in generated_deps {
        command = replace_dep_in_command(command, dep, prefix);
    }

    let previous_command = command.clone();
    command = replace_source_root_in_command(command, source_root, input_ref_for_genrule);
    if previous_command != command {
        inputs.insert(input_ref_for_genrule);
    }

    return Ok(command);
}

fn generate_object(
    name: &str,
    target: &BuildTarget,
    targets_map: &HashMap<String, &BuildTarget>,
    source_root: &str,
    native_lib_root: &str,
    build_root: &str,
    prefix: &str,
    optimize_for_size: bool,
    copy_for_device: bool,
) -> Result<String, String> {
    let mut package = SoongPackage::new(name, optimize_for_size, false);
    let target_name = crate::target::rework_target_name(target.get_name(), prefix);
    package.add_single_string(
        "name",
        if copy_for_device {
            "HOST_".to_string() + &target_name
        } else {
            target_name.clone()
        },
    );

    let mut includes: HashSet<String> = HashSet::new();
    let mut defines: HashSet<String> = HashSet::new();
    let mut srcs: HashSet<String> = HashSet::new();
    for input in target.get_inputs() {
        let Some(target) = targets_map.get(input) else {
            return error!(format!("unsupported input for library: {input}"));
        };
        let (src, src_includes, src_defines) =
            match target.get_compiler_target_info(source_root, build_root) {
                Ok(return_values) => return_values,
                Err(err) => return Err(err),
            };
        for inc in src_includes {
            includes.insert(inc);
        }
        for def in src_defines {
            defines.insert(String::from("-D") + &def);
        }
        srcs.insert(src);
    }
    package.add_list_string("srcs", srcs);
    package.add_list_string("local_include_dirs", includes);
    package.add_list_string("cflags", defines);

    let (version_script, link_flags) = target.get_link_flags(source_root);
    package.add_list_string("ldflags", link_flags);
    package.add_single_string("version_script", version_script);

    let (static_libs, shared_libs, system_shared_libs) =
        match target.get_link_libraries(native_lib_root, prefix) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };

    package.add_list_string("system_shared_libs", system_shared_libs);
    package.add_list_string("static_libs", static_libs);
    package.add_list_string("shared_libs", shared_libs);

    let generated_headers = match target.get_generated_headers(&targets_map, prefix) {
        Ok(generated_headers) => generated_headers,
        Err(err) => return Err(err),
    };
    package.add_list_string("generated_headers", generated_headers);

    let mut result = match package.print() {
        Ok(return_value) => return_value,
        Err(err) => return Err(err),
    };

    if copy_for_device {
        let mut copy_package = SoongPackage::new("genrule", false, false);
        copy_package.add_single_string("name", target_name.clone());
        copy_package.add_list_string_single("tools", ":HOST_".to_string() + &target_name);
        copy_package.add_list_string_single("out", target.get_name().clone());
        copy_package.add_single_string(
            "cmd",
            "cp $(location :HOST_".to_string() + &target_name + ") $(out)",
        );

        result += &match copy_package.print() {
            Ok(return_value) => return_value,
            Err(err) => return Err(err),
        };
    }

    return Ok(result);
}

fn generate_simple_genrule(
    target: &BuildTarget,
    source_root: &str,
    prefix: &str,
    command: &str,
    error_prefix: &str,
    host: bool,
) -> Result<String, String> {
    let mut package = SoongPackage::new("cc_genrule", false, host);
    package.add_single_string(
        "name",
        crate::target::rework_target_name(&target.get_name(), prefix),
    );

    let inputs = target.get_inputs();
    let outputs = target.get_outputs();
    if inputs.len() != 1 || outputs.len() != 1 {
        return error!(format!(
            "{0} with wrong number of input/output: {target:#?}",
            error_prefix,
        ));
    }

    let input = &inputs[0];
    if input == "bin/clang-20" {
        package.add_list_string_single("tools", crate::target::rework_target_name(input, prefix));
    } else {
        package.add_list_string_single(
            "srcs",
            crate::target::rework_source_path(input, source_root),
        );
    }
    package.add_list_string_single("out", rework_output_path(&outputs[0]));
    package.add_single_string("cmd", command.to_string());

    return package.print();
}

pub struct SoongGenerator();

impl crate::generators::Generator for SoongGenerator {
    fn generate_shared_library(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String> {
        generate_object(
            if host {
                "cc_library_host_shared"
            } else {
                "cc_library_shared"
            },
            target,
            targets_map,
            source_root,
            native_lib_root,
            build_root,
            prefix,
            false,
            false,
        )
    }
    fn generate_static_library(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String> {
        generate_object(
            if host {
                "cc_library_host_static"
            } else {
                "cc_library_static"
            },
            target,
            &targets_map,
            source_root,
            native_lib_root,
            build_root,
            prefix,
            !host,
            false,
        )
    }
    fn generate_executable(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String> {
        generate_object(
            if host { "cc_binary_host" } else { "cc_binary" },
            target,
            &targets_map,
            source_root,
            native_lib_root,
            build_root,
            prefix,
            false,
            host,
        )
    }
    fn generate_custom_command(
        &self,
        target: &BuildTarget,
        command: String,
        source_root: &str,
        build_root: &str,
        prefix: &str,
        input_ref_for_genrule: &String,
        host: bool,
    ) -> Result<String, String> {
        let mut package = SoongPackage::new("cc_genrule", false, host);
        package.add_single_string(
            "name",
            crate::target::rework_target_name(&target.get_name(), prefix),
        );

        let mut filtered_inputs: HashSet<&String> = HashSet::new();
        let mut generated_deps: HashSet<&String> = HashSet::new();
        for input in target.get_inputs() {
            if input.starts_with(source_root) {
                filtered_inputs.insert(input);
            } else {
                generated_deps.insert(input);
            }
        }
        let outputs = target.get_outputs();
        let command = &match rework_command(
            command,
            &mut filtered_inputs,
            outputs,
            &generated_deps,
            source_root,
            build_root,
            prefix,
            input_ref_for_genrule,
        ) {
            Ok(command) => command,
            Err(err) => return Err(err),
        };

        let mut srcs_set: HashSet<String> = HashSet::new();
        for input in filtered_inputs {
            srcs_set.insert(crate::target::rework_source_path(input, source_root));
        }
        for dep in generated_deps {
            srcs_set.insert(":".to_string() + &crate::target::rework_target_name(dep, prefix));
        }
        package.add_list_string("srcs", srcs_set);

        let mut out_set: HashSet<String> = HashSet::new();
        for output in outputs {
            out_set.insert(rework_output_path(output));
        }
        package.add_list_string("out", out_set);
        package.add_single_string("cmd", command.to_string());

        return package.print();
    }
    fn generate_copy(
        &self,
        target: &BuildTarget,
        source_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String> {
        generate_simple_genrule(target, source_root, prefix, "cp $(in) $(out)", "Copy", host)
    }
    fn generate_cmake_link(
        &self,
        target: &BuildTarget,
        source_root: &str,
        prefix: &str,
    ) -> Result<String, String> {
        generate_simple_genrule(
            target,
            source_root,
            prefix,
            "ln -s $(in) $(out)",
            "Symlink",
            false,
        )
    }
}
