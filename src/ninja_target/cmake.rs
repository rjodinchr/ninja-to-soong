// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use super::*;

#[derive(Debug)]
pub struct CmakeNinjaTarget {
    rule: String,
    outputs: Vec<PathBuf>,
    implicit_outputs: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    implicit_deps: Vec<PathBuf>,
    order_only_deps: Vec<PathBuf>,
    variables: HashMap<String, String>,
}

impl NinjaTarget for CmakeNinjaTarget {
    fn new(
        rule: String,
        outputs: Vec<PathBuf>,
        implicit_outputs: Vec<PathBuf>,
        inputs: Vec<PathBuf>,
        implicit_deps: Vec<PathBuf>,
        order_only_deps: Vec<PathBuf>,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            rule,
            outputs,
            implicit_outputs,
            inputs,
            implicit_deps,
            order_only_deps,
            variables,
        }
    }

    fn set_globals(&mut self, _globals: HashMap<String, String>) {}

    fn set_rule(&mut self, _rules: &HashMap<String, String>) {}

    fn get_rule(&self) -> Option<NinjaRule> {
        Some(if self.rule.starts_with("CXX_SHARED_LIBRARY") {
            NinjaRule::SharedLibrary
        } else if self.rule.starts_with("CXX_STATIC_LIBRARY") {
            NinjaRule::StaticLibrary
        } else if self.rule.starts_with("CUSTOM_COMMAND") {
            NinjaRule::CustomCommand
        } else {
            return None;
        })
    }

    fn get_inputs(&self) -> &Vec<PathBuf> {
        &self.inputs
    }

    fn get_implicit_deps(&self) -> &Vec<PathBuf> {
        &self.implicit_deps
    }

    fn get_order_only_deps(&self) -> &Vec<PathBuf> {
        &self.order_only_deps
    }

    fn get_outputs(&self) -> &Vec<PathBuf> {
        &self.outputs
    }

    fn get_implicit_ouputs(&self) -> &Vec<PathBuf> {
        &self.implicit_outputs
    }

    fn get_sources(&self) -> Result<Vec<PathBuf>, String> {
        if self.inputs.len() != 1 {
            return error!("Too many inputs in: {self:#?}");
        }
        Ok(self.inputs.clone())
    }

    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        let Some(flags) = self.variables.get("LINK_FLAGS") else {
            return (None, Vec::new());
        };
        common::get_link_flags(flags)
    }

    fn get_link_libraries(&self, _prefix: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
        let Some(libs) = self.variables.get("LINK_LIBRARIES") else {
            return Ok((Vec::new(), Vec::new()));
        };
        common::get_link_libraries(libs)
    }

    fn get_defines(&self) -> Vec<String> {
        let Some(defs) = self.variables.get("DEFINES") else {
            return Vec::new();
        };
        common::get_defines(defs)
    }

    fn get_includes(&self) -> Vec<PathBuf> {
        let Some(incs) = self.variables.get("INCLUDES") else {
            return Vec::new();
        };
        common::get_includes(incs)
    }

    fn get_cflags(&self) -> Vec<String> {
        let Some(flags) = self.variables.get("FLAGS") else {
            return Vec::new();
        };
        common::get_cflags(flags)
    }

    fn get_cmd(&self) -> Result<Option<String>, String> {
        let Some(command) = self.variables.get("COMMAND") else {
            return error!("No command in: {self:#?}");
        };
        let mut split = command.split(" && ");
        let split_count = split.clone().count();
        if split_count < 2 {
            return error!(
                "Could not find enough split in command (expected at least 2, got {split_count}"
            );
        }
        let command = split.nth(1).unwrap();
        Ok(if command.contains("bin/cmake ") {
            None
        } else {
            Some(command.to_string())
        })
    }
}

pub fn cmake_configure(
    src_path: &Path,
    build_path: &Path,
    ndk_path: &Path,
    args: Vec<&str>,
) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_CONFIGURE").is_ok() {
        return Ok(false);
    }
    let mut command = std::process::Command::new("cmake");
    command
        .args([
            "-B",
            &path_to_string(build_path),
            "-S",
            &path_to_string(src_path),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            &("-DCMAKE_TOOLCHAIN_FILE=".to_string()
                + &path_to_string(ndk_path.join("build/cmake/android.toolchain.cmake"))),
            &("-DANDROID_ABI=".to_string() + ANDROID_ABI),
            &("-DANDROID_PLATFORM=".to_string() + ANDROID_PLATFORM),
        ])
        .args(args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!("cmake_configure({src_path:#?}) failed: {err}");
    }
    Ok(true)
}

pub fn cmake_build(build_path: &Path, targets: &Vec<PathBuf>) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_BUILD").is_ok() {
        return Ok(false);
    }
    let targets_args = targets.into_iter().fold(Vec::new(), |mut vec, target| {
        vec.push("--target");
        vec.push(target.to_str().unwrap_or_default());
        vec
    });
    let mut command = std::process::Command::new("cmake");
    command
        .args(["--build", &path_to_string(build_path)])
        .args(targets_args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!("cmake_build({build_path:#?}) failed: {err}");
    }
    Ok(true)
}
