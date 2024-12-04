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
}

const CLVK_NAME: &str = "clvk";
const CLSPV_NAME: &str = "clspv";
const LLVM_PROJECT_NAME: &str = "llvm-project";
const SPIRV_HEADERS_NAME: &str = "SPIRV-Headers";
const SPIRV_TOOLS_NAME: &str = "SPIRV-Tools";

impl ProjectId {
    pub fn from(project: &str) -> Result<Self, String> {
        Ok(match project {
            CLVK_NAME => Self::Clvk,
            CLSPV_NAME => Self::Clspv,
            LLVM_PROJECT_NAME => Self::LlvmProject,
            SPIRV_HEADERS_NAME => Self::SpirvHeaders,
            SPIRV_TOOLS_NAME => Self::SpirvTools,
            _ => return error!("Unknown project '{project}'"),
        })
    }
    pub const fn str(self) -> &'static str {
        match self {
            Self::Clvk => CLVK_NAME,
            Self::Clspv => CLSPV_NAME,
            Self::LlvmProject => LLVM_PROJECT_NAME,
            Self::SpirvHeaders => SPIRV_HEADERS_NAME,
            Self::SpirvTools => SPIRV_TOOLS_NAME,
        }
    }
    pub fn android_path(self, android_dir: &str) -> String {
        android_dir.to_string() + "/external/" + self.str()
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
pub type ProjectsMap<'a> = HashMap<ProjectId, &'a dyn Project>;
pub type CmdInputAndDeps = (HashSet<String>, HashSet<(String, String)>);

pub trait Project {
    fn init(&mut self, android_dir: &str, ndk_dir: &str, temp_dir: &str);
    fn get_id(&self) -> ProjectId;
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String>;

    fn get_ninja_file_path(
        &mut self,
        _projects_map: &ProjectsMap,
    ) -> Result<Option<String>, String> {
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
    fn get_target_alias(&self, _target: &str) -> Option<String> {
        None
    }
    fn ignore_define(&self, _define: &str) -> bool {
        false
    }
    fn ignore_gen_header(&self, _header: &str) -> bool {
        false
    }
    fn ignore_include(&self, _include: &str) -> bool {
        false
    }
    fn ignore_link_flag(&self, _flag: &str) -> bool {
        false
    }
    fn ignore_target(&self, _target: &str) -> bool {
        false
    }
    fn optimize_target_for_size(&self, _target: &str) -> bool {
        false
    }
}
