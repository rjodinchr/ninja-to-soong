// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct SpirvTools {
    build_path: PathBuf,
    spirv_headers_path: PathBuf,
    gen_deps: Vec<PathBuf>,
}

impl Project for SpirvTools {
    fn get_name(&self) -> &'static str {
        "SPIRV-Tools"
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
        self.build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;
        self.spirv_headers_path = ProjectId::SpirvHeaders.get_android_path(projects_map, ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx)?.join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&ndk_path),
                    &path_to_string(&self.spirv_headers_path),
                ]
            )?;
        }

        let mut package = SoongPackage::new(
            &["//external/clvk"],
            "SPIRV-Tools_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE"],
        )
        .generate(
            NinjaTargetsToGenMap::from_dep(Dep::SpirvToolsTargets.get(projects_map)?),
            parse_build_ninja::<CmakeNinjaTarget>(&self.build_path)?,
            &src_path,
            &ndk_path,
            &self.build_path,
            None,
            self,
            ctx,
        )?
        .add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvTools,
            vec![String::from("include")],
        ));
        self.gen_deps = package.get_gen_deps();

        package.print(ctx)
    }

    fn get_deps_prefix(&self) -> Vec<(PathBuf, Dep)> {
        vec![(self.spirv_headers_path.clone(), Dep::SpirvHeaders)]
    }
    fn get_deps(&self, dep: Dep) -> Vec<PathBuf> {
        match dep {
            Dep::SpirvHeaders => self
                .gen_deps
                .iter()
                .map(|header| strip_prefix(header, &self.spirv_headers_path))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn extend_module(&self, _target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        module
            .add_prop("vendor_available", SoongProp::Bool(true))
            .add_prop(
                "header_libs",
                SoongProp::VecStr(vec![CcLibraryHeaders::SpirvHeaders.str()]),
            )
            .extend_prop("export_include_dirs", vec!["include"])?
            .add_prop(
                "export_header_lib_headers",
                SoongProp::VecStr(vec![CcLibraryHeaders::SpirvHeaders.str()]),
            )
            .extend_prop("cflags", vec!["-Wno-implicit-fallthrough"])
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_include(&self, include: &Path) -> bool {
        !(include.starts_with(&self.build_path) || include.starts_with(&self.spirv_headers_path))
    }
    fn filter_define(&self, _define: &str) -> bool {
        false
    }
}
