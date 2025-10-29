// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Vkoverhead();

impl Project for Vkoverhead {
    fn get_name(&self) -> &'static str {
        "vkoverhead"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(Path::new("external").join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = ctx.get_android_path(self)?;
        let build_path = ctx.get_temp_path(Path::new(self.get_name()))?;
        let ndk_path = get_ndk_path(ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&build_path),
                    &path_to_string(&ndk_path),
                ]
            )?;
        }

        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::new(
            &[],
            "vkoverhead_license",
            &["SPDX-license-identifier-MIT"],
            &["LICENSE"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[target!("vkoverhead", "vkoverhead")]),
            parse_build_ninja::<MesonNinjaTarget>(&build_path)?,
            &src_path,
            &ndk_path,
            &build_path,
            Some(MESON_GENERATED),
            self,
            ctx,
        )?;

        let gen_deps = package.get_dep_gen_assets();
        package.filter_gen_deps(MESON_GENERATED, &gen_deps)?;

        package.print(ctx)
    }

    fn extend_module(&self, _target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        module
            .add_prop(
                "header_libs",
                SoongProp::VecStr(vec![String::from("libcutils_headers")]),
            )
            .extend_prop("shared_libs", vec!["libcutils"])
    }
    fn extend_python_binary_host(
        &self,
        _python_binary_path: &Path,
        module: SoongModule,
    ) -> Result<SoongModule, String> {
        Ok(module.add_prop("libs", SoongProp::VecStr(vec![String::from("mako")])))
    }

    fn map_cmd_output(&self, output: &Path) -> Option<String> {
        Some(file_name(output))
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
}
