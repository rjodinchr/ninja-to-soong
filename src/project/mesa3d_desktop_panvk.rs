// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DDesktopPanVK {
    src_path: PathBuf,
}

const DEFAULTS: &str = "mesa3d-desktop-panvk-defaults";

impl Project for Mesa3DDesktopPanVK {
    fn get_name(&self) -> &'static str {
        "mesa3d/desktop-panvk"
    }
    fn get_android_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx
            .get_android_path()?
            .join("vendor/google/graphics")
            .join(self.get_name()))
    }
    fn get_test_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx.test_path.join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        self.src_path = self.get_android_path(ctx)?;
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;
        let build_path = ctx.temp_path.join(self.get_name());

        let mesa_clc_path = if !ctx.skip_build {
            let mesa_clc_build_path = ctx.temp_path.join("mesa_clc");
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx)?.join("build_mesa_clc.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&mesa_clc_build_path)
                ]
            )?;
            mesa_clc_build_path.join("bin")
        } else {
            self.get_test_path(ctx)?
        };

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx)?.join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&build_path),
                    &path_to_string(mesa_clc_path),
                    &path_to_string(&ndk_path)
                ]
            )?;
        }
        if !ctx.skip_build {
            execute_cmd!("meson", ["compile", "-C", &path_to_string(&build_path)])?;
        }

        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::new(
            &["//visibility:public"],
            "mesa3d_desktop_panvk_licenses",
            &[
                "SPDX-license-identifier-Apache-2.0",
                "SPDX-license-identifier-MIT",
            ],
            &["licenses/Apache-2.0", "licenses/MIT"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[
                NinjaTargetToGen(
                    "src/panfrost/vulkan/libvulkan_panfrost.so",
                    Some("mesa3d_desktop-panvk_libvulkan_panfrost"),
                    Some("vulkan.panfrost"),
                ),
                NinjaTargetToGen(
                    "src/tool/pps/pps-producer",
                    Some("mesa3d_desktop-panvk_pps-producer"),
                    Some("pps-producer"),
                ),
            ]),
            parse_build_ninja::<MesonNinjaTarget>(&build_path)?,
            &self.src_path,
            &ndk_path,
            &build_path,
            Some(MESON_GENERATED),
            self,
            ctx,
        )?;

        let gen_deps = package
            .get_gen_deps()
            .into_iter()
            .filter(|include| !include.starts_with("subprojects"))
            .collect();
        package.filter_local_include_dirs(MESON_GENERATED, &gen_deps)?;
        common::clean_gen_deps(&gen_deps, &build_path, ctx)?;
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        let default_module = SoongModule::new("cc_defaults")
            .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
            .add_props(package.get_props("mesa3d_desktop-panvk_pps-producer", vec!["cflags"])?);

        package.add_module(default_module).print()
    }

    fn extend_module(&self, target: &Path, module: SoongModule) -> SoongModule {
        let is_soc_specific = |module: SoongModule| -> SoongModule {
            for lib in ["libvulkan_panfrost.so", "pps-producer"] {
                if target.ends_with(lib) {
                    return module.add_prop("soc_specific", SoongProp::Bool(true));
                }
            }
            module
        };
        let module = is_soc_specific(module);

        let relative_install = |module: SoongModule| -> SoongModule {
            if target.ends_with("libvulkan_panfrost.so") {
                return module
                    .add_prop("relative_install_path", SoongProp::Str(String::from("hw")));
            }
            module
        };
        let module = relative_install(module);

        let mut libs = Vec::new();
        for lib in [
            "libmesa_util.a",
            "libpanfrost_lib.a",
            "libpanfrost_perf.a",
            "libpankmod_lib.a",
            "libpanvk_v6.a",
            "libpanvk_v7.a",
            "libpanvk_v10.a",
            "libpanvk_v12.a",
            "libpanvk_v13.a",
            "libpps.a",
            "libpps-panfrost.a",
            "libvulkan_instance.a",
            "libvulkan_lite_runtime.a",
            "libvulkan_wsi.a",
        ] {
            if target.ends_with(lib) {
                libs.push("libdrm_headers");
                break;
            }
        }
        if target.ends_with("libpanvk_v6.a")
            || target.ends_with("libpanvk_v7.a")
            || target.ends_with("libpanvk_v10.a")
            || target.ends_with("libpanvk_v12.a")
            || target.ends_with("libpanvk_v13.a")
            || target.ends_with("libvulkan_panfrost.so")
        {
            libs.push("hwvulkan_headers");
        }

        let module = module.add_prop(
            "header_libs",
            SoongProp::VecStr(libs.into_iter().map(|lib| String::from(lib)).collect()),
        );

        let module = if target.ends_with("libvulkan_panfrost.so") {
            module.add_prop("afdo", SoongProp::Bool(true))
        } else {
            module
        };

        if !["libperfetto.a"].contains(&file_name(target).as_str()) {
            module.add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
        } else {
            module
        }
    }
    fn extend_cflags(&self, target: &Path) -> Vec<String> {
        let mut cflags = vec![
            "-Wno-constant-conversion",
            "-Wno-enum-conversion",
            "-Wno-error",
            "-Wno-ignored-qualifiers",
            "-Wno-initializer-overrides",
            "-Wno-macro-redefined",
            "-Wno-non-virtual-dtor",
            "-Wno-pointer-arith",
            "-Wno-unused-parameter",
        ];
        if target.ends_with("libvulkan_lite_runtime.a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        cflags.into_iter().map(|flag| String::from(flag)).collect()
    }
    fn extend_shared_libs(&self, target: &Path) -> Vec<String> {
        let mut libs = Vec::new();
        if target.ends_with("libpankmod_lib.a")
            || target.ends_with("libvulkan_lite_runtime.a")
            || target.ends_with("libvulkan_wsi.a")
        {
            libs.push("libsync");
        }
        if target.ends_with("libmesa_util.a") {
            libs.push("libz");
        }
        if target.starts_with("src/panfrost/vulkan") || target.ends_with("libvulkan_lite_runtime.a")
        {
            libs.push("libnativewindow");
        }
        libs.into_iter().map(|lib| String::from(lib)).collect()
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

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_define(&self, define: &str) -> bool {
        define != "WITH_LIBBACKTRACE" // b/120606663
    }
    fn filter_include(&self, include: &Path) -> bool {
        let inc = path_to_string(include);
        let subprojects = self.src_path.join("subprojects");
        !include.ends_with("android_stub")
            && (!inc.contains(&path_to_string(&subprojects))
                || inc.contains(&path_to_string(&subprojects.join("perfetto"))))
    }
    fn filter_link_flag(&self, flag: &str) -> bool {
        flag == "-Wl,--build-id=sha1"
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_lib(&self, lib: &str) -> bool {
        !lib.contains("libbacktrace")
    }
    fn filter_target(&self, target: &Path) -> bool {
        let file_name = file_name(target);
        !file_name.ends_with(".o")
            && !file_name.ends_with(".def")
            && !file_name.contains("libdrm")
            && !target.starts_with("src/android_stub")
    }
}
