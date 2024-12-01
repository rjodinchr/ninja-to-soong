// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

pub struct SpirvHeaders<'a> {
    src_root: &'a str,
    ndk_root: &'a str,
}

const SPIRV_HEADERS_ID: ProjectId = ProjectId::SpirvHeaders;
const SPIRV_HEADERS_NAME: &str = SPIRV_HEADERS_ID.str();

impl<'a> SpirvHeaders<'a> {
    pub fn new(ndk_root: &'a str, spirv_headers_root: &'a str) -> Self {
        SpirvHeaders {
            src_root: spirv_headers_root,
            ndk_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for SpirvHeaders<'a> {
    fn get_id(&self) -> ProjectId {
        SPIRV_HEADERS_ID
    }

    fn generate_package(
        &mut self,
        _targets: Vec<NinjaTarget>,
        project_map: &ProjectMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            "",
            SPIRV_HEADERS_NAME,
            "//visibility:public",
            "SPDX-license-identifier-MIT",
            "LICENSE",
        );

        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIB_HEADERS_SPIRV_HEADERS,
            ["include".to_string()].into(),
        ));

        let mut files: Vec<String> = Vec::new();
        files.extend(Dependency::SpirvHeadersFiles.get(self, ProjectId::SpirvTools, project_map));
        files.extend(Dependency::SpirvHeadersFiles.get(self, ProjectId::CLSPV, project_map));
        let files_set: HashSet<String> = HashSet::from_iter(files);
        files = Vec::from_iter(files_set);
        files.sort();
        for file in files {
            package.add_module(SoongModule::new_copy_genrule(
                spirv_headers_name(self.src_root, &file),
                file.replace(&add_slash_suffix(self.src_root), ""),
                file.rsplit_once("/").unwrap().1.to_string(),
            ));
        }

        Ok(package)
    }

    fn get_project_dependencies(&self) -> Vec<ProjectId> {
        vec![ProjectId::SpirvTools, ProjectId::CLSPV]
    }
}
