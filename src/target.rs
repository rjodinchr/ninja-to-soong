use std::collections::HashMap;
use std::collections::HashSet;

use crate::utils::error;

#[derive(Debug)]
pub struct BuildTarget {
    rule: String,
    outputs: Vec<String>,
    implicit_outputs: Vec<String>,
    inputs: Vec<String>,
    implicit_dependencies: Vec<String>,
    order_only_dependencies: Vec<String>,
    variables: HashMap<String, String>,
}

fn get_defines(target: &BuildTarget) -> HashSet<String> {
    let mut defines: HashSet<String> = HashSet::new();

    if let Some(defs) = target.variables.get("DEFINES") {
        for def in defs.trim().split("-D") {
            defines.insert(def.replace(" ", ""));
        }
    };
    defines.remove("");
    return defines;
}

fn get_includes(target: &BuildTarget, source_root: &str, build_root: &str) -> HashSet<String> {
    let mut includes: HashSet<String> = HashSet::new();
    if let Some(incs) = target.variables.get("INCLUDES") {
        for inc in incs.split(" ") {
            if inc.contains(build_root) {
                continue;
            }
            if let Some(stripped_inc) = inc.strip_prefix("-I") {
                includes.insert(crate::utils::rework_source_path(stripped_inc, source_root));
            }
        }
    }
    includes.remove("");
    return includes;
}

pub fn create(
    rule: String,
    outputs: Vec<String>,
    implicit_outputs: Vec<String>,
    inputs: Vec<String>,
    implicit_dependencies: Vec<String>,
    order_only_dependencies: Vec<String>,
    variables: HashMap<String, String>,
) -> BuildTarget {
    BuildTarget {
        rule,
        outputs,
        implicit_outputs,
        inputs,
        implicit_dependencies,
        order_only_dependencies,
        variables,
    }
}

pub fn get_inputs(target: &BuildTarget) -> &Vec<String> {
    &target.inputs
}

pub fn get_rule(target: &BuildTarget) -> &String {
    &target.rule
}

pub fn get_outputs(target: &BuildTarget) -> &Vec<String> {
    &target.outputs
}

pub fn get_implicit_outputs(target: &BuildTarget) -> &Vec<String> {
    &target.implicit_outputs
}

pub fn get_name(target: &BuildTarget) -> &String {
    return &target.outputs[0];
}

pub fn get_all_inputs(target: &BuildTarget) -> Vec<String> {
    let mut inputs: Vec<String> = Vec::new();
    for input in &target.inputs {
        inputs.push(input.clone());
    }
    for input in &target.implicit_dependencies {
        inputs.push(input.clone());
    }
    for input in &target.order_only_dependencies {
        inputs.push(input.clone());
    }
    return inputs;
}

pub fn create_map(targets: &Vec<BuildTarget>) -> HashMap<String, &BuildTarget> {
    let mut map: HashMap<String, &BuildTarget> = HashMap::new();
    for target in targets {
        for output in &target.outputs {
            map.insert(output.clone(), target);
        }
        for output in &target.implicit_outputs {
            map.insert(output.clone(), target);
        }
    }

    return map;
}

pub fn get_link_flags(
    target: &BuildTarget,
    source_root: &str,
) -> (Option<String>, HashSet<String>) {
    let mut link_flags: HashSet<String> = HashSet::new();
    let mut version_script: Option<String> = None;
    if let Some(flags) = target.variables.get("LINK_FLAGS") {
        for flag in flags.split(" ") {
            if flag.contains("-Bsymbolic") {
                link_flags.insert(flag.replace(source_root, ""));
            } else if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                version_script = Some(crate::utils::rework_source_path(vs, source_root));
            }
        }
    }
    link_flags.remove("");
    return (version_script, link_flags);
}

pub fn get_link_libraries(
    target: &BuildTarget,
    native_lib_root: &str,
    prefix: &str,
) -> Result<
    (
        HashSet<String>,
        HashSet<String>,
        HashSet<String>,
        Vec<String>,
    ),
    String,
