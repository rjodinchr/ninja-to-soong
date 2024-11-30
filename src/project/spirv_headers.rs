// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::ninja_target::NinjaTarget;
use crate::project::Project;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;
use crate::utils::*;

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
        dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<SoongPackage, String> {
        let spirv_tools = dep_packages.get(&ProjectId::SpirvTools).unwrap();
        let mut files = spirv_tools.get_generated_deps();
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

        files.insert(self.src_root.to_string() + "/include/spirv/unified1/spirv.hpp"); // for clspv
        let mut sorted = Vec::from_iter(files);
        sorted.sort();
        for file in sorted {
            package.add_module(SoongModule::new_copy_genrule(
                spirv_headers_name(self.src_root, &file),
                file.replace(&add_slash_suffix(self.src_root), ""),
                file.rsplit_once("/").unwrap().1.to_string(),
            ));
        }

        return Ok(package);
    }
    fn get_build_directory(
        &mut self,
        dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<String, String> {
        Ok(dep_packages
            .get(&ProjectId::SpirvTools)
            .unwrap()
            .get_generated_build_directory())
    }
    fn get_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::SpirvTools]
    }
}
