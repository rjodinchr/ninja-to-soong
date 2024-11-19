use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;
use crate::target::BuildTarget;

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
        "$(location ".to_string() + &crate::target::rework_target_name(tool, prefix) + ")";
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

fn print_hashset(set: HashSet<String>, set_name: &str, fct: fn(&mut String, &String)) -> String {
    let mut result = String::new();
    if set.len() == 0 {
        return result;
    }
    result += "\t";
    result += set_name;
    result += ": [\n";
    for set_element in &set {
        result += "\t\t\"";
        fct(&mut result, set_element);
        result += "\",\n";
    }
    result += "\t],\n";
    return result;
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
) -> Result<String, String> {
    let mut result = String::new();
    result += name;
    result += " {\n";

    result += "\tname: \"";
    result += &crate::target::rework_target_name(&crate::target::get_name(target), prefix);
    result += "\",\n";

    let mut includes: HashSet<String> = HashSet::new();
    let mut defines: HashSet<String> = HashSet::new();
    result += "\tsrcs: [\n";
    for input in crate::target::get_inputs(target) {
        result += "\t\t\"";
        let (src, src_includes, src_defines) = match crate::target::get_compiler_target_info(
            input,
            targets_map,
            source_root,
            build_root,
        ) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };
        for inc in src_includes {
            includes.insert(inc);
        }
        for def in src_defines {
            defines.insert(def);
        }
        result += &src;
        result += "\",\n";
    }
    result += "\t],\n";

    result += &print_hashset(includes, "local_include_dirs", |result, element| {
        result.push_str(element)
    });
    result += &print_hashset(defines, "cflags", |result, element| {
        result.push_str(&format!("-D{element}"))
    });
    let (version_script, link_flags) = crate::target::get_link_flags(target, source_root);
    result += &print_hashset(link_flags, "ldflags", |result, element| {
        result.push_str(element)
    });
    if let Some(vs) = version_script {
        result += "\tversion_script: \"";
        result += &vs;
        result += "\",\n";
    }

    let (static_libs, shared_libs, system_shared_libs) =
        match crate::target::get_link_libraries(target, native_lib_root, prefix) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };
    result += &print_hashset(
        system_shared_libs,
        "system_shared_libs",
        |result, element| result.push_str(element),
    );
    result += &print_hashset(static_libs, "static_libs", |result, element| {
        result.push_str(element)
    });
    result += &print_hashset(shared_libs, "shared_libs", |result, element| {
        result.push_str(element)
    });

    let generated_headers = match crate::target::get_generated_headers(target, &targets_map, prefix)
    {
        Ok(generated_headers) => generated_headers,
        Err(err) => return Err(err),
    };
    result += &print_hashset(generated_headers, "generated_headers", |result, element| {
        result.push_str(element)
    });

    if optimize_for_size {
        result += "\toptimize_for_size: true,\n";
    }

    result += "}\n\n";
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
    let mut result = String::new();
    result += "cc_genrule {\n";

    result += "\tname: \"";
    result += &crate::target::rework_target_name(&crate::target::get_name(target), prefix);
    result += "\",\n";

    let inputs = crate::target::get_inputs(target);
    let outputs = crate::target::get_outputs(target);
    if inputs.len() != 1 || outputs.len() != 1 {
        return error!(format!(
            "{0} with wrong number of input/output: {target:#?}",
            error_prefix,
        ));
    }

    let input = &inputs[0];
    if input == "bin/clang-20" {
        result += "\ttools: [\n\t\t\"";
        result += &crate::target::rework_target_name(input, prefix);
    } else {
        result += "\tsrcs: [\n\t\t\"";
        result += &crate::target::rework_source_path(input, source_root);
    }
    result += "\",\n\t],\n";

    result += "\tout: [\n\t\t\"";
    result += &rework_output_path(&outputs[0]);
    result += "\",\n\t],\n";

    if host {
        result += "\thost_supported: true,\n";
    }

    result += "\tcmd: \"";
    result += command;
    result += "\",\n";

    result += "}\n\n";
    return Ok(result);
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
        let mut result = String::new();
        result += "cc_genrule {\n";

        result += "\tname: \"";
        result += &crate::target::rework_target_name(&crate::target::get_name(target), prefix);
        result += "\",\n";

        let inputs = crate::target::get_inputs(target);
        let mut filtered_inputs: HashSet<&String> = HashSet::new();
        let mut generated_deps: HashSet<&String> = HashSet::new();
        for input in inputs {
            if input.starts_with(source_root) {
                filtered_inputs.insert(input);
            } else {
                generated_deps.insert(input);
            }
        }
        let outputs = crate::target::get_outputs(target);
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

        if filtered_inputs.len() + generated_deps.len() > 0 {
            result += "\tsrcs: [\n";
            for input in filtered_inputs {
                result += "\t\t\"";
                result += &crate::target::rework_source_path(input, source_root);
                result += "\",\n";
            }
            for dep in generated_deps {
                result += "\t\t\":";
                result += &crate::target::rework_target_name(dep, prefix);
                result += "\",\n";
            }
            result += "\t],\n";
        }

        result += "\tout: [\n";
        for output in outputs {
            result += "\t\t\"";
            result += &rework_output_path(output);
            result += "\",\n";
        }
        result += "\t],\n";

        if host {
            result += "\thost_supported: true,\n";
        }

        result += "\tcmd: \"";
        result += command;
        result += "\",\n";

        result += "}\n\n";
        return Ok(result);
    }
    fn generate_copy(
        &self,
        target: &BuildTarget,
        source_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String> {
        generate_simple_genrule(
            target,
            source_root,
            prefix,
            "cp $(in) $(out)",
            "Copy",
            host,
        )
    }
    fn generate_cmake_link(
        &self,
        target: &BuildTarget,
        source_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String> {
        generate_simple_genrule(
            target,
            source_root,
            prefix,
            "ln -s $(in) $(out)",
            "Symlink",
            host,
        )
    }
}
