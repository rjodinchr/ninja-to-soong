// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

pub trait Mesa3dProject {
    fn get_name(&self) -> &'static str;
    fn get_subprojects_path(&self) -> String;
    fn create_package(
        &mut self,
        ctx: &Context,
        src_path: &Path,
        build_path: &Path,
        ndk_path: &Path,
        meson_generated: &str,
    ) -> Result<SoongPackage, String>;
    fn get_default_module(&self, package: &SoongPackage) -> Result<SoongModule, String>;
    fn get_raw_suffix(&self) -> String;
    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String>;
}

impl<T> Project for T
where
    T: Mesa3dProject,
{
    fn get_name(&self) -> &'static str {
        self.get_name()
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(Path::new("vendor/google/graphics").join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = ctx.get_android_path(self)?;
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;
        let build_path = ctx.temp_path.join(self.get_name());
        let script_path = ctx.get_script_path(self);

        let mesa_clc_path = if !ctx.skip_build {
            let mesa_clc_build_path = ctx.temp_path.join("mesa_clc");
            execute_cmd!(
                "bash",
                [
                    &path_to_string(script_path.join("build_mesa_clc.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&mesa_clc_build_path)
                ]
            )?;
            mesa_clc_build_path.join("bin")
        } else {
            script_path.clone()
        };

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(script_path.join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&build_path),
                    &path_to_string(mesa_clc_path),
                    &path_to_string(&ndk_path)
                ]
            )?;
        }

        const MESON_GENERATED: &str = "meson_generated";
        let mut package =
            self.create_package(ctx, &src_path, &build_path, &ndk_path, MESON_GENERATED)?;

        let gen_deps = package
            .get_gen_deps()
            .into_iter()
            .filter(|include| !include.starts_with("subprojects"))
            .collect();

        common::ninja_build(&build_path, &gen_deps, ctx)?;
        // Clean libdrm to prevent Soong from parsing blueprints that came with it
        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "git",
                [
                    "-C",
                    &path_to_string(&src_path),
                    "clean",
                    "-xfd",
                    "subprojects/libdrm*"
                ]
            )?;
        }

        package.filter_local_include_dirs(MESON_GENERATED, &gen_deps)?;
        common::clean_gen_deps(&gen_deps, &build_path, ctx)?;
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        let default_module = self.get_default_module(&package)?;

        package
            .add_module(default_module)
            .add_raw_suffix(&self.get_raw_suffix())
            .add_raw_prefix(
                r#"
soong_namespace {
}
"#,
            )
            .print(ctx)
    }

    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        self.extend_module(target, module)
    }

    fn map_lib(&self, library: &Path) -> Option<PathBuf> {
        if library.starts_with("src/android_stub")
            || (!library.starts_with("src") && !library.starts_with("subprojects/perfetto"))
        {
            Some(PathBuf::from(file_stem(library)))
        } else {
            None
        }
    }

    fn filter_cflag(&self, cflag: &str) -> bool {
        cflag == "-mclflushopt"
    }
    fn filter_include(&self, include: &Path) -> bool {
        !path_to_string(include).contains(&self.get_subprojects_path())
    }
    fn filter_link_flag(&self, flag: &str) -> bool {
        flag == "-Wl,--build-id=sha1"
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_target(&self, target: &Path) -> bool {
        let file_name = file_name(target);
        !file_name.ends_with(".o")
            && !file_name.ends_with(".def")
            && !file_name.contains("libdrm")
            && !target.starts_with("src/android_stub")
    }
}
