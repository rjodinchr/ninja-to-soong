// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::context::*;
use crate::ninja_target;
use crate::parser::parse_build_ninja;
use crate::soong_module::*;
use crate::soong_package::*;
use crate::utils::*;

pub mod angle;
pub mod clspv;
pub mod clvk;
pub mod llvm_project;
pub mod mesa;
pub mod spirv_headers;
pub mod spirv_tools;

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ProjectId {
    Angle,
    Clvk,
    Clspv,
    LlvmProject,
    Mesa,
    SpirvHeaders,
    SpirvTools,
}

const ANGLE_NAME: &str = "angle";
const CLVK_NAME: &str = "clvk";
const CLSPV_NAME: &str = "clspv";
const LLVM_PROJECT_NAME: &str = "llvm-project";
const MESA_NAME: &str = "mesa";
const SPIRV_HEADERS_NAME: &str = "SPIRV-Headers";
const SPIRV_TOOLS_NAME: &str = "SPIRV-Tools";

impl ProjectId {
    pub fn from(project: &str) -> Result<Self, String> {
        Ok(match project {
            ANGLE_NAME => Self::Angle,
            CLVK_NAME => Self::Clvk,
            CLSPV_NAME => Self::Clspv,
            LLVM_PROJECT_NAME => Self::LlvmProject,
            MESA_NAME => Self::Mesa,
            SPIRV_HEADERS_NAME => Self::SpirvHeaders,
            SPIRV_TOOLS_NAME => Self::SpirvTools,
            _ => return error!("Unknown project '{project}'"),
        })
    }
    pub const fn str(self) -> &'static str {
        match self {
            Self::Angle => ANGLE_NAME,
            Self::Clvk => CLVK_NAME,
            Self::Clspv => CLSPV_NAME,
            Self::LlvmProject => LLVM_PROJECT_NAME,
            Self::Mesa => MESA_NAME,
            Self::SpirvHeaders => SPIRV_HEADERS_NAME,
            Self::SpirvTools => SPIRV_TOOLS_NAME,
        }
    }
    pub fn android_path(self, ctx: &Context) -> PathBuf {
        ctx.android_path.join("external").join(self.str())
    }
}

#[derive(Eq, PartialEq, Hash)]
pub enum GenDeps {
    SpirvHeaders,
    TargetsToGen,
    ClangHeaders,
    LibclcBins,
}
impl GenDeps {
    fn get(
        self,
        project: &dyn Project,
        from: ProjectId,
        projects_map: &ProjectsMap,
    ) -> Vec<PathBuf> {
        let mut vec = projects_map
            .get(&from)
            .unwrap()
            .get_deps_map(project.get_id())
            .get(&self)
            .unwrap()
            .clone();
        vec.sort();
        vec
    }
    pub const fn str(self) -> &'static str {
        match self {
            Self::SpirvHeaders => "spirv-header-dep",
            Self::TargetsToGen => "target-dep",
            Self::ClangHeaders => "clang-dep",
            Self::LibclcBins => "libclc-dep",
        }
    }
}

pub type GenDepsMap = HashMap<GenDeps, Vec<PathBuf>>;
pub type ProjectsMap<'a> = HashMap<ProjectId, &'a dyn Project>;

pub trait Project {
    // MANDATORY FUNCTIONS
    fn get_id(&self) -> ProjectId;
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String>;
    // DEPENDENCIES FUNCTIONS
    fn get_project_deps(&self) -> Vec<ProjectId> {
        Vec::new()
    }
    fn get_deps_info(&self) -> Vec<(PathBuf, GenDeps)> {
        Vec::new()
    }
    fn get_deps_map(&self, _project: ProjectId) -> GenDepsMap {
        GenDepsMap::new()
    }
    // TARGET FUNCTIONS
    fn get_target_alias(&self, _target: &str) -> Option<String> {
        None
    }
    fn get_target_object_module(&self, _target: &str, module: SoongModule) -> SoongModule {
        module
    }
    fn get_target_cflags(&self, _target: &str) -> Vec<String> {
        Vec::new()
    }
    fn get_target_shared_libs(&self, _target: &str) -> Vec<String> {
        Vec::new()
    }
    fn get_target_header_libs(&self, _target: &str) -> Vec<String> {
        Vec::new()
    }
    // REWORK FUNCTIONS
    fn get_cmd_output(&self, output: &Path) -> PathBuf {
        PathBuf::from(output)
    }
    fn get_define(&self, define: &str) -> String {
        String::from(define)
    }
    fn get_include(&self, include: &Path) -> PathBuf {
        PathBuf::from(include)
    }
    fn get_lib(&self, lib: &Path) -> PathBuf {
        PathBuf::from(lib)
    }
    fn get_source(&self, source: &Path) -> PathBuf {
        PathBuf::from(source)
    }
    // FILTER FUNCTIONS
    fn filter_cflag(&self, _cflag: &str) -> bool {
        true
    }
    fn filter_define(&self, _define: &str) -> bool {
        true
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        true
    }
    fn filter_include(&self, _include: &Path) -> bool {
        true
    }
    fn filter_lib(&self, _lib: &str) -> bool {
        true
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        true
    }
    fn filter_source(&self, _source: &Path) -> bool {
        true
    }
    fn filter_target(&self, _target: &Path) -> bool {
        true
    }
}
