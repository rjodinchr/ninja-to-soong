// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0;

use crate::soong_module::*;
use crate::soong_package::*;
use crate::utils::*;

use std::collections::HashMap;

const ALL_CPU_TARGETS: &str = "N2S_ALL_CPUS";

pub struct SoongPackageMerger {
    packages: Vec<(String, SoongPackage)>,
    merged_package: SoongPackage,
}

impl SoongPackageMerger {
    pub fn new(
        inputs: Vec<(&'static str, Result<SoongPackage, String>)>,
        merged_package: SoongPackage,
    ) -> Result<Self, String> {
        let mut packages = Vec::new();
        for (target_cpu, package) in inputs {
            packages.push((String::from(target_cpu), package?));
        }
        Ok(Self {
            packages,
            merged_package,
        })
    }

    fn merge_props_bool(
        prop_name: &str,
        props: Vec<(String, SoongNamedProp)>,
    ) -> Result<Vec<(String, SoongNamedProp)>, String> {
        let mut booleans = Vec::new();
        for (_, prop) in &props {
            match prop.get_prop() {
                SoongProp::Bool(bool) => booleans.push(bool),
                SoongProp::None => return Ok(props),
                _ => return error!("unexpected prop"),
            }
        }
        let Some(boolean) = booleans.pop() else {
            return Ok(props);
        };
        if booleans.into_iter().all(|bool| bool == boolean) {
            return Ok(vec![(
                String::from(ALL_CPU_TARGETS),
                SoongNamedProp::new(prop_name, SoongProp::Bool(boolean)),
            )]);
        }
        Ok(props)
    }

    fn merge_props_str(
        prop_name: &str,
        props: Vec<(String, SoongNamedProp)>,
    ) -> Result<Vec<(String, SoongNamedProp)>, String> {
        let mut strings = Vec::new();
        for (_, prop) in &props {
            match prop.get_prop() {
                SoongProp::Str(str) => strings.push(str),
                SoongProp::None => return Ok(props),
                _ => return error!("unexpected prop"),
            }
        }
        let Some(string) = strings.pop() else {
            return Ok(props);
        };
        if strings.into_iter().all(|str| str == string) {
            return Ok(vec![(
                String::from(ALL_CPU_TARGETS),
                SoongNamedProp::new(prop_name, SoongProp::Str(string)),
            )]);
        }
        Ok(props)
    }

    fn merge_props_vec_str(
        prop_name: &str,
        props: Vec<(String, SoongNamedProp)>,
    ) -> Result<Vec<(String, SoongNamedProp)>, String> {
        let mut inputs = Vec::new();
        let mut outputs = HashMap::new();
        let mut all_strings = Vec::new();
        outputs.insert(ALL_CPU_TARGETS, Vec::<String>::new());
        for (target_cpu, prop) in &props {
            outputs.insert(target_cpu, Vec::new());
            match prop.get_prop() {
                SoongProp::VecStr(vec) => {
                    all_strings.extend(vec.clone());
                    inputs.push((target_cpu, vec));
                }
                SoongProp::None => inputs.push((target_cpu, Vec::new())),
                _ => return error!("unexpected prop"),
            }
        }
        all_strings.sort_unstable();
        all_strings.dedup();
        for string in all_strings {
            if inputs.iter().all(|(_, vec)| vec.contains(&string)) {
                outputs
                    .get_mut(ALL_CPU_TARGETS)
                    .unwrap()
                    .push(string.clone());
            } else {
                for (target_cpu, vec) in &inputs {
                    if vec.contains(&string) {
                        outputs
                            .get_mut(target_cpu.as_str())
                            .unwrap()
                            .push(string.clone());
                    }
                }
            }
        }
        Ok(outputs
            .into_iter()
            .map(|(target_cpu, vec)| {
                (
                    String::from(target_cpu),
                    SoongNamedProp::new(prop_name, SoongProp::VecStr(vec)),
                )
            })
            .collect())
    }

    fn merge_props(
        prop_name: &str,
        props: Vec<(String, SoongNamedProp)>,
    ) -> Result<Vec<(String, SoongNamedProp)>, String> {
        for idx in 0..props.len() {
            let (_, prop) = &props[idx];
            match prop.get_prop() {
                SoongProp::Bool(_) => return Self::merge_props_bool(prop_name, props),
                SoongProp::Str(_) => return Self::merge_props_str(prop_name, props),
                SoongProp::VecStr(_) => return Self::merge_props_vec_str(prop_name, props),
                SoongProp::Prop(_) => return error!("prop not supported"),
                SoongProp::None => continue,
            }
        }
        error!("unexpected error")
    }

    fn merge_modules(
        module_name: &str,
        mut modules: Vec<(String, SoongModule)>,
    ) -> Result<SoongModule, String> {
        let mut map = HashMap::new();
        for module_idx in 0..modules.len() {
            let props_name = modules[module_idx].1.get_props_name();
            for prop_name in props_name {
                let mut props = Vec::new();
                for (target_cpu, module) in &mut modules {
                    props.push((
                        target_cpu.clone(),
                        if let Some(prop) = module.pop_prop(&prop_name) {
                            prop
                        } else {
                            SoongNamedProp::new(&prop_name, SoongProp::None)
                        },
                    ));
                }
                for (target_cpu, prop) in Self::merge_props(&prop_name, props)? {
                    let mut vec = match map.remove(&target_cpu) {
                        Some(vec) => vec,
                        None => Vec::new(),
                    };
                    vec.push(prop);
                    map.insert(target_cpu, vec);
                }
            }
        }
        let mut module = SoongModule::new(module_name);
        if let Some(all_cpu_props) = map.remove(ALL_CPU_TARGETS) {
            for prop in all_cpu_props {
                module = module.add_named_prop(prop);
            }
        }
        Ok(module.add_prop(
            "arch",
            SoongProp::Prop(Box::new(
                modules
                    .into_iter()
                    .map(|(target_cpu, _)| {
                        SoongNamedProp::new(
                            &target_cpu,
                            SoongProp::Prop(Box::new(map.remove(&target_cpu).unwrap())),
                        )
                    })
                    .collect(),
            )),
        ))
    }

    pub fn merge(mut self) -> Result<SoongPackage, String> {
        let mut modules_name = Vec::new();
        for (_, package) in &self.packages {
            modules_name.extend(package.get_modules_name());
        }
        modules_name.sort_unstable();
        modules_name.dedup();
        for module_name in modules_name {
            let mut modules = Vec::new();
            let mut module_type = String::new();
            for (target_cpu, package) in &mut self.packages {
                let Some(module) = package.pop_module(&module_name) else {
                    return error!("Could not find module in package");
                };
                module_type = module.get_name();
                modules.push((target_cpu.clone(), module));
            }
            self.merged_package = self
                .merged_package
                .add_module(Self::merge_modules(&module_type, modules)?);
        }
        Ok(self.merged_package)
    }
}
