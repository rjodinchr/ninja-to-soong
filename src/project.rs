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

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ProjectId {
    CLVK,
    CLSPV,
    LLVM,
    SpirvHeaders,
    SpirvTools,
    All,
}

const ALL_NAME: &str = "all";
const CLVK_NAME: &str = "clvk";
const CLSPV_NAME: &str = "clspv";
const LLVM_NAME: &str = "llvm-project";
const SPIRV_HEADERS_NAME: &str = "SPIRV-Headers";
const SPIRV_TOOLS_NAME: &str = "SPIRV-Tools";

impl ProjectId {
    pub fn from(str: &str) -> Option<ProjectId> {
        match str {
            ALL_NAME => Some(ProjectId::All),
            CLVK_NAME => Some(ProjectId::CLVK),
            CLSPV_NAME => Some(ProjectId::CLSPV),
            LLVM_NAME => Some(ProjectId::LLVM),
            SPIRV_HEADERS_NAME => Some(ProjectId::SpirvHeaders),
            SPIRV_TOOLS_NAME => Some(ProjectId::SpirvTools),
            _ => None,
        }
    }
    pub const fn str(&self) -> &'static str {
        match self {
            ProjectId::All => ALL_NAME,
            ProjectId::CLVK => CLVK_NAME,
            ProjectId::CLSPV => CLSPV_NAME,
            ProjectId::LLVM => LLVM_NAME,
            ProjectId::SpirvHeaders => SPIRV_HEADERS_NAME,
            ProjectId::SpirvTools => SPIRV_TOOLS_NAME,
        }
    }
}

pub type ProjectDeps = HashMap<Dependency, HashSet<String>>;
pub type ProjectMap<'a> = HashMap<ProjectId, &'a dyn Project<'a>>;
pub type CommandInputsAndDeps = (HashSet<String>, HashSet<(String, String)>);

fn get_dependency(
    project: &dyn Project,
    from: ProjectId,
    dependency: Dependency,
    project_map: &ProjectMap,
) -> Vec<String> {
    let mut vec = Vec::from_iter(
        project_map
            .get(&from)
            .unwrap()
            .get_generated_deps(project.get_id())
            .get(&dependency)
            .unwrap()
            .clone(),
    );
    vec.sort();
    vec
}

pub trait Project<'a> {
    // MANDATORY
    fn get_id(&self) -> ProjectId;

    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        project_map: &ProjectMap,
    ) -> Result<SoongPackage, String>;

    // OPTIONAL
    fn get_build_directory(&mut self, _project_map: &ProjectMap) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn get_command_inputs_and_deps(
        &self,
        target_inputs: &Vec<String>,
    ) -> Result<CommandInputsAndDeps, String> {
        Ok((HashSet::from_iter(target_inputs.clone()), HashSet::new()))
    }
    fn get_command_output(&self, output: &str) -> String {
        output.to_string()
    }
    fn get_include(&self, include: &str) -> String {
        include.to_string()
    }
    fn get_project_dependencies(&self) -> Vec<ProjectId> {
        Vec::new()
    }
    fn get_generated_build_directory(&self) -> String {
        String::new()
    }
    fn get_generated_deps(&self, _project: ProjectId) -> ProjectDeps {
        HashMap::new()
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
    fn handle_link_flag(&self, _flag: &str, _link_flags: &mut HashSet<String>) {}
}
