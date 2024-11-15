use std::collections::HashMap;
use std::collections::HashSet;

use crate::target::BuildTarget;

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
    ndk_root: &str,
    build_root: &str,
) -> Result<(String, Vec<String>), String> {
    let mut targets: Vec<String> = Vec::new();
    let mut result = String::new();
    result += name;
    result += " {\n";

    result += "\tname: \"";
    result += &crate::utils::rework_target_name(crate::target::get_name(target));
    result += "\",\n";

    let mut includes: HashSet<String> = HashSet::new();
    let mut defines: HashSet<String> = HashSet::new();
    result += "\tsrcs: [\n";
    for input in crate::target::get_inputs(target) {
        result += "\t\t\"";
        let (src, src_includes, src_defines) =
            match crate::target::get_compiler_target_info(input, targets_map, source_root) {
                Ok(return_values) => return_values,
                Err(err) => return Err(err),
            };
        for inc in src_includes {
            if inc.starts_with(build_root) {
                continue;
            }
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
    result += &print_hashset(
        crate::target::get_link_flags(target, source_root),
        "ldflags",
        |result, element| result.push_str(element),
    );

    let (static_libs, shared_libs, system_shared_libs, mut deps) =
        match crate::target::get_link_libraries(target, ndk_root) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };
    targets.append(&mut deps);
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

    let generated_headers = match crate::target::get_generated_headers(target, &targets_map) {
        Ok(generated_headers) => generated_headers,
        Err(err) => return Err(err),
    };
    result += &print_hashset(generated_headers, "generated_headers", |result, element| {
        result.push_str(element)
    });

    result += "}\n\n";
    return Ok((result, targets));
}

fn generate_genrule(
    target: &BuildTarget,
    command: String,
    source_root: &str,
    build_root: &str,
) -> Result<String, String> {
    let mut result = String::new();
    result += "cc_genrule {\n";

    result += "\tname: \"";
    result += &crate::utils::rework_target_name(crate::target::get_name(target));
    result += "\",\n";

    let inputs = crate::target::get_inputs(target);
    let mut filtered_inputs: Vec<&String> = Vec::new();
    let mut tools: Vec<&String> = Vec::new();
    let mut tool_files: Vec<&String> = Vec::new();
    for input in inputs {
        if input.contains("NATIVE/bin") {
            tools.push(input);
        } else if input.ends_with(".py") {
            tool_files.push(input);
        } else {
            filtered_inputs.push(input);
        }
    }
    let outputs = crate::target::get_outputs(target);
    let command = &match crate::utils::rework_command(
        command,
        &mut filtered_inputs,
        outputs,
        source_root,
        build_root,
    ) {
        Ok(command) => command,
        Err(err) => return Err(err),
    };

    if filtered_inputs.len() > 0 {
        result += "\tsrcs: [\n";
        for input in filtered_inputs {
            result += "\t\t\"";
            result += &crate::utils::rework_source_path(input, source_root);
            result += "\",\n";
        }
        result += "\t],\n";
    }

    result += "\tout: [\n";
    for output in outputs {
        result += "\t\t\"";
        result += &crate::utils::rework_output_path(output);
        result += "\",\n";
    }
    result += "\t],\n";

    if tools.len() > 1 {
        return Err(format!(
            "Unsupported tools '{tools:#?}' for target: {target:#?}"
        ));
    }
    if tools.len() == 1 {
        result += "\ttools: [\n";
        for tool in tools {
            result += "\t\t\"";
            result += &crate::utils::rework_target_name(tool);
            result += "\",\n";
        }
        result += "\t],\n";
    }

    if tool_files.len() > 1 {
        return Err(format!(
            "Unsupported tool_files '{tool_files:#?}' for target: {target:#?}"
        ));
    }
    if tool_files.len() == 1 {
        result += "\ttool_files: [\n";
        for tool in tool_files {
            result += "\t\t\"";
            result += &crate::utils::rework_source_path(tool, source_root);
            result += "\",\n";
        }
        result += "\t],\n";
    }

    result += "\tcmd: \"";
    result += command;
    result += "\",\n";

    result += "}\n\n";
    return Ok(result);
}

pub fn generate_android_bp(
    entry_targets: Vec<String>,
    targets_map: HashMap<String, &BuildTarget>,
    source_root: &str,
    ndk_root: &str,
    build_root: &str,
) -> Result<String, String> {
    let mut result = String::new();
    let mut target_seen: HashSet<String> = HashSet::new();
    let mut target_to_generate = entry_targets;

    while let Some(input) = target_to_generate.pop() {
        println!("target: {input}");
        if target_seen.contains(&input) || input.contains("llvm/bin/") {
            continue;
        }
        let Some(target) = targets_map.get(&input) else {
            continue;
        };

        if crate::target::get_rule(target).starts_with("CXX_SHARED_LIBRARY") {
            let (generated_target, mut next_targets) = match generate_object(
                "cc_library_shared",
                target,
                &targets_map,
                source_root,
                ndk_root,
                build_root,
            ) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
            result += &generated_target;
            target_to_generate.append(&mut next_targets);
            target_to_generate.append(&mut crate::target::get_all_inputs(target));
            for input in crate::target::get_inputs(target) {
                target_seen.insert(input.clone());
            }
        } else if crate::target::get_rule(target).starts_with("CXX_STATIC_LIBRARY") {
            let (generated_target, mut next_targets) = match generate_object(
                "cc_library_static",
                target,
                &targets_map,
                source_root,
                ndk_root,
                build_root,
            ) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
            result += &generated_target;
            target_to_generate.append(&mut next_targets);
            target_to_generate.append(&mut crate::target::get_all_inputs(target));
            for input in crate::target::get_inputs(target) {
                target_seen.insert(input.clone());
            }
        } else if crate::target::get_rule(target).starts_with("CXX_EXECUTABLE") {
            let (generated_target, mut next_targets) =
                match generate_object("cc_binary", target, &targets_map, source_root, ndk_root, build_root)
                {
                    Ok(return_value) => return_value,
                    Err(err) => return Err(err),
                };
            result += &generated_target;
            target_to_generate.append(&mut next_targets);
            target_to_generate.append(&mut crate::target::get_all_inputs(target));
            for input in crate::target::get_inputs(target) {
                target_seen.insert(input.clone());
            }
        } else if crate::target::get_rule(target) == "phony" {
            target_to_generate.append(&mut crate::target::get_all_inputs(target));
        } else if crate::target::get_rule(target) == "CUSTOM_COMMAND" {
            let command = match crate::target::get_command(target) {
                Ok(option) => match option {
                    Some(command) => command,
                    None => continue,
                },
                Err(err) => return Err(err),
            };
            result += &match generate_genrule(target, command, source_root, build_root) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
        } else if crate::target::get_rule(target).starts_with("CXX_COMPILER")
            || crate::target::get_rule(target).starts_with("C_COMPILER")
        {
            continue;
        } else {
            return Err(format!("unsupported target: {target:#?}"));
        }
        for output in crate::target::get_outputs(target) {
            target_seen.insert(output.clone());
        }
        for output in crate::target::get_implicit_outputs(target) {
            target_seen.insert(output.clone());
        }
    }
    return Ok(result);
}
