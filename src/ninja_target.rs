// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::utils::*;

pub mod cmake;
pub use cmake::*;

#[derive(PartialEq, Eq)]
pub enum NinjaRule {
    StaticLibrary,
    SharedLibrary,
    CustomCommand,
}

pub trait GeneratorTarget {
    fn new(
        rule: String,
        outputs: Vec<PathBuf>,
        implicit_outputs: Vec<PathBuf>,
        inputs: Vec<PathBuf>,
        implicit_deps: Vec<PathBuf>,
        order_only_deps: Vec<PathBuf>,
        variables: HashMap<String, String>,
    ) -> Self;
    fn set_globals(&mut self, globals: HashMap<String, String>);
    fn get_rule(&self) -> Option<NinjaRule>;
    fn get_all_inputs(&self) -> Vec<PathBuf>;
    fn get_inputs(&self) -> &Vec<PathBuf>;
    fn get_all_outputs(&self) -> Vec<PathBuf>;
    fn get_outputs(&self) -> &Vec<PathBuf>;
    fn get_name(&self, prefix: &Path) -> String;
    fn get_link_flags(&self) -> (Option<PathBuf>, HashSet<String>);
    fn get_link_libraries(&self) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String>;
    fn get_defines(&self) -> HashSet<String>;
    fn get_includes(&self) -> HashSet<PathBuf>;
    fn get_cmd(&self) -> Result<Option<String>, String>;
}

pub struct NinjaTarget<G>
where
    G: GeneratorTarget,
{
    target: G,
}

impl<'a, G> NinjaTarget<G>
where
    G: GeneratorTarget,
{
    pub fn new(
        rule: String,
        outputs: Vec<PathBuf>,
        implicit_outputs: Vec<PathBuf>,
        inputs: Vec<PathBuf>,
        implicit_deps: Vec<PathBuf>,
        order_only_deps: Vec<PathBuf>,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            target: G::new(
                rule,
                outputs,
                implicit_outputs,
                inputs,
                implicit_deps,
                order_only_deps,
                variables,
            ),
        }
    }
    pub fn set_globals(&mut self, globals: HashMap<String, String>) {
        self.target.set_globals(globals);
    }
    pub fn get_rule(&self) -> Option<NinjaRule> {
        self.target.get_rule()
    }
    pub fn get_all_inputs(&self) -> Vec<PathBuf> {
        self.target.get_all_inputs()
    }
    pub fn get_inputs(&self) -> &Vec<PathBuf> {
        self.target.get_inputs()
    }
    pub fn get_all_outputs(&self) -> Vec<PathBuf> {
        self.target.get_all_outputs()
    }
    pub fn get_outputs(&self) -> &Vec<PathBuf> {
        self.target.get_outputs()
    }
    pub fn get_name(&self, prefix: &Path) -> String {
        self.target.get_name(prefix)
    }
    pub fn get_link_flags(&self) -> (Option<PathBuf>, HashSet<String>) {
        self.target.get_link_flags()
    }
    pub fn get_link_libraries(&self) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String> {
        self.target.get_link_libraries()
    }
    pub fn get_defines(&self) -> HashSet<String> {
        self.target.get_defines()
    }
    pub fn get_includes(&self) -> HashSet<PathBuf> {
        self.target.get_includes()
    }
    pub fn get_cmd(&self) -> Result<Option<String>, String> {
        self.target.get_cmd()
    }
}

pub struct NinjaTargetsMap<'a, Gen>(HashMap<PathBuf, &'a NinjaTarget<Gen>>)
where
    Gen: GeneratorTarget;

impl<'a, Gen> NinjaTargetsMap<'a, Gen>
where
    Gen: GeneratorTarget,
{
    pub fn new(targets: &'a Vec<NinjaTarget<Gen>>) -> Self {
        let mut map = HashMap::new();
        for target in targets {
            for output in target.get_all_outputs() {
                map.insert(output, target);
            }
        }
        Self(map)
    }
    pub fn get(&self, key: &Path) -> Option<&&NinjaTarget<Gen>> {
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
        F: FnMut(
            &mut Iterator,
            Option<NinjaRule>,
            PathBuf,
            &NinjaTarget<Gen>,
        ) -> Result<(), String>,
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
            targets.extend(target.get_all_inputs());
            targets_seen.extend(target.get_all_outputs());
            f(&mut iterator, target.get_rule(), target_name, target)?;
        }
        Ok(iterator)
    }
}
