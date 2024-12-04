// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::ninja_target::NinjaTarget;
use crate::utils::*;

fn parse_output_section(section: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let mut split = section.split("|");
    if split.clone().count() < 1 {
        return error!("parse_output_section failed: '{section}'");
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
    Ok((target_outputs, target_implicit_outputs))
}

fn parse_input_and_rule_section(section: &str) -> Result<(String, Vec<String>), String> {
    let mut split = section.trim().split(" ");
    if split.clone().count() < 1 {
        return error!("parse_input_and_rule_section failed: '{section}'");
    }
    let target_rule = String::from(split.nth(0).unwrap());
    let mut target_inputs: Vec<String> = Vec::new();
    for input in split {
        target_inputs.push(String::from(input.trim()));
    }
    Ok((target_rule, target_inputs))
}

fn parse_input_and_deps_section(
    section: &str,
) -> Result<(String, Vec<String>, Vec<String>), String> {
    let mut split = section.split("|");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!("parse_input_and_deps_section failed: '{section}'");
    }

    let (target_rule, target_inputs) = parse_input_and_rule_section(split.nth(0).unwrap())?;

    let mut target_implicit_dependencies: Vec<String> = Vec::new();
    if let Some(implicit_dependencies) = split.next() {
        for implicit_dep in implicit_dependencies.trim().split(" ") {
            target_implicit_dependencies.push(String::from(implicit_dep.trim()));
        }
    }
    Ok((target_rule, target_inputs, target_implicit_dependencies))
}

fn parse_input_section(
    section: &str,
) -> Result<(String, Vec<String>, Vec<String>, Vec<String>), String> {
    let mut split = section.split("||");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!("parse_input_section failed: '{section}'");
    }

    let (target_rule, target_inputs, target_implicit_dependencies) =
        parse_input_and_deps_section(split.nth(0).unwrap())?;

    let mut target_order_only_dependencies: Vec<String> = Vec::new();
    if let Some(order_only_dependencies) = split.next() {
        for dep in order_only_dependencies.trim().split(" ") {
            target_order_only_dependencies.push(String::from(dep.trim()));
        }
    }
    Ok((
        target_rule,
        target_inputs,
        target_implicit_dependencies,
        target_order_only_dependencies,
    ))
}

fn parse_build_target(line: &str, lines: &mut std::str::Lines<'_>) -> Result<NinjaTarget, String> {
    let Some(line_stripped) = line.strip_prefix("build") else {
        return error!("parse_build_target failed: '{line}'");
    };

    let mut split = line_stripped.split(":");
    let split_count = split.clone().count();
    if split_count != 2 {
        return error!("parse_build_target failed: '{line}'");
    }

    let (target_outputs, target_implicit_outputs) = parse_output_section(split.nth(0).unwrap())?;
    let (target_rule, target_inputs, target_implicit_dependencies, target_order_only_dependencies) =
        parse_input_section(split.nth(0).unwrap())?;

    let mut target_variables: HashMap<String, String> = HashMap::new();
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with("  ") {
            break;
        }
        let Some(split) = next_line.split_once("=") else {
            return error!("parse_build_target failed: '{next_line}'");
        };
        let key = String::from(split.0.trim());
        let val = String::from(split.1.trim());
        target_variables.insert(key, val);
    }

    Ok(NinjaTarget::new(
        target_rule,
        target_outputs,
        target_implicit_outputs,
        target_inputs,
        target_implicit_dependencies,
        target_order_only_dependencies,
        target_variables,
    ))
}

pub fn parse_build_ninja(path: String) -> Result<Vec<NinjaTarget>, String> {
    let mut targets: Vec<NinjaTarget> = Vec::new();
    let file = read_file(&(path + "/build.ninja"))?;
    let mut lines = file.lines();
    while let Some(line) = lines.next() {
        if !line.trim().starts_with("build") {
            continue;
        }
        targets.push(parse_build_target(line, &mut lines)?);
    }

    Ok(targets)
}
