use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;
use crate::target::BuildTarget;

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
    ) -> Result<String, String>;
    fn generate_static_library(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String>;
    fn generate_executable(
        &self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
    ) -> Result<String, String>;
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
    ) -> Result<String, String>;

    fn generate_rule(
        &self,
        rule: &str,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        prefix: &str,
        host: bool,
        input_ref_for_genrule: &String,
    ) -> Result<Option<String>, String> {
        let result = if rule.starts_with("CXX_SHARED_LIBRARY") {
            self.generate_shared_library(
                target,
                &targets_map,
                source_root,
                native_lib_root,
                build_root,
                prefix,
                host,
            )
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            self.generate_static_library(
                target,
                targets_map,
                source_root,
                native_lib_root,
                build_root,
                prefix,
                host,
            )
        } else if rule.starts_with("CXX_EXECUTABLE") {
            self.generate_executable(
                target,
                targets_map,
                source_root,
                native_lib_root,
                build_root,
                prefix,
                host,
            )
        } else if rule == "CUSTOM_COMMAND" {
            let command = match target.get_command() {
                Ok(option) => match option {
                    Some(command) => command,
                    None => return Ok(None),
                },
                Err(err) => return Err(err),
            };
            if command == crate::target::COPY_TARGET {
                self.generate_copy(target, source_root, prefix, host)
            } else {
                self.generate_custom_command(
                    target,
                    command,
                    source_root,
                    build_root,
                    prefix,
                    input_ref_for_genrule,
                    host,
                )
            }
        } else if rule.starts_with("CMAKE_SYMLINK") {
            self.generate_cmake_link(target, source_root, prefix)
        } else if rule.starts_with("CXX_COMPILER")
            || rule.starts_with("C_COMPILER")
            || rule.starts_with("ASM_COMPILER")
            || rule == "phony"
        {
            return Ok(None);
        } else {
            error!(format!("unsupported target: {target:#?}"))
        };
        return match result {
            Ok(return_value) => Ok(Some(return_value)),
            Err(err) => Err(err),
        };
    }
}

fn create_map(targets: &Vec<BuildTarget>) -> HashMap<String, &BuildTarget> {
    let mut map: HashMap<String, &BuildTarget> = HashMap::new();
    for target in targets {
        for output in &target.get_all_outputs() {
            map.insert(output.clone(), target);
        }
    }

    return map;
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
    let targets_map = create_map(targets);

    while let Some(input) = target_to_generate.pop() {
        println!("target: {prefix}{input}");
        if target_seen.contains(&input) || input.contains("llvm/bin/") {
            continue;
        }
        let Some(target) = targets_map.get(&input) else {
            continue;
        };

        target_to_generate.append(&mut target.get_all_inputs());
        for output in target.get_all_outputs() {
            target_seen.insert(output);
        }

        result += &match generator.generate_rule(
            target.get_rule(),
            target,
            &targets_map,
            source_root,
            native_lib_root,
            build_root,
            prefix,
            host,
            input_ref_for_genrule,
        ) {
            Ok(option) => match option {
                Some(return_value) => return_value,
                None => continue,
            },
            Err(err) => return Err(err),
        };
    }
    return Ok(result);
}
