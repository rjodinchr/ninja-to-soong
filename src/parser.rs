// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::ninja_target::*;
use crate::utils::*;

const INDENT: &str = "  ";

fn parse_output_section(section: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut split = section.split("|");
    if split.clone().count() < 1 {
        return error!("parse_output_section failed: '{section}'");
    }
    let mut target_outputs = Vec::new();
    for output in split.nth(0).unwrap().trim().split(" ") {
        target_outputs.push(PathBuf::from(output.trim()));
    }
    let mut target_implicit_outputs = Vec::new();
    if let Some(implicit_outputs) = split.next() {
        for implicit_output in implicit_outputs.trim().split(" ") {
            target_implicit_outputs.push(PathBuf::from(implicit_output.trim()));
        }
    }
    Ok((target_outputs, target_implicit_outputs))
}

fn parse_input_and_rule_section(section: &str) -> Result<(String, Vec<PathBuf>), String> {
    let mut split = section.trim().split(" ");
    if split.clone().count() < 1 {
        return error!("parse_input_and_rule_section failed: '{section}'");
    }
    let target_rule = String::from(split.nth(0).unwrap());
    let mut target_inputs = Vec::new();
    for input in split {
        target_inputs.push(PathBuf::from(input.trim()));
    }
    Ok((target_rule, target_inputs))
}

fn parse_input_and_deps_section(
    section: &str,
) -> Result<(String, Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut split = section.split("|");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!("parse_input_and_deps_section failed: '{section}'");
    }

    let (target_rule, target_inputs) = parse_input_and_rule_section(split.nth(0).unwrap())?;

    let mut target_implicit_dependencies = Vec::new();
    if let Some(implicit_dependencies) = split.next() {
        for implicit_dep in implicit_dependencies.trim().split(" ") {
            target_implicit_dependencies.push(PathBuf::from(implicit_dep.trim()));
        }
    }
    Ok((target_rule, target_inputs, target_implicit_dependencies))
}

fn parse_input_section(
    section: &str,
) -> Result<(String, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut split = section.split("||");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!("parse_input_section failed: '{section}'");
    }

    let (target_rule, target_inputs, target_implicit_dependencies) =
        parse_input_and_deps_section(split.nth(0).unwrap())?;

    let mut target_order_only_dependencies = Vec::new();
    if let Some(order_only_dependencies) = split.next() {
        for dep in order_only_dependencies.trim().split(" ") {
            target_order_only_dependencies.push(PathBuf::from(dep.trim()));
        }
    }
    Ok((
        target_rule,
        target_inputs,
        target_implicit_dependencies,
        target_order_only_dependencies,
    ))
}

fn find_column_index(line: &str) -> Option<usize> {
    let Some(index) = line.find(":") else {
        return None;
    };
    if line.as_bytes()[0..index].ends_with("$".as_bytes()) {
        if let Some(sub_index) =
            find_column_index(std::str::from_utf8(&line.as_bytes()[index + 1..]).unwrap())
        {
            return Some(index + 1 + sub_index);
        } else {
            return None;
        }
    }
    Some(index)
}

fn split_output_and_input_sections(line: &str) -> Result<(&str, &str), String> {
    let Some(index) = find_column_index(line) else {
        return error!("split_output_and_input_sections failed: '{line}'");
    };
    Ok((
        std::str::from_utf8(&line.as_bytes()[0..index]).unwrap(),
        std::str::from_utf8(&line.as_bytes()[index + 1..]).unwrap(),
    ))
}

fn parse_key_value(line: &str) -> Result<(String, String), String> {
    let Some(split) = line.split_once("=") else {
        return error!("parse_key_value failed: '{line}'");
    };
    Ok((String::from(split.0.trim()), String::from(split.1.trim())))
}

fn parse_build_target<G>(
    line: &str,
    lines: &mut std::str::Lines<'_>,
) -> Result<NinjaTarget<G>, String>
where
    G: GeneratorTarget,
{
    let Some(line_stripped) = line.strip_prefix("build") else {
        return error!("parse_build_target failed: '{line}'");
    };

    let (output_section, input_section) = split_output_and_input_sections(line_stripped)?;

    let (target_outputs, target_implicit_outputs) = parse_output_section(output_section)?;
    let (target_rule, target_inputs, target_implicit_dependencies, target_order_only_dependencies) =
        parse_input_section(input_section)?;

    let mut target_variables: HashMap<String, String> = HashMap::new();
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with(INDENT) {
            break;
        }
        let (key, value) = parse_key_value(next_line)?;
        target_variables.insert(key, value);
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

fn skip_ninja_rule(lines: &mut std::str::Lines<'_>) {
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with(INDENT) {
            return;
        }
    }
}

fn parse_subninja_file<G>(line: &str, dir_path: &Path) -> Result<Vec<NinjaTarget<G>>, String>
where
    G: GeneratorTarget,
{
    let mut split = line.split(" ");
    let split_count = split.clone().count();
    if split_count != 2 {
        return error!("parse_subninja_file failed: '{line}'");
    }
    parse_ninja_file(dir_path.join(split.nth(1).unwrap()))
}

fn parse_ninja_file<'a, G>(file_path: PathBuf) -> Result<Vec<NinjaTarget<G>>, String>
where
    G: GeneratorTarget,
{
    let mut all_targets = Vec::new();
    let mut targets = Vec::new();
    let mut globals = HashMap::new();

    let dir_path = file_path.parent().unwrap();
    let file = read_file(&file_path)?.replace("$\n", " ");
    let mut lines = file.lines();
    while let Some(line) = lines.next() {
        if line.is_empty()
            || line.starts_with("default ")
            || line.starts_with("pool ")
            || line.starts_with("#")
        {
            continue;
        } else if line.starts_with("build ") {
            targets.push(parse_build_target(line, &mut lines)?);
        } else if line.starts_with("rule ") {
            skip_ninja_rule(&mut lines);
        } else if line.starts_with("subninja ") || line.starts_with("include ") {
            let subtargets = parse_subninja_file(line, dir_path)?;
            all_targets.extend(subtargets);
        } else {
            let (key, value) = parse_key_value(line)?;
            globals.insert(key, value);
        }
    }
    for target in &mut targets {
        target.set_globals(globals.clone());
    }
    all_targets.extend(targets);

    Ok(all_targets)
}

pub fn parse_build_ninja<G>(ninja_file_dir_path: &Path) -> Result<Vec<NinjaTarget<G>>, String>
where
    G: GeneratorTarget,
{
    let file_path = ninja_file_dir_path.join("build.ninja");
    parse_ninja_file(file_path)
}
