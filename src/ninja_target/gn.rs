// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use super::*;

const SHARED_LIB: &str = "solink";
const STATIC_LIB: &str = "alink";

#[derive(Debug)]
pub struct GnNinjaTarget {
    rule: String,
    rule_cmd: Option<NinjaRuleCmd>,
    outputs: Vec<PathBuf>,
    implicit_outputs: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    implicit_deps: Vec<PathBuf>,
    order_only_deps: Vec<PathBuf>,
    variables: HashMap<String, String>,
    globals: Option<HashMap<String, String>>,
}

impl NinjaTarget for GnNinjaTarget {
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
            rule_cmd: None,
            outputs,
            implicit_outputs,
            inputs,
            implicit_deps,
            order_only_deps,
            variables,
            globals: None,
        }
    }

    fn set_globals(&mut self, globals: HashMap<String, String>) {
        self.globals = Some(globals);
    }

    fn set_rule(&mut self, rules: &NinjaRulesMap) {
        if let Some(rule) = rules.get(&self.rule) {
            self.rule_cmd = Some(rule.clone());
        }
    }

    fn get_rule(&self) -> Result<NinjaRule, String> {
        Ok(if self.rule == SHARED_LIB {
            NinjaRule::SharedLibrary
        } else if self.rule == STATIC_LIB {
            NinjaRule::StaticLibrary
        } else if self.rule.ends_with("__rule") {
            let Some(command) = self.rule_cmd.clone() else {
                return error!("No command in: {self:#?}");
            };
            NinjaRule::CustomCommand(command)
        } else {
            NinjaRule::None
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
        if !(self.rule == "cxx" || self.rule == "cc" || self.rule == "asm") {
            return Ok(Vec::new());
        }
        Ok(common::get_sources(&self.inputs, build_path))
    }

    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        let Some(libs) = self.variables.get("ldflags") else {
            return (None, Vec::new());
        };
        common::get_link_flags(libs)
    }

    fn get_link_libraries(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
        let mut static_libs = Vec::new();
        let mut shared_libs = Vec::new();
        for lib_key in ["libs", "solibs"] {
            if let Some(libs) = self.variables.get(lib_key) {
                let (static_libraries, shared_libraries) = common::get_link_libraries(libs)?;
                static_libs.extend(static_libraries);
                shared_libs.extend(shared_libraries);
            }
        }

        if self.rule == SHARED_LIB {
            shared_libs.push(self.outputs[0].clone());
        } else if self.rule == STATIC_LIB {
            static_libs.push(self.outputs[0].clone());
        }
        Ok((static_libs, shared_libs))
    }

    fn get_defines(&self) -> Vec<String> {
        if let Some(globals) = &self.globals {
            if let Some(defs) = globals.get("defines") {
                return common::get_defines(defs);
            }
        }
        Vec::new()
    }

    fn get_includes(&self, build_path: &Path) -> Vec<PathBuf> {
        if let Some(globals) = &self.globals {
            if let Some(incs) = globals.get("include_dirs") {
                return common::get_includes(incs, build_path);
            }
        }
        Vec::new()
    }

    fn get_cflags(&self) -> Vec<String> {
        let mut cflags = Vec::new();
        for cflag in ["cflags", "cflags_cc"] {
            if let Some(globals) = &self.globals {
                if let Some(defs) = globals.get(cflag) {
                    cflags.append(&mut common::get_cflags(defs));
                }
            }
        }
        cflags
    }
}
