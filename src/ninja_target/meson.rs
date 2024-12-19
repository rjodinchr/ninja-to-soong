// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use super::*;

#[derive(Debug)]
pub struct MesonNinjaTarget {
    rule: String,
    outputs: Vec<PathBuf>,
    implicit_outputs: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    implicit_deps: Vec<PathBuf>,
    order_only_deps: Vec<PathBuf>,
    variables: HashMap<String, String>,
}

impl NinjaTarget for MesonNinjaTarget {
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
        Some(if self.rule == "c_LINKER" || self.rule == "cpp_LINKER" {
            let (_, link_flags) = self.get_link_flags();
            if link_flags.contains(&String::from("-fPIC")) {
                NinjaRule::SharedLibrary
            } else {
                NinjaRule::Binary
            }
        } else if self.rule == "STATIC_LINKER" {
            NinjaRule::StaticLibrary
        } else if self.rule == "CUSTOM_COMMAND" || self.rule == "CUSTOM_COMMAND_DEP" {
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

    fn get_sources(&self, build_path: &Path) -> Result<Vec<PathBuf>, String> {
        if self.inputs.len() != 1 {
            return error!("Too many inputs in: {self:#?}");
        }
        Ok(common::get_sources(&self.inputs, build_path))
    }

    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        let Some(args) = self.variables.get("LINK_ARGS") else {
            return (None, Vec::new());
        };
        let flags = args
            .split(" ")
            .filter(|arg| {
                arg.starts_with("-Wl")
                    || (!arg.starts_with("-l") && !arg.contains(".a") && !arg.contains(".so"))
            })
            .collect::<Vec<&str>>();
        common::get_link_flags(&flags.join(" "))
    }

    fn get_link_libraries(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
        let Some(args) = self.variables.get("LINK_ARGS") else {
            return Ok((Vec::new(), Vec::new()));
        };
        let flags = args
            .split(" ")
            .filter(|arg| {
                (arg.starts_with("-l") || arg.contains(".a") || arg.contains(".so"))
                    && !arg.starts_with("-Wl")
            })
            .collect::<Vec<&str>>();
        common::get_link_libraries(&flags.join(" "))
    }

    fn get_defines(&self) -> Vec<String> {
        let Some(args) = self.variables.get("ARGS") else {
            return Vec::new();
        };
        let defines = args
            .split(" ")
            .map(|arg| arg.trim_matches('\''))
            .filter(|arg| arg.starts_with("-D"))
            .map(|arg| arg.replace("\"", "\\\""))
            .collect::<Vec<String>>();
        common::get_defines(&defines.join(" "))
    }

    fn get_includes(&self, build_path: &Path) -> Vec<PathBuf> {
        let Some(args) = self.variables.get("ARGS") else {
            return Vec::new();
        };
        let includes = args
            .split(" ")
            .filter(|arg| arg.starts_with("-I"))
            .collect::<Vec<&str>>();
        common::get_includes(&includes.join(" "), build_path)
    }

    fn get_cflags(&self) -> Vec<String> {
        let Some(args) = self.variables.get("ARGS") else {
            return Vec::new();
        };
        let cflags = args
            .split(" ")
            .filter(|arg| !arg.starts_with("-I") && !arg.starts_with("-D"))
            .collect::<Vec<&str>>();
        common::get_cflags(&cflags.join(" "))
    }

    fn get_cmd(&self) -> Result<Option<NinjaRuleCmd>, String> {
        let Some(command) = self.variables.get("COMMAND") else {
            return error!("No command in: {self:#?}");
        };
        Ok(Some(NinjaRuleCmd {
            command: String::from(command.split_once(" -- ").unwrap_or(("", command)).1),
            rsp_info: None,
        }))
    }
}
