// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::project::*;

#[derive(Default)]
pub struct SpirvHeaders {
    src_path: PathBuf,
}

impl Project for SpirvHeaders {
    fn get_id(&self) -> ProjectId {
        ProjectId::SpirvHeaders
    }

    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_id().android_path(ctx);
        let mut package = SoongPackage::new(
            &self.src_path,
            Path::new(""),
            Path::new(""),
            Path::new(self.get_id().str()),
            "//visibility:public",
            "SPDX-license-identifier-MIT",
            "LICENSE",
        );

        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvHeaders,
            vec![String::from("include")],
        ));

        let mut set: HashSet<PathBuf> = HashSet::new();
        set.extend(GenDeps::SpirvHeaders.get(self, ProjectId::SpirvTools, projects_map));
        set.extend(GenDeps::SpirvHeaders.get(self, ProjectId::Clspv, projects_map));
        let mut files = Vec::from_iter(set);
        files.sort();
        for file in files {
            package.add_module(SoongModule::new_copy_genrule(
                dep_name(
                    &file,
                    &self.src_path,
                    GenDeps::SpirvHeaders.str(),
                    Path::new(""),
                ),
                path_to_string(strip_prefix(&file, &self.src_path)),
                file_name(&file),
            ));
        }

        Ok(package)
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::SpirvTools, ProjectId::Clspv]
    }
}
