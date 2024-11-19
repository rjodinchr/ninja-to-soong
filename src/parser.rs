use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::macros::error;
use crate::target::BuildTarget;

fn parse_output_section(section: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let mut split = section.split("|");
    if split.clone().count() < 1 {
        return error!(format!("parse_output_section failed: '{section}'"));
    }
    let mut target_outputs: Vec<String> = Vec::new();
    for output in split.nth(0).unwrap().trim().split(" ") {
        target_outputs.push(String::from(output.trim()));
    }
    let mut target_implicit_outputs: Vec<String> = Vec::new();
    if let Some(implicit_outputs) = split.next() {
        for implicit_output in implicit_outputs.trim().split(" ") {
            target_implicit_outputs.push(String::from(implicit_output.trim()));
        }
    }
    return Ok((target_outputs, target_implicit_outputs));
}

fn parse_input_and_rule_section(section: &str) -> Result<(String, Vec<String>), String> {
    let mut split = section.trim().split(" ");
    if split.clone().count() < 1 {
        return error!(format!("parse_input_and_rule_section failed: '{section}'"));
    }
    let target_rule = String::from(split.nth(0).unwrap());
    let mut target_inputs: Vec<String> = Vec::new();
    for input in split {
        target_inputs.push(String::from(input.trim()));
    }
    return Ok((target_rule, target_inputs));
}

fn parse_input_and_dep_section(
    section: &str,
) -> Result<(String, Vec<String>, Vec<String>), String> {
    let mut split = section.split("|");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!(format!("parse_input_and_dep_section failed: '{section}'"));
    }

    let (target_rule, target_inputs) = match parse_input_and_rule_section(split.nth(0).unwrap()) {
        Ok(return_values) => return_values,
        Err(err) => return Err(err),
    };

    let mut target_implicit_dependencies: Vec<String> = Vec::new();
    if let Some(implicit_dependencies) = split.next() {
        for implicit_dep in implicit_dependencies.trim().split(" ") {
            target_implicit_dependencies.push(String::from(implicit_dep.trim()));
        }
    }
    return Ok((target_rule, target_inputs, target_implicit_dependencies));
}

fn parse_input_section(
    section: &str,
) -> Result<(String, Vec<String>, Vec<String>, Vec<String>), String> {
    let mut split = section.split("||");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!(format!("parse_input_section failed: '{section}'"));
    }

    let (target_rule, target_inputs, target_implicit_dependencies) =
        match parse_input_and_dep_section(split.nth(0).unwrap()) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };

    let mut target_order_only_dependencies: Vec<String> = Vec::new();
    if let Some(order_only_dependencies) = split.next() {
        for dep in order_only_dependencies.trim().split(" ") {
            target_order_only_dependencies.push(String::from(dep.trim()));
        }
    }
    return Ok((
        target_rule,
        target_inputs,
        target_implicit_dependencies,
        target_order_only_dependencies,
    ));
}

fn parse_build_target(line: &str, lines: &mut std::str::Lines<'_>) -> Result<BuildTarget, String> {
    let Some(line_stripped) = line.strip_prefix("build") else {
        return error!(format!("No build prefix: '{line}'"));
    };

    let mut split = line_stripped.split(":");
    let split_count = split.clone().count();
    if split_count != 2 {
        return error!(format!(
            "Error while parsing column (expected 2 splits, got {0}): '{split:#?}'",
            split_count,
        ));
    }

    let (target_outputs, target_implicit_outputs) =
        match parse_output_section(split.nth(0).unwrap()) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };
    let (target_rule, target_inputs, target_implicit_dependencies, target_order_only_dependencies) =
        match parse_input_section(split.nth(0).unwrap()) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };

    let mut target_variables: HashMap<String, String> = HashMap::new();
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with("  ") {
            break;
        }
        let Some(split) = next_line.split_once("=") else {
            return error!(format!("Error while parsing variable: '{next_line}'"));
        };
        let key = String::from(split.0.trim());
        let val = String::from(split.1.trim());
        target_variables.insert(key, val);
    }

    return Ok(crate::target::BuildTarget::new(
        target_rule,
        target_outputs,
        target_implicit_outputs,
        target_inputs,
        target_implicit_dependencies,
        target_order_only_dependencies,
        target_variables,
    ));
}

pub fn parse_build_ninja(path: &str) -> Result<Vec<BuildTarget>, String> {
    let mut targets: Vec<BuildTarget> = Vec::new();
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return error!(format!("Could not open '{path}': '{0}'", err)),
    };
    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        return error!(format!("Could not read '{path}': '{err:#?}'"));
    }
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if !line.trim().starts_with("build") {
            continue;
        }
        match parse_build_target(line, &mut lines) {
            Ok(target) => targets.push(target),
            Err(err) => return error!(format!("Could not parse build target: '{err}'")),
        }
    }
    return Ok(targets);
}
