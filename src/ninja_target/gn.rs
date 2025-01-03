// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

const SHARED_LIB: &str = "solink";
const STATIC_LIB: &str = "alink";

#[derive(Debug)]
pub struct GnNinjaTarget {
    rule_cmd: Option<NinjaRuleCmd>,
    common: NinjaTargetCommon,
    globals: Option<HashMap<String, String>>,
}

impl NinjaTarget for GnNinjaTarget {
    fn new(common: NinjaTargetCommon) -> Self {
        Self {
            rule_cmd: None,
            common,
            globals: None,
        }
    }
    fn get_common(&self) -> &NinjaTargetCommon {
        &self.common
    }

    fn get_rule(&self) -> Result<NinjaRule, String> {
        Ok(if self.common.rule == SHARED_LIB {
            NinjaRule::SharedLibrary
        } else if self.common.rule == STATIC_LIB {
            NinjaRule::StaticLibrary
        } else if self.common.rule.ends_with("__rule") {
            let Some(command) = self.rule_cmd.clone() else {
                return error!("No command in: {self:#?}");
            };
            NinjaRule::CustomCommand(command)
        } else {
            NinjaRule::None
        })
    }
    fn get_sources(&self, build_path: &Path) -> Result<Vec<PathBuf>, String> {
        if !(self.common.rule == "cxx" || self.common.rule == "cc" || self.common.rule == "asm") {
            return Ok(Vec::new());
        }
        Ok(common::get_sources(&self.common.inputs, build_path))
    }
    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        let Some(libs) = self.common.variables.get("ldflags") else {
            return (None, Vec::new());
        };
        common::get_link_flags(libs)
    }
    fn get_link_libraries(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
        let mut static_libs = Vec::new();
        let mut shared_libs = Vec::new();
        for lib_key in ["libs", "solibs"] {
            if let Some(libs) = self.common.variables.get(lib_key) {
                let (static_libraries, shared_libraries) = common::get_link_libraries(libs)?;
                static_libs.extend(static_libraries);
                shared_libs.extend(shared_libraries);
            }
        }

        if self.common.rule == SHARED_LIB {
            shared_libs.push(self.common.outputs[0].clone());
        } else if self.common.rule == STATIC_LIB {
            static_libs.push(self.common.outputs[0].clone());
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

    fn set_globals(&mut self, globals: HashMap<String, String>) {
        self.globals = Some(globals);
    }
    fn set_rule(&mut self, rules: &NinjaRulesMap) {
        if let Some(rule) = rules.get(&self.common.rule) {
            self.rule_cmd = Some(rule.clone());
        }
    }
}
