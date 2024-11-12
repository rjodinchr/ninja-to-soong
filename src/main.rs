use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
struct BuildTarget {
    rule: String,
    outputs: Vec<String>,
    implicit_outputs: Vec<String>,
    inputs: Vec<String>,
    implicit_dependencies: Vec<String>,
    order_only_dependencies: Vec<String>,
    variables: HashMap<String, String>,
}

fn parse_build_target(line: &str, lines: &mut std::str::Lines<'_>) -> Result<BuildTarget, String> {
    let Some(line_stripped) = line.strip_prefix("build") else {
        return Err(format!("No build prefix: '{line}'"));
    };

    let mut split_column = line_stripped.split(":");
    let split_column_count = split_column.clone().count();
    if split_column_count != 2 {
        return Err(format!(
            "Error while parsing column (expected 2, got {0}): '{split_column:#?}'",
            split_column_count,
        ));
    }
    let output_section = split_column.nth(0).unwrap();
    let input_section = split_column.nth(0).unwrap();

    let mut split_pipe_output_section = output_section.split("|");
    if split_pipe_output_section.clone().count() < 1 {
        return Err(format!(
            "Error while parsing output section for pipe: '{output_section}'"
        ));
    }
    let mut target_outputs: Vec<String> = Vec::new();
    for output in split_pipe_output_section.nth(0).unwrap().trim().split(" ") {
        target_outputs.push(String::from(output.trim()));
    }
    let mut target_implicit_outputs: Vec<String> = Vec::new();
    if let Some(implicit_outputs) = split_pipe_output_section.next() {
        for implicit_output in implicit_outputs.trim().split(" ") {
            target_implicit_outputs.push(String::from(implicit_output.trim()));
        }
    }

    let mut split_pipe_input_section = input_section.split("||");
    let split_pipe_input_section_count = split_pipe_input_section.clone().count();
    if split_pipe_input_section_count != 1 && split_pipe_input_section_count != 2 {
        return Err(format!(
            "Error while parsing input section for double pipe: '{input_section}'"
        ));
    }
    let input_and_dep_section = split_pipe_input_section.nth(0).unwrap();

    let mut split_pipe_input_and_dep_section = input_and_dep_section.split("|");
    let split_pipe_input_and_dep_section_count = split_pipe_input_and_dep_section.clone().count();
    if split_pipe_input_and_dep_section_count != 1 && split_pipe_input_and_dep_section_count != 2 {
        return Err(format!(
            "Error while parsing input section for pipe: '{input_and_dep_section}'"
        ));
    }
    let inputs_and_rule = split_pipe_input_and_dep_section.nth(0).unwrap();

    let mut split_inputs_and_rule = inputs_and_rule.trim().split(" ");
    if split_inputs_and_rule.clone().count() < 1 {
        return Err(format!(
            "Error while parsing inputs and rule: '{inputs_and_rule}'"
        ));
    }
    let target_rule = String::from(split_inputs_and_rule.nth(0).unwrap());
    let mut target_inputs: Vec<String> = Vec::new();
    for input in split_inputs_and_rule {
        target_inputs.push(String::from(input.trim()));
    }
    let mut target_implicit_dependencies: Vec<String> = Vec::new();
    if let Some(implicit_dependencies) = split_pipe_input_and_dep_section.next() {
        for implicit_dep in implicit_dependencies.split(" ") {
            target_implicit_dependencies.push(String::from(implicit_dep.trim()));
        }
    }
    let mut target_order_only_dependencies: Vec<String> = Vec::new();
    if let Some(order_only_dependencies) = split_pipe_input_section.next() {
        for dep in order_only_dependencies.trim().split(" ") {
            target_order_only_dependencies.push(String::from(dep.trim()));
        }
    }
    let mut target_variables: HashMap<String, String> = HashMap::new();
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with("  ") {
            break;
        }
        let mut split = next_line.split("=");
        if split.clone().count() < 2 {
            return Err(format!("Error while parsing variable: '{next_line}'"));
        }
        let key = String::from(split.nth(0).unwrap().trim());
        let val = String::from(split.fold(String::new(), |res, elem| res + elem).trim());
        target_variables.insert(key, val);
    }

    return Ok(BuildTarget {
        rule: target_rule,
        outputs: target_outputs,
        implicit_outputs: target_implicit_outputs,
        inputs: target_inputs,
        implicit_dependencies: target_implicit_dependencies,
        order_only_dependencies: target_order_only_dependencies,
        variables: target_variables,
    });
}

fn parse_build_ninja(path: &str) -> Result<Vec<BuildTarget>, String> {
    let mut targets: Vec<BuildTarget> = Vec::new();
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(format!("Could not open '{path}': '{0}'", err)),
    };
    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        return Err(format!("Could not read '{path}': '{0}'", err));
    }
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if !line.trim().starts_with("build") {
            continue;
        }
        match parse_build_target(line, &mut lines) {
            Ok(target) => targets.push(target),
            Err(err) => return Err(format!("Could not parse build target: '{err}'")),
        }
    }
    return Ok(targets);
}

fn main() {
    let targets = match parse_build_ninja(
        "/usr/local/google/home/rjodin/work/clvk/build_android/build/build.ninja",
    ) {
        Ok(targets) => targets,
        Err(err) => {
            println!("Could not parse build.ninja: '{err}'");
            return;
        }
    };
    println!("{targets:#?}");
}
