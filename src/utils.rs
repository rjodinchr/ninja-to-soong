#[macro_export]
macro_rules! error {
    ($message:expr) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), $message))
    };
}
pub use error;

use std::collections::HashSet;

pub fn rework_target_name(target_name: &str, prefix: &str) -> String {
    let mut name = prefix.to_string() + target_name;
    name = name.strip_suffix(".so").unwrap_or(&name).to_string();
    name = name.strip_suffix(".a").unwrap_or(&name).to_string();
    return name.replace("/", "__").replace(".", "__");
}

pub fn rework_source_path(source: &str, source_root: &str) -> String {
    let source = source.strip_prefix(source_root).unwrap_or(&source);
    return String::from(source);
}

pub fn rework_output_path(output: &str) -> String {
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
    let replace_input = String::from("$(location ") + &rework_source_path(input, source_root) + ")";
    return command.replace(input, &replace_input);
}

fn replace_dep_in_command(command: String, tool: &String, prefix: &str) -> String {
    let replace_tool = "$(location ".to_string() + &rework_target_name(tool, prefix) + ")";
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

pub fn rework_command<'a>(
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
