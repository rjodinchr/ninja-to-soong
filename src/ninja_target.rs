// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::parser::parse_build_ninja;
use crate::utils::*;

pub mod cmake;
pub mod common;
pub mod gn;

pub type NinjaRuleCmd = (String, Option<(String, String)>);
pub type NinjaRulesMap = HashMap<String, NinjaRuleCmd>;

pub enum NinjaRule {
    Binary,
    StaticLibrary,
    SharedLibrary,
    CustomCommand,
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
    fn get_name(&self, prefix: &Path) -> String {
        path_to_id(prefix.join(&self.get_outputs()[0]))
    }
    fn get_indent() -> &'static str {
        "  "
    }

    fn set_globals(&mut self, globals: HashMap<String, String>);
    fn set_rule(&mut self, rules: &NinjaRulesMap);
    fn get_rule(&self) -> Option<NinjaRule>;
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
    fn get_cmd(&self) -> Result<Option<NinjaRuleCmd>, String>;
}

#[derive(Debug)]
pub struct NinjaTargetsMap<'a, T>(HashMap<PathBuf, &'a T>)
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
                map.insert(output.clone(), target);
            }
            for output in target.get_implicit_ouputs() {
                map.insert(output.clone(), target);
            }
        }
        Self(map)
    }
    pub fn get(&self, key: &Path) -> Option<&&T> {
        self.0.get(key)
    }
    pub fn traverse_from<Iterator, F, G>(
        &self,
        mut targets: Vec<PathBuf>,
        mut iterator: Iterator,
        mut f: F,
        ignore_target: G,
    ) -> Result<Iterator, String>
    where
        F: FnMut(&mut Iterator, NinjaRule, &T) -> Result<(), String>,
        G: Fn(&Path) -> bool,
    {
        let mut targets_seen = HashSet::new();
        while let Some(target_name) = targets.pop() {
            if targets_seen.contains(&target_name) || ignore_target(&target_name) {
                continue;
            }
            let Some(target) = self.get(&target_name) else {
                continue;
            };
            targets.extend(target.get_inputs().clone());
            targets.extend(target.get_implicit_deps().clone());
            targets.extend(target.get_order_only_deps().clone());
            targets_seen.extend(target.get_outputs().clone());
            targets_seen.extend(target.get_implicit_ouputs().clone());
            if let Some(rule) = target.get_rule() {
                f(&mut iterator, rule, target)?;
            }
        }
        Ok(iterator)
    }
}