> {
    let mut static_libraries: HashSet<String> = HashSet::new();
    let mut shared_libraries: HashSet<String> = HashSet::new();
    let mut system_shared_libraries: HashSet<String> = HashSet::new();
    let mut deps: Vec<String> = Vec::new();
    if let Some(libs) = target.variables.get("LINK_LIBRARIES") {
        for lib in libs.split(" ") {
            if lib.strip_prefix("-Wl").is_some() || lib == "-lrt" || lib == "-pthread" || lib == "-latomic" {
                continue;
            } else if let Some(stripped_lib) = lib.strip_prefix("-l") {
                system_shared_libraries.insert("lib".to_string() + stripped_lib);
            } else if lib.starts_with(native_lib_root) {
                let new_lib = lib.split("/").last().unwrap();
                if let Some(new_lib_stripped) = new_lib.strip_suffix(".a") {
                    static_libraries.insert(String::from(new_lib_stripped));
                } else if let Some(new_lib_stripped) = new_lib.strip_suffix(".so") {
                    shared_libraries.insert(String::from(new_lib_stripped));
                }
            } else {
                let lib_name = crate::utils::rework_target_name(lib, prefix);
                if lib != "" {
                    deps.push(String::from(lib));
                }
                if lib.ends_with(".a") {
                    static_libraries.insert(lib_name);
                } else if lib.ends_with(".so") {
                    shared_libraries.insert(lib_name);
                } else if lib == "" {
                    continue;
                } else {
                    return error!(format!(
                        "unsupported library '{lib}' from target: {target:#?}"
                    ));
                }
            }
        }
    }
    static_libraries.remove("");
    shared_libraries.remove("");
    system_shared_libraries.remove("");
    return Ok((
        static_libraries,
        shared_libraries,
        system_shared_libraries,
        deps,
    ));
}

pub fn get_compiler_target_info(
    compiler_target_name: &String,
    targets_map: &HashMap<String, &BuildTarget>,
    source_root: &str,
    build_root: &str,
) -> Result<(String, HashSet<String>, HashSet<String>), String> {
    let Some(input_target) = targets_map.get(compiler_target_name) else {
        return error!(format!(
            "unsupported input for library: {compiler_target_name}"
        ));
    };
    let mut defines_no_assembly: HashSet<String> = HashSet::new();
    if input_target.rule.starts_with("ASM_COMPILER") {
        defines_no_assembly.insert("BLAKE3_NO_AVX512".to_string());
        defines_no_assembly.insert("BLAKE3_NO_AVX2".to_string());
        defines_no_assembly.insert("BLAKE3_NO_SSE41".to_string());
        defines_no_assembly.insert("BLAKE3_NO_SSE2".to_string());
    } else if !input_target.rule.starts_with("CXX_COMPILER")
        && !input_target.rule.starts_with("C_COMPILER")
    {
        return error!(format!(
            "unsupported input target for library: {input_target:#?}"
        ));
    }
    if input_target.inputs.len() != 1 {
        return error!(format!(
            "Too many inputs in CXX_COMPILER input target for library: {input_target:#?}"
        ));
    }
    let mut defines = get_defines(input_target);
    for def in defines_no_assembly {
        defines.insert(def);
    }
    let includes = get_includes(input_target, source_root, build_root);
    let src = crate::utils::rework_source_path(&input_target.inputs[0], source_root);
    return Ok((src, includes, defines));
}

pub const COPY_TARGET: &str = "cmake_copy_if_different";

pub fn get_command(target: &BuildTarget) -> Result<Option<String>, String> {
    if target.rule != "CUSTOM_COMMAND" {
        return error!(format!(
            "Can only look for command in CUSTOM_COMMAND: {target:#?}"
        ));
    }
    let Some(command) = target.variables.get("COMMAND") else {
        return error!(format!("No command in CUSTOM_COMMAND: {target:#?}"));
    };
    if command.contains("cmake -E copy_if_different") {
        return Ok(Some(COPY_TARGET.to_string()));
    } else if command.contains("/usr/bin/cmake") {
        return Ok(None);
    }
    return Ok(Some(command.clone()));
}

pub fn get_generated_headers(
    root_target: &BuildTarget,
    targets_map: &HashMap<String, &BuildTarget>,
    prefix: &str,
) -> Result<HashSet<String>, String> {
    let mut generated_headers: HashSet<String> = HashSet::new();
    let mut target_seen: HashSet<String> = HashSet::new();
    let mut target_to_parse = vec![get_name(root_target).clone()];

    while let Some(target_name) = target_to_parse.pop() {
        if target_seen.contains(&target_name) {
            continue;
        }
        let Some(target) = targets_map.get(&target_name) else {
            continue;
        };
        for output in &target.outputs {
            target_seen.insert(output.clone());
        }
        for output in &target.implicit_outputs {
            target_seen.insert(output.clone());
        }
        if target.rule == "CUSTOM_COMMAND" {
            match get_command(target) {
                Ok(option) => match option {
                    Some(_) => {
                        generated_headers
                            .insert(crate::utils::rework_target_name(&get_name(target), prefix));
                    }
                    None => continue,
                },
                Err(err) => return Err(err),
            }
        } else {
            target_to_parse.append(&mut get_all_inputs(target));
        }
    }

    return Ok(generated_headers);
}
