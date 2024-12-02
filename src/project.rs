// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::ninja_target::NinjaTarget;
use crate::soong_package::SoongPackage;
use crate::utils::*;

pub mod clspv;
pub mod clvk;
pub mod llvm_project;
pub mod spirv_headers;
pub mod spirv_tools;

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ProjectId {
    Clvk,
    Clspv,
    LlvmProject,
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
            CLVK_NAME => Some(ProjectId::Clvk),
            CLSPV_NAME => Some(ProjectId::Clspv),
            LLVM_NAME => Some(ProjectId::LlvmProject),
            SPIRV_HEADERS_NAME => Some(ProjectId::SpirvHeaders),
            SPIRV_TOOLS_NAME => Some(ProjectId::SpirvTools),
            _ => None,
        }
    }
    pub const fn str(&self) -> &'static str {
        match self {
            ProjectId::All => ALL_NAME,
            ProjectId::Clvk => CLVK_NAME,
            ProjectId::Clspv => CLSPV_NAME,
            ProjectId::LlvmProject => LLVM_NAME,
            ProjectId::SpirvHeaders => SPIRV_HEADERS_NAME,
            ProjectId::SpirvTools => SPIRV_TOOLS_NAME,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum GenDeps {
    SpirvHeadersFiles,
    TargetsToGenerate,
    ClangHeaders,
    LibclcBinaries,
}
impl GenDeps {
    fn get(
        self,
        project: &dyn Project,
        from: ProjectId,
        projects_map: &ProjectsMap,
    ) -> Vec<String> {
        let mut vec = Vec::from_iter(
            projects_map
                .get(&from)
                .unwrap()
                .get_gen_deps(project.get_id())
                .get(&self)
                .unwrap()
                .clone(),
        );
        vec.sort();
        vec
    }
}

pub type GenDepsMap = HashMap<GenDeps, HashSet<String>>;
pub type ProjectsMap<'a> = HashMap<ProjectId, &'a dyn Project<'a>>;
pub type CmdInputAndDeps = (HashSet<String>, HashSet<(String, String)>);

pub trait Project<'a> {
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String>;
    fn get_id(&self) -> ProjectId;

    fn get_build_dir(&mut self, _projects_map: &ProjectsMap) -> Result<Option<String>, String> {
        Ok(None)
    }
    fn get_cmd_inputs_and_deps(
        &self,
        target_inputs: &Vec<String>,
    ) -> Result<CmdInputAndDeps, String> {
        Ok((HashSet::from_iter(target_inputs.clone()), HashSet::new()))
    }
    fn get_cmd_output(&self, output: &str) -> String {
        output.to_string()
    }
    fn get_default_cflags(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn get_gen_deps(&self, _project: ProjectId) -> GenDepsMap {
        HashMap::new()
    }
    fn get_include(&self, include: &str) -> String {
        include.to_string()
    }
    fn get_library_name(&self, library: &str) -> String {
        library.to_string()
    }
    fn get_project_deps(&self) -> Vec<ProjectId> {
        Vec::new()
    }
    fn get_target_header_libs(&self, _target: &str) -> HashSet<String> {
        HashSet::new()
    }
    fn get_target_alias(&self, _target: &str) -> String {
        String::new()
    }
    fn ignore_defines(&self) -> bool {
        false
    }
    fn ignore_gen_header(&self, _header: &str) -> bool {
        false
    }
    fn ignore_include(&self, _include: &str) -> bool {
        false
    }
    fn ignore_target(&self, _target: &str) -> bool {
        false
    }
    fn optimize_target_for_size(&self, _target: &str) -> bool {
        false
    }
    fn update_link_flags(&self, _flag: &str, _link_flags: &mut HashSet<String>) {}
}
