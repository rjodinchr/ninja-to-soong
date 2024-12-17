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

    fn get_rule(&self) -> Option<NinjaRule> {
        Some(if self.rule == SHARED_LIB {
            NinjaRule::SharedLibrary
        } else if self.rule == STATIC_LIB {
            NinjaRule::StaticLibrary
        } else if self.rule.ends_with("__rule") {
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
        if !(self.rule == "cxx" || self.rule == "cc" || self.rule == "asm") {
            return Ok(Vec::new());
        }
        Ok(common::get_sources(&self.inputs, build_path))
    }

    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        if let Some(libs) = self.variables.get("ldflags") {
            common::get_link_flags(libs)
        } else {
            (None, Vec::new())
        }
    }

    fn get_link_libraries(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
        let (mut static_libs, mut shared_libs) = if let Some(libs) = self.variables.get("libs") {
            common::get_link_libraries(libs)?
        } else {
            (Vec::new(), Vec::new())
        };

        if let Some(libs) = self.variables.get("solibs") {
            let (static_libraries, shared_libraries) = common::get_link_libraries(libs)?;
            static_libs.extend(static_libraries);
            shared_libs.extend(shared_libraries);
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
        for cflag in vec!["cflags", "cflags_cc"] {
            if let Some(globals) = &self.globals {
                if let Some(defs) = globals.get(cflag) {
                    cflags.append(&mut common::get_cflags(defs));
                }
            }
        }
        cflags
    }

    fn get_cmd(&self) -> Result<Option<NinjaRuleCmd>, String> {
        let Some(command) = &self.rule_cmd else {
            return error!("No command in: {self:#?}");
        };
        Ok(Some(command.clone()))
    }
}

fn bash_c(path: &Path, cmd: &Vec<&str>) -> Result<(), String> {
    let path_str = path_to_string(path);
    let mut command = vec!["cd", &path_str, ";"];
    command.append(&mut cmd.clone());
    let bash_arg = command.join(" ");
    execute_cmd!("bash", vec!["-c", &bash_arg], None)
}

fn gn_gen(src_path: &Path, build_path: &Path, gn_args: Vec<&str>) -> Result<(), String> {
    let build_dir = path_to_string(build_path);
    let gn_gen = vec!["gn", "gen", &build_dir];
    bash_c(src_path, &gn_gen)?;

    let target_cpu = "target_cpu=\\\"".to_string() + ANDROID_CPU + "\\\"";
    let mut args_list = vec!["target_os=\\\"android\\\"", &target_cpu];
    args_list.extend(gn_args);
    let args = "--args=\"".to_string() + &args_list.join(" ") + "\"";
    let gn_args = vec![
        "gn",
        "args",
        &build_dir,
        "--list",
        "--overrides-only",
        "--short",
        &args,
    ];
    bash_c(src_path, &gn_args)?;
    bash_c(src_path, &gn_gen)
}

pub fn get_targets(
    src_path: &Path,
    build_path: &Path,
    gn_args: Vec<&str>,
    ctx: &Context,
) -> Result<Vec<GnNinjaTarget>, String> {
    if !ctx.skip_gen_ninja {
        gn_gen(src_path, build_path, gn_args)?;
    }

    parse_build_ninja(build_path)
}
