// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::str;

use crate::ninja_target::*;
use crate::utils::*;

fn parse_output_section(section: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut split = section.split("|");
    if split.clone().count() < 1 {
        return error!("parse_output_section failed: '{section}'");
    }
    let split_outputs = split.nth(0).unwrap().trim().split(" ");
    let outputs = split_outputs
        .map(|output| PathBuf::from(output.trim()))
        .collect();
    let mut implicit_outputs = Vec::new();
    if let Some(implicit_outs) = split.next() {
        for implicit_output in implicit_outs.trim().split(" ") {
            implicit_outputs.push(PathBuf::from(implicit_output.trim()));
        }
    }
    Ok((outputs, implicit_outputs))
}

fn parse_input_and_rule_section(section: &str) -> Result<(String, Vec<PathBuf>), String> {
    let mut split = section.trim().split(" ");
    if split.clone().count() < 1 {
        return error!("parse_input_and_rule_section failed: '{section}'");
    }
    let rule = String::from(split.nth(0).unwrap());
    let inputs = split
        .map(|input| PathBuf::from(input.trim()))
        .collect::<Vec<PathBuf>>();
    Ok((rule, inputs))
}

fn parse_input_and_deps_section(
    section: &str,
) -> Result<(String, Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut split = section.split("|");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!("parse_input_and_deps_section failed: '{section}'");
    }

    let (rule, inputs) = parse_input_and_rule_section(split.nth(0).unwrap())?;

    let mut implicit_deps = Vec::new();
    if let Some(implicit_dependencies) = split.next() {
        for implicit_dep in implicit_dependencies.trim().split(" ") {
            implicit_deps.push(PathBuf::from(implicit_dep.trim()));
        }
    }
    Ok((rule, inputs, implicit_deps))
}

fn parse_input_section(
    section: &str,
) -> Result<(String, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut split = section.split("||");
    let split_count = split.clone().count();
    if split_count != 1 && split_count != 2 {
        return error!("parse_input_section failed: '{section}'");
    }

    let (rule, inputs, implicit_deps) = parse_input_and_deps_section(split.nth(0).unwrap())?;

    let mut order_only_deps = Vec::new();
    if let Some(order_only_dependencies) = split.next() {
        for dep in order_only_dependencies.trim().split(" ") {
            order_only_deps.push(PathBuf::from(dep.trim()));
        }
    }
    Ok((rule, inputs, implicit_deps, order_only_deps))
}

fn find_column_index(line: &str) -> Option<usize> {
    let Some(index) = line.find(":") else {
        return None;
    };
    if line.as_bytes()[0..index].ends_with("$".as_bytes()) {
        if let Some(sub_index) =
            find_column_index(str::from_utf8(&line.as_bytes()[index + 1..]).unwrap())
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
        str::from_utf8(&line.as_bytes()[0..index]).unwrap(),
        str::from_utf8(&line.as_bytes()[index + 1..]).unwrap(),
    ))
}

fn parse_key_value(line: &str) -> Result<(String, String), String> {
    let Some(split) = line.split_once("=") else {
        return error!("parse_key_value failed: '{line}'");
    };
    Ok((String::from(split.0.trim()), String::from(split.1.trim())))
}

fn parse_build_target<T>(line: &str, mut lines: str::Lines) -> Result<T, String>
where
    T: NinjaTarget,
{
    let Some(line_stripped) = line.strip_prefix("build ") else {
        return error!("parse_build_target failed: '{line}'");
    };

    let (output_section, input_section) = split_output_and_input_sections(line_stripped)?;

    let (outputs, implicit_outputs) = parse_output_section(output_section)?;
    let (rule, inputs, implicit_deps, order_only_deps) = parse_input_section(input_section)?;

    let mut variables: HashMap<String, String> = HashMap::new();
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with(" ") {
            break;
        }
        let (key, value) = parse_key_value(next_line)?;
        variables.insert(key, value);
    }

    Ok(T::new(
        rule,
        outputs,
        implicit_outputs,
        inputs,
        implicit_deps,
        order_only_deps,
        variables,
    ))
}

fn parse_ninja_rule(line: &str, mut lines: str::Lines) -> Result<(String, NinjaRuleCmd), String> {
    let Some(rule) = line.strip_prefix("rule ") else {
        return error!("parse_ninja_rule failed: '{line}'");
    };
    let mut command = None;
    let mut rspfile = None;
    let mut rspfile_content = None;
    while let Some(next_line) = lines.next() {
        if !next_line.starts_with(" ") {
            break;
        }
        let (key, value) = parse_key_value(next_line)?;
        match key.as_str() {
            "command" => command = Some(value),
            "rspfile" => rspfile = Some(value),
            "rspfile_content" => rspfile_content = Some(value),
            _ => (),
        }
    }
    if command.is_none() {
        return error!("parse_ninja_rule failed");
    }
    return Ok((
        String::from(rule),
        NinjaRuleCmd {
            command: command.unwrap(),
            rsp_info: if rspfile.is_some() && rspfile_content.is_some() {
                Some((rspfile.unwrap(), rspfile_content.unwrap()))
            } else {
                None
            },
        },
    ));
}

fn parse_subninja_file<T>(line: &str, dir_path: &Path) -> Result<(Vec<T>, NinjaRulesMap), String>
where
    T: NinjaTarget,
{
    let mut split = line.split(" ");
    let split_count = split.clone().count();
    if split_count != 2 {
        return error!("parse_subninja_file failed: '{line}'");
    }
    parse_ninja_file(dir_path.join(split.nth(1).unwrap()), dir_path)
}

fn parse_ninja_file<T>(
    file_path: PathBuf,
    build_path: &Path,
) -> Result<(Vec<T>, NinjaRulesMap), String>
where
    T: NinjaTarget,
{
    let mut all_targets = Vec::new();
    let mut targets: Vec<T> = Vec::new();
    let mut globals = HashMap::new();
    let mut all_rules = HashMap::new();

    let file = read_file(&file_path)?.replace("$\n", " ");

    let mut lines = file.lines();
    while let Some(line) = lines.next() {
        if line.is_empty()
            || line.starts_with("default ")
            || line.starts_with("pool ")
            || line.starts_with("#")
            || line.starts_with(" ")
        {
            continue;
        } else if line.starts_with("rule ") {
            let (rule, rule_command) = parse_ninja_rule(line, lines.clone())?;
            all_rules.insert(rule, rule_command);
        } else if line.starts_with("build ") {
            targets.push(parse_build_target(line, lines.clone())?);
        } else if line.starts_with("subninja ") || line.starts_with("include ") {
            let (subtargets, rules) = parse_subninja_file(line, build_path)?;
            all_targets.extend(subtargets);
            all_rules.extend(rules);
        } else {
            let (key, value) = parse_key_value(line)?;
            globals.insert(key, value);
        }
    }
    for target in &mut targets {
        target.set_globals(globals.clone());
    }
    all_targets.extend(targets);

    Ok((all_targets, all_rules))
}

pub fn parse_build_ninja<T>(build_path: &Path) -> Result<Vec<T>, String>
where
    T: NinjaTarget,
{
    let file_path = build_path.join("build.ninja");
    let (mut targets, rules) = parse_ninja_file::<T>(file_path, build_path)?;
    for target in &mut targets {
        target.set_rule(&rules);
    }
    Ok(targets)
}
