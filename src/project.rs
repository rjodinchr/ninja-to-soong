// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::soong_package::SoongPackage;
use crate::utils::*;

pub mod clspv;
pub mod clvk;
pub mod llvm;
pub mod spirv_headers;
pub mod spirv_tools;

pub trait Project<'a> {
    // MANDATORY
    fn get_id(&self) -> ProjectId;
    fn get_build_directory(
        &mut self,
        dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<String, String>;
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<SoongPackage, String>;

    // OPTIONAL
    fn get_deps(&self) -> Vec<ProjectId> {
        Vec::new()
    }
    fn get_generated_build_directory(&self) -> String {
        String::new()
    }
    fn get_generated_deps(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn parse_custom_command_inputs(
        &self,
        _: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        internal_error!()
    }
    fn get_default_cflags(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn get_headers_to_copy(&self, _headers: &HashSet<String>) -> HashSet<String> {
        HashSet::new()
    }
    fn get_headers_to_generate(&self, _headers: &HashSet<String>) -> HashSet<String> {
        HashSet::new()
    }
    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        HashSet::new()
    }
    fn get_target_stem(&self, _target: &String) -> String {
        String::new()
    }
    fn get_library_name(&self, library: &str) -> String {
        library.to_string()
    }
    fn optimize_target_for_size(&self, _target: &String) -> bool {
        false
    }
    fn ignore_target(&self, _target: &String) -> bool {
        false
    }
    fn ignore_include(&self, _include: &str) -> bool {
        false
    }
    fn ignore_define(&self, _define: &str) -> bool {
        false
    }
    fn rework_include(&self, include: &str) -> String {
        include.to_string()
    }
    fn rework_command_output(&self, output: &str) -> String {
        output.to_string()
    }
    fn handle_link_flag(&self, _flag: &str, _link_flags: &mut HashSet<String>) {}
}
