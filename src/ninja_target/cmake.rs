// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Debug)]
pub struct CmakeNinjaTarget(NinjaTargetCommon);

impl NinjaTarget for CmakeNinjaTarget {
    fn new(common: NinjaTargetCommon) -> Self {
        Self(common)
    }
    fn get_common(&self) -> &NinjaTargetCommon {
        &self.0
    }

    fn get_rule(&self) -> Result<NinjaRule, String> {
        Ok(if self.0.rule.starts_with("CXX_SHARED_LIBRARY") {
            NinjaRule::SharedLibrary
        } else if self.0.rule.starts_with("CXX_STATIC_LIBRARY") {
            NinjaRule::StaticLibrary
        } else if self.0.rule.starts_with("CUSTOM_COMMAND") {
            let Some(command) = self.0.variables.get("COMMAND") else {
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
            if command.contains("bin/cmake ") {
                NinjaRule::None
            } else {
                NinjaRule::CustomCommand(NinjaRuleCmd {
                    command: String::from(command),
                    rsp_info: None,
                })
            }
        } else if self.0.rule.starts_with("CXX_EXECUTABLE") {
            NinjaRule::Binary
        } else {
            NinjaRule::None
        })
    }
    fn get_sources(&self, build_path: &Path) -> Result<Vec<PathBuf>, String> {
        if self.0.inputs.len() != 1 {
            return error!("Too many inputs in: {self:#?}");
        }
        Ok(common::get_sources(&self.0.inputs, build_path))
    }
    fn get_libs_static_whole(&self) -> Vec<PathBuf> {
        Vec::new()
    }
    fn get_libs_static(&self) -> Vec<PathBuf> {
        let Some(libs) = self.0.variables.get("LINK_LIBRARIES") else {
            return Vec::new();
        };
        common::get_libs_static(libs)
    }
    fn get_libs_shared(&self) -> Vec<PathBuf> {
        let Some(libs) = self.0.variables.get("LINK_LIBRARIES") else {
            return Vec::new();
        };
        common::get_libs_shared(libs)
    }
    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>) {
        let Some(flags) = self.0.variables.get("LINK_FLAGS") else {
            return (None, Vec::new());
        };
        common::get_link_flags(flags)
    }
    fn get_defines(&self) -> Vec<String> {
        let Some(defs) = self.0.variables.get("DEFINES") else {
            return Vec::new();
        };
        common::get_defines(defs)
    }
    fn get_includes(&self, build_path: &Path) -> Vec<PathBuf> {
        let Some(incs) = self.0.variables.get("INCLUDES") else {
            return Vec::new();
        };
        common::get_includes(incs, build_path)
    }
    fn get_cflags(&self) -> Vec<String> {
        let Some(flags) = self.0.variables.get("FLAGS") else {
            return Vec::new();
        };
        common::get_cflags(flags)
    }
}
