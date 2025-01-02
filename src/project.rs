// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::context::*;
use crate::ninja_parser::*;
use crate::ninja_target::*;
use crate::soong_module::*;
use crate::soong_package::*;
use crate::utils::*;

mod common;

define_ProjectId!(
    (Angle, angle),
    (Clvk, clvk),
    (Clspv, clspv),
    (LlvmProject, llvm_project),
    (Mesa, mesa),
    (SpirvHeaders, spirv_headers),
    (SpirvTools, spirv_tools)
);
impl ProjectId {
    pub fn get_deps(&self) -> Vec<ProjectId> {
        let mut projects = std::collections::HashSet::new();
        for gen_deps in get_deps() {
            let (project_dep, projects_dep) = gen_deps.projects();
            if &project_dep == self {
                projects.extend(projects_dep);
            }
        }
        Vec::from_iter(projects)
    }
    pub fn get_android_path(self, map: &ProjectsMap, ctx: &Context) -> Result<PathBuf, String> {
        Ok(map.get(self)?.get_android_path(ctx))
    }
}

define_Dep!(
    (ClangHeaders, LlvmProject, (Clspv)),
    (ClspvTargets, Clspv, (Clvk)),
    (LibclcBins, LlvmProject, (Clspv)),
    (LlvmProjectTargets, LlvmProject, (Clvk)),
    (SpirvHeaders, SpirvHeaders, (Clspv, SpirvTools)),
    (SpirvToolsTargets, SpirvTools, (Clvk))
);
impl Dep {
    pub fn get_id(self, input: &Path, prefix: &Path, build_path: &Path) -> String {
        path_to_id(Path::new(&format!("{self:#?}")).join(strip_prefix(
            canonicalize_path(input, build_path),
            canonicalize_path(prefix, build_path),
        )))
    }
    pub fn get(self, projects_map: &ProjectsMap) -> Result<Vec<PathBuf>, String> {
        let mut all_deps = Vec::new();
        let projects = self.projects().1;
        for project in projects {
            all_deps.extend(projects_map.get(project)?.get_deps(self));
        }
        all_deps.sort_unstable();
        all_deps.dedup();
        Ok(all_deps)
    }
}

pub struct ProjectsMap(HashMap<ProjectId, Box<dyn Project>>);
impl ProjectsMap {
    pub fn new() -> Self {
        Self(get_projects())
    }
    pub fn insert(&mut self, id: ProjectId, project: Box<dyn Project>) {
        self.0.insert(id, project);
    }
    pub fn remove(&mut self, id: &ProjectId) -> Result<Box<dyn Project>, String> {
        let Some(entry) = self.0.remove(id) else {
            return error!("'{id:#?}' not found in projects map");
        };
        Ok(entry)
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, ProjectId, Box<dyn Project>> {
        self.0.iter()
    }
    pub fn get(&self, id: ProjectId) -> Result<&Box<dyn Project>, String> {
        let Some(project) = self.0.get(&id) else {
            return error!("'{id:#?}' not found in projects map");
        };
        Ok(project)
    }
}

pub trait Project {
    // MANDATORY FUNCTIONS
    fn get_name(&self) -> &'static str;
    fn get_android_path(&self, ctx: &Context) -> PathBuf;
    fn get_test_path(&self, ctx: &Context) -> PathBuf;
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<String, String>;
    // DEPENDENCIES FUNCTIONS
    fn get_deps_prefix(&self) -> Vec<(PathBuf, Dep)> {
        Vec::new()
    }
    fn get_deps(&self, _dep: Dep) -> Vec<PathBuf> {
        Vec::new()
    }
    // TARGET FUNCTIONS
    fn get_target_name(&self, target: &Path) -> PathBuf {
        PathBuf::from(target)
    }
    fn get_target_stem(&self, _target: &Path) -> Option<String> {
        None
    }
    fn get_target_module(&self, _target: &Path, module: SoongModule) -> SoongModule {
        module
    }
    // EXTEND FUNCTIONS
    fn extend_cflags(&self, _target: &Path) -> Vec<String> {
        Vec::new()
    }
    fn extend_shared_libs(&self, _target: &Path) -> Vec<String> {
        Vec::new()
    }
    // MAP FUNCTIONS
    fn map_cmd_output(&self, output: &Path) -> PathBuf {
        PathBuf::from(output)
    }
    fn map_define(&self, define: &str) -> String {
        String::from(define)
    }
    fn map_include(&self, include: &Path) -> PathBuf {
        PathBuf::from(include)
    }
    fn map_lib(&self, lib: &Path) -> PathBuf {
        PathBuf::from(lib)
    }
    fn map_source(&self, source: &Path) -> PathBuf {
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
