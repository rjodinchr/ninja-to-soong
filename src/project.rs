// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::context::*;
use crate::ninja_target;
use crate::parser::parse_build_ninja;
use crate::soong_module::*;
use crate::soong_package::*;
use crate::utils::*;

mod angle;
mod clspv;
mod clvk;
mod llvm_project;
mod mesa;
mod spirv_headers;
mod spirv_tools;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ProjectId {
    Angle,
    Clvk,
    Clspv,
    LlvmProject,
    Mesa,
    SpirvHeaders,
    SpirvTools,
}
pub struct ProjectsMap(HashMap<ProjectId, Box<dyn Project>>);
impl ProjectsMap {
    pub fn new() -> Self {
        let projects: Vec<Box<dyn Project>> = vec![
            Box::new(angle::Angle::default()),
            Box::new(clvk::Clvk::default()),
            Box::new(clspv::Clspv::default()),
            Box::new(llvm_project::LlvmProject::default()),
            Box::new(mesa::Mesa::default()),
            Box::new(spirv_headers::SpirvHeaders::default()),
            Box::new(spirv_tools::SpirvTools::default()),
        ];
        Self(
            projects
                .into_iter()
                .fold(HashMap::new(), |mut map, project| {
                    map.insert(project.get_id(), project);
                    map
                }),
        )
    }
    pub fn insert(&mut self, id: ProjectId, project: Box<dyn Project>) {
        self.0.insert(id, project);
    }
    pub fn remove_entry(
        &mut self,
        id: &ProjectId,
    ) -> Result<(ProjectId, Box<dyn Project>), String> {
        let Some(entry) = self.0.remove_entry(id) else {
            return error!("'{id:#?}' not found in projects map");
        };
        Ok(entry)
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, ProjectId, Box<dyn Project>> {
        self.0.iter()
    }
    pub fn get_deps(
        &self,
        from: ProjectId,
        to: ProjectId,
        gen_deps: GenDeps,
    ) -> Result<Vec<PathBuf>, String> {
        let Some(project) = self.0.get(&from) else {
            return error!("'{from:#?}' not found in projects map");
        };
        let deps_map = project.get_deps_map(to);
        let Some(gen_deps) = deps_map.get(&gen_deps) else {
            return error!("'{gen_deps:#?}' not found in deps map");
        };
        let mut gen_deps = gen_deps.clone();
        gen_deps.sort();
        Ok(gen_deps)
    }
    pub fn get_android_path(&self, id: ProjectId, ctx: &Context) -> Result<PathBuf, String> {
        let Some(project) = self.0.get(&id) else {
            return error!("'{id:#?}' not found in projects map");
        };
        Ok(project.get_android_path(ctx))
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum GenDeps {
    SpirvHeaders,
    TargetsToGen,
    ClangHeaders,
    LibclcBins,
}
impl GenDeps {
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

pub trait Project {
    // MANDATORY FUNCTIONS
    fn get_id(&self) -> ProjectId;
    fn get_name(&self) -> &'static str;
    fn get_android_path(&self, ctx: &Context) -> PathBuf;
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
