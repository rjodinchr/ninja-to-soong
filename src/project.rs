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
    pub fn android_path(self, android_path: &Path) -> PathBuf {
        android_path.join("external").join(self.str())
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
    ) -> Vec<PathBuf> {
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
    pub const fn str(self) -> &'static str {
        match self {
            Self::SpirvHeadersFiles => "spirv-header-dep",
            Self::TargetsToGenerate => "target-dep",
            Self::ClangHeaders => "clang-dep",
            Self::LibclcBinaries => "libclc-dep",
        }
    }
}

pub type GenDepsMap = HashMap<GenDeps, HashSet<PathBuf>>;
pub type ProjectsMap<'a> = HashMap<ProjectId, &'a dyn Project>;

pub trait Project {
    fn init(&mut self, android_path: &Path, ndk_path: &Path, temp_path: &Path);
    fn get_id(&self) -> ProjectId;
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String>;

    fn get_ninja_file_path(
        &mut self,
        _projects_map: &ProjectsMap,
    ) -> Result<Option<PathBuf>, String> {
        Ok(None)
    }
    fn get_cmd_output(&self, output: &Path) -> PathBuf {
        output.to_path_buf()
    }
    fn get_default_cflags(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn get_deps_info(&self) -> Vec<(PathBuf, GenDeps)> {
        Vec::new()
    }
    fn get_gen_deps(&self, _project: ProjectId) -> GenDepsMap {
        HashMap::new()
    }
    fn get_include(&self, include: &Path) -> PathBuf {
        include.to_path_buf()
    }
    fn get_library_name(&self, library: &Path) -> PathBuf {
        library.to_path_buf()
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
    fn ignore_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn ignore_include(&self, _include: &Path) -> bool {
        false
    }
    fn ignore_link_flag(&self, _flag: &str) -> bool {
        false
    }
    fn ignore_target(&self, _target: &Path) -> bool {
        false
    }
    fn optimize_target_for_size(&self, _target: &str) -> bool {
        false
    }
}
