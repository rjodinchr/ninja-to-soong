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

    fn set_rule(&mut self, _rules: &NinjaRulesMap) {}

    fn get_rule(&self) -> Option<NinjaRule> {
        Some(if self.rule.starts_with("CXX_SHARED_LIBRARY") {
            NinjaRule::SharedLibrary
        } else if self.rule.starts_with("CXX_STATIC_LIBRARY") {
            NinjaRule::StaticLibrary
        } else if self.rule.starts_with("CUSTOM_COMMAND") {
            NinjaRule::CustomCommand
        } else if self.rule.starts_with("CXX_EXECUTABLE") {
            NinjaRule::Binary
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

    fn get_sources(&self, build_path: &Path) -> Result<Vec<PathBuf>, String> {
        if self.inputs.len() != 1 {
            return error!("Too many inputs in: {self:#?}");
        }
        Ok(common::get_sources(&self.inputs, build_path))
    }

    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        let Some(flags) = self.variables.get("LINK_FLAGS") else {
            return (None, Vec::new());
        };
        common::get_link_flags(flags)
    }

    fn get_link_libraries(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
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

    fn get_includes(&self, build_path: &Path) -> Vec<PathBuf> {
        let Some(incs) = self.variables.get("INCLUDES") else {
            return Vec::new();
        };
        common::get_includes(incs, build_path)
    }

    fn get_cflags(&self) -> Vec<String> {
        let Some(flags) = self.variables.get("FLAGS") else {
            return Vec::new();
        };
        common::get_cflags(flags)
    }

    fn get_cmd(&self) -> Result<Option<NinjaRuleCmd>, String> {
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
            Some(NinjaRuleCmd {
                command: String::from(command),
                rsp_info: None,
            })
        })
    }
}
