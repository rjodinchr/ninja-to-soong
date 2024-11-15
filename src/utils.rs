pub fn rework_target_name(target_name: &str) -> String {
    let mut name = target_name;
    name = name.strip_suffix(".so").unwrap_or(name);
    name = name.strip_suffix(".a").unwrap_or(name);
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
    let reworked_output = rework_output_path(output);
    let mut replace_output = String::from("$(location ");
    replace_output += &reworked_output;
    replace_output += ")";
    let command = command.replace(output, &replace_output);

    let mut second_output_pattern = String::new();
    second_output_pattern += "-o ";
    second_output_pattern += output.split("/").last().unwrap();
    second_output_pattern += " ";

    let mut second_replace_output = String::new();
    second_replace_output += "-o ";
    second_replace_output += &replace_output;
    second_replace_output += " ";

    command.replace(&second_output_pattern, &second_replace_output)
}

fn replace_input_in_command(command: String, input: &String, source_root: &str) -> String {
    let mut replace_input = String::from("$(location ");
    replace_input += &crate::utils::rework_source_path(input, source_root);
    replace_input += ")";
    return command.replace(input, &replace_input);
}

fn get_location_source_root(inputs: &Vec<&String>, source_root: &str) -> Option<String> {
    if inputs.len() < 1 {
        return None;
    }
    let input = crate::utils::rework_source_path(inputs[0], source_root);
    let split = input.split("/");

    let mut result = String::new();
    result += "$$(dirname $(location ";
    result += &input;
    result += "))/";
    for _ in 1..split.count() {
        result += "../";
    }
    return Some(result);
}

pub fn rework_command(
    command: String,
    inputs: &mut Vec<&String>,
    outputs: &Vec<String>,
    source_root: &str,
    build_root: &str,
) -> Result<String, String> {
    let command = match command.split_once(" && ") {
        Some(split) => split.1,
        None => return Err(format!("Could not split command: {command}")),
    };
    let command = if let Some(split) = command.split_once(" -d ") {
        split.0
    } else {
        command
    };

    let split = command.split_once(" ").unwrap();
    let mut command = if split.0.contains("python") {
        split.1.split_once(" ").unwrap().1
    } else {
        split.1
    }
    .to_string();

    command = command.replace(build_root, "");
    for output in outputs {
        command = replace_output_in_command(command, output);
    }
    for input in inputs.clone() {
        command = replace_input_in_command(command, input, source_root);
    }

    if let Some(location_source_root) = get_location_source_root(inputs, source_root) {
        while let Some(start_pos) = command.find(source_root) {
            let to_replace = if let Some(end_pos) = command[start_pos..].find(" ") {
                &command[start_pos..(start_pos + end_pos)]
            } else {
                &command[start_pos..]
            };
            let to_replace_reworked = crate::utils::rework_source_path(to_replace, source_root);
            let mut replace_with = String::new();
            replace_with += &location_source_root;
            replace_with += &to_replace_reworked;
            command = command.replace(to_replace, &replace_with);
        }
    }

    let mut result = String::new();
    result += "$(location) ";
    result += &command;

    return Ok(result);
}
