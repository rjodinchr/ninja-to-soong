// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct SpirvHeaders();

impl Project for SpirvHeaders {
    fn get_name(&self) -> &'static str {
        "SPIRV-Headers"
    }
    fn get_android_path(&self, ctx: &Context) -> PathBuf {
        ctx.android_path.join("external").join(self.get_name())
    }
    fn get_test_path(&self, ctx: &Context) -> PathBuf {
        ctx.test_path.join(self.get_name())
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = self.get_android_path(ctx);
        let mut package = SoongPackage::new(
            &src_path,
            Path::new(""),
            Path::new(""),
            "//visibility:public",
            "SPIRV-Headers_license",
            vec!["SPDX-license-identifier-MIT"],
            vec!["LICENSE"],
        );
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvHeaders,
            vec![String::from("include")],
        ));

        for file in Dep::SpirvHeaders.get(projects_map)? {
            package.add_module(SoongModule::new_copy_genrule(
                Dep::SpirvHeaders.get_id(&file, &src_path, Path::new("")),
                &file,
            ));
        }

        Ok(package.print())
    }
}
