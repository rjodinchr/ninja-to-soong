// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;
use crate::soong_module::SoongModule;

#[derive(Default)]
pub struct SpirvHeaders {
    src_path: PathBuf,
}

impl Project for SpirvHeaders {
    fn init(&mut self, android_path: &Path, _ndk_path: &Path, _temp_path: &Path) {
        self.src_path = self.get_id().android_path(android_path);
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::SpirvHeaders
    }

    fn generate_package(
        &mut self,
        _targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            &self.src_path,
            Path::new(""),
            Path::new(""),
            self.get_id().str(),
            "//visibility:public",
            "SPDX-license-identifier-MIT",
            "LICENSE",
        );

        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_SPIRV_HEADERS,
            ["include".to_string()].into(),
        ));

        let mut set: HashSet<PathBuf> = HashSet::new();
        set.extend(GenDeps::SpirvHeadersFiles.get(self, ProjectId::SpirvTools, projects_map));
        set.extend(GenDeps::SpirvHeadersFiles.get(self, ProjectId::Clspv, projects_map));
        let mut files = Vec::from_iter(set);
        files.sort();
        for file in files {
            package.add_module(SoongModule::new_copy_genrule(
                spirv_headers_name(&self.src_path, &file),
                str(&strip_prefix(&file, &self.src_path)),
                file_name(&file),
            ));
        }

        Ok(package)
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::SpirvTools, ProjectId::Clspv]
    }
}
