// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct MesaClc();

impl Project for MesaClc {
    fn get_name(&self) -> &'static str {
        "mesa_clc"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from("vendor/google/graphics/mesa3d/desktop-panvk"))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = ctx.get_android_path(self)?;
        let build_path = ctx.get_temp_path(Path::new(self.get_name()))?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&build_path),
                ]
            )?;
        }
        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::default().generate(
            NinjaTargetsToGenMap::from(&[
                target!("src/compiler/clc/mesa_clc"),
                target!("src/compiler/spirv/vtn_bindgen2"),
            ]),
            parse_build_ninja::<MesonNinjaTarget>(&build_path)?,
            &src_path,
            Path::new("<no_sdk>"),
            &build_path,
            Some(MESON_GENERATED),
            self,
            ctx,
        )?;

        let gen_deps = package.get_dep_gen_assets();
        package.filter_gen_deps(MESON_GENERATED, &gen_deps)?;
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        package.print(ctx)
    }

    fn filter_cflag(&self, cflag: &str) -> bool {
        cflag.starts_with("-W")
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
}
