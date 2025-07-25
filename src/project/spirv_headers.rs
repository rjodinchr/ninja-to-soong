// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct SpirvHeaders();

impl Project for SpirvHeaders {
    fn get_name(&self) -> &'static str {
        "SPIRV-Headers"
    }
    fn get_android_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx
            .get_android_path()?
            .join("external")
            .join(self.get_name()))
    }
    fn get_test_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx.test_path.join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = self.get_android_path(ctx)?;
        let mut package = SoongPackage::new(
            &[
                "//external/SPIRV-Tools",
                "//external/clspv",
                "//external/clvk",
            ],
            "SPIRV-Headers_license",
            &["SPDX-license-identifier-MIT"],
            &["LICENSE"],
        )
        .add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvHeaders,
            vec![String::from("include")],
        ));

        for file in Dep::SpirvHeaders.get(projects_map)? {
            package = package.add_module(SoongModule::new_filegroup(
                Dep::SpirvHeaders.get_id(&file, &src_path, Path::new("")),
                vec![path_to_string(file)],
            ));
        }

        package.print(ctx)
    }
}
