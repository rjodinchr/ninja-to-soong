// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::utils::*;

pub mod cmake;
pub mod common;
pub mod gn;
pub mod meson;

pub use cmake::*;
pub use gn::*;
pub use meson::*;

#[derive(Debug, Clone)]
pub struct NinjaRuleCmd {
    pub command: String,
    pub rsp_info: Option<(String, String)>,
}
pub type NinjaRulesMap = HashMap<String, NinjaRuleCmd>;

pub enum NinjaRule {
    Binary,
    StaticLibrary,
    SharedLibrary,
    CustomCommand(NinjaRuleCmd),
    None,
}

pub trait NinjaTarget: std::fmt::Debug {
    fn new(
        rule: String,
        outputs: Vec<PathBuf>,
        implicit_outputs: Vec<PathBuf>,
        inputs: Vec<PathBuf>,
        implicit_deps: Vec<PathBuf>,
        order_only_deps: Vec<PathBuf>,
        variables: HashMap<String, String>,
    ) -> Self;
    fn get_name(&self, prefix: &str) -> PathBuf {
        Path::new(prefix).join(&self.get_outputs()[0])
    }
    fn set_globals(&mut self, _globals: HashMap<String, String>) {}
    fn set_rule(&mut self, _rules: &NinjaRulesMap) {}

    fn get_rule(&self) -> Result<NinjaRule, String>;
    fn get_inputs(&self) -> &Vec<PathBuf>;
    fn get_implicit_deps(&self) -> &Vec<PathBuf>;
    fn get_order_only_deps(&self) -> &Vec<PathBuf>;
    fn get_outputs(&self) -> &Vec<PathBuf>;
    fn get_implicit_ouputs(&self) -> &Vec<PathBuf>;
    fn get_sources(&self, build_path: &Path) -> Result<Vec<PathBuf>, String>;
    fn get_link_flags(&self) -> (Option<PathBuf>, Vec<String>);
    fn get_link_libraries(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String>;
    fn get_defines(&self) -> Vec<String>;
    fn get_includes(&self, build_path: &Path) -> Vec<PathBuf>;
    fn get_cflags(&self) -> Vec<String>;
}

#[derive(Debug)]
pub struct NinjaTargetsMap<'a, T>(HashMap<&'a Path, &'a T>)
where
    T: NinjaTarget;

impl<'a, T> NinjaTargetsMap<'a, T>
where
    T: NinjaTarget,
{
    pub fn new(targets: &'a Vec<T>) -> Self {
        let mut map = HashMap::new();
        for target in targets {
            for output in target.get_outputs() {
                map.insert(output.as_path(), target);
            }
            for output in target.get_implicit_ouputs() {
                map.insert(output.as_path(), target);
            }
        }
        Self(map)
    }
    pub fn get(&self, key: &Path) -> Option<&&T> {
        self.0.get(key)
    }
    pub fn traverse_from<F>(
        &self,
        mut targets: Vec<PathBuf>,
        mut filter_target: F,
    ) -> Result<(), String>
    where
        F: FnMut(&T) -> Result<bool, String>,
    {
        let mut targets_seen = std::collections::HashSet::new();
        while let Some(target_name) = targets.pop() {
            if targets_seen.contains(&target_name) {
                continue;
            }
            targets_seen.insert(target_name.clone());
            let Some(target) = self.get(&target_name) else {
                continue;
            };
            targets_seen.extend(target.get_outputs().clone());
            targets_seen.extend(target.get_implicit_ouputs().clone());
            if !filter_target(target)? {
                continue;
            }
            targets.extend(target.get_inputs().clone());
            targets.extend(target.get_implicit_deps().clone());
            targets.extend(target.get_order_only_deps().clone());
        }
        Ok(())
    }
}
