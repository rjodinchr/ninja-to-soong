use std::collections::HashMap;
use std::collections::HashSet;

use crate::target::BuildTarget;
use crate::utils::error;

pub mod soong_generator;

pub trait Generator {
    fn generate_shared_library(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<(String, Vec<String>), String>;
    fn generate_static_library(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<(String, Vec<String>), String>;
    fn generate_executable(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<(String, Vec<String>), String>;
    fn generate_custom_command(
        &self,
        target: &BuildTarget,
        command: String,
        source_root: &str,
        build_root: &str,
        prefix: &str,
        input_ref_for_genrule: &String,
        host: bool,
    ) -> Result<String, String>;
    fn generate_copy(
        &self,
        target: &BuildTarget,
        source_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String>;
    fn generate_cmake_link(
        &self,
        target: &BuildTarget,
        source_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String>;
}

pub fn generate(
    generator: &dyn Generator,
    entry_targets: Vec<String>,
    targets: &Vec<BuildTarget>,
    source_root: &str,
    native_lib_root: &str,
    build_root: &str,
    prefix: &str,
    host: bool,
    input_ref_for_genrule: &String,
) -> Result<String, String> {
    let mut result = String::new();
    let mut target_seen: HashSet<String> = HashSet::new();
    let mut target_to_generate = entry_targets;
    let targets_map = crate::target::create_map(targets);

    while let Some(input) = target_to_generate.pop() {
        println!("target: {prefix}{input}");
        if target_seen.contains(&input) || input.contains("llvm/bin/") {
            continue;
        }
        let Some(target) = targets_map.get(&input) else {
            continue;
        };

        let rule = crate::target::get_rule(target);
        if rule.starts_with("CXX_SHARED_LIBRARY") {
            let (generated_target, mut next_targets) = match generator.generate_shared_library(
                target,
                &targets_map,
                source_root,
                native_lib_root,
                build_root,
                prefix,
                host,
            ) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
            result += &generated_target;
            target_to_generate.append(&mut next_targets);
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            let (generated_target, mut next_targets) = match generator.generate_static_library(
                target,
                &targets_map,
                source_root,
                native_lib_root,
                build_root,
                prefix,
                !host,
            ) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
            result += &generated_target;
            target_to_generate.append(&mut next_targets);
        } else if rule.starts_with("CXX_EXECUTABLE") {
            let (generated_target, mut next_targets) = match generator.generate_executable(
                target,
                &targets_map,
                source_root,
                native_lib_root,
                build_root,
                prefix,
                false,
            ) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
            result += &generated_target;
            target_to_generate.append(&mut next_targets);
        } else if rule == "CUSTOM_COMMAND" {
            let command = match crate::target::get_command(target) {
                Ok(option) => match option {
                    Some(command) => command,
                    None => continue,
                },
                Err(err) => return Err(err),
            };
            if command == crate::target::COPY_TARGET {
                result += &match generator.generate_copy(target, source_root, prefix, host) {
                    Ok(return_value) => return_value,
                    Err(err) => return Err(err),
                };
            } else {
                result += &match generator.generate_custom_command(
                    target,
                    command,
                    source_root,
                    build_root,
                    prefix,
                    input_ref_for_genrule,
                    host,
                ) {
                    Ok(return_value) => return_value,
                    Err(err) => return Err(err),
                };
            }
        } else if rule.starts_with("CMAKE_SYMLINK") {
            result += &match generator.generate_cmake_link(target, source_root, prefix, host) {
                Ok(return_value) => return_value,
                Err(err) => return Err(err),
            };
        } else if !(rule.starts_with("CXX_COMPILER")
            || rule.starts_with("C_COMPILER")
            || rule.starts_with("ASM_COMPILER")
            || rule == "phony")
        {
            return error!(format!("unsupported target: {target:#?}"));
        }

        target_to_generate.append(&mut crate::target::get_all_inputs(target));
        for output in crate::target::get_outputs(target) {
            target_seen.insert(output.clone());
        }
        for output in crate::target::get_implicit_outputs(target) {
            target_seen.insert(output.clone());
        }
    }
    return Ok(result);
}
