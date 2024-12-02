// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

pub struct SpirvHeaders<'a> {
    src_dir: &'a str,
}

impl<'a> SpirvHeaders<'a> {
    pub fn new(spirv_headers_dir: &'a str) -> Self {
        SpirvHeaders {
            src_dir: spirv_headers_dir,
        }
    }
}

impl<'a> crate::project::Project<'a> for SpirvHeaders<'a> {
    fn generate_package(
        &mut self,
        _targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_dir,
            "",
            "",
            ProjectId::SpirvHeaders.str(),
            "//visibility:public",
            "SPDX-license-identifier-MIT",
            "LICENSE",
        );

        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_SPIRV_HEADERS,
            ["include".to_string()].into(),
        ));

        let mut set: HashSet<String> = HashSet::new();
        set.extend(GenDeps::SpirvHeadersFiles.get(self, ProjectId::SpirvTools, projects_map));
        set.extend(GenDeps::SpirvHeadersFiles.get(self, ProjectId::Clspv, projects_map));
        let mut files = Vec::from_iter(set);
        files.sort();
        for file in files {
            package.add_module(SoongModule::new_copy_genrule(
                spirv_headers_name(self.src_dir, &file),
                file.replace(&add_slash_suffix(self.src_dir), ""),
                file.rsplit_once("/").unwrap().1.to_string(),
            ));
        }

        Ok(package)
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::SpirvHeaders
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::SpirvTools, ProjectId::Clspv]
    }
}
