// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DDesktopIntel {
    src_path: PathBuf,
}

const DEFAULTS: &str = "mesa3d-desktop-intel-defaults";
const RAW_DEFAULTS: &str = "mesa3d-desktop-intel-raw-defaults";

impl Project for Mesa3DDesktopIntel {
    fn get_name(&self) -> &'static str {
        "mesa3d/desktop-intel"
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

        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::new(
            &["//visibility:public"],
            "mesa3d_desktop_intel_licenses",
            &[
                "SPDX-license-identifier-MIT",
                "SPDX-license-identifier-Apache-2.0",
                "SPDX-license-identifier-GPL-1.0-or-later",
                "SPDX-license-identifier-GPL-2.0-only",
            ],
            &[
                "licenses/MIT",
                "licenses/Apache-2.0",
                "licenses/GPL-1.0-or-later",
                "licenses/GPL-2.0-only",
            ],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[
                target!(
                    "src/mapi/shared-glapi/libglapi.so.0.0.0",
                    "mesa3d_desktop-intel_libglapi",
                    "libglapi"
                ),
                target!(
                    "src/gallium/targets/dri/libgallium_dri.so",
                    "mesa3d_desktop-intel_libgallium_dri",
                    "libgallium_dri"
                ),
                target!(
                    "src/egl/libEGL_mesa.so.1.0.0",
                    "mesa3d_desktop-intel_libEGL_mesa",
                    "libEGL_mesa"
                ),
                target!(
                    "src/mapi/es2api/libGLESv2_mesa.so.2.0.0",
                    "mesa3d_desktop-intel_libGLESv2_mesa",
                    "libGLESv2_mesa"
                ),
                target!(
                    "src/mapi/es1api/libGLESv1_CM_mesa.so.1.1.0",
                    "mesa3d_desktop-intel_libGLESv1_CM_mesa",
                    "libGLESv1_CM_mesa"
                ),
                target!(
                    "src/intel/vulkan/libvulkan_intel.so",
                    "mesa3d_desktop-intel_libvulkan_intel",
                    "vulkan.intel"
                ),
                target!(
                    "src/tool/pps/pps-producer",
                    "mesa3d_desktop-intel_pps-producer",
                    "pps-producer"
                ),
                target!(
                    "src/tool/pps/libgpudataproducer.so",
                    "mesa3d_desktop-intel_libgpudataproducer",
                    "libgpudataproducer"
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

        common::ninja_build(&build_path, &gen_deps, ctx)?;

        package.filter_local_include_dirs(MESON_GENERATED, &gen_deps)?;
        common::clean_gen_deps(&gen_deps, &build_path, ctx)?;
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        let default_module = SoongModule::new("cc_defaults")
            .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
            .add_props(package.get_props("mesa3d_desktop-intel_pps-producer", vec!["cflags"])?)
            .add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from(RAW_DEFAULTS)]),
            );

        package
            .add_module(default_module)
            .add_raw_suffix(&format!(
                r#"
cc_defaults {{
    name: "{RAW_DEFAULTS}",
    soc_specific: true,
    header_libs: [
        "libcutils_headers",
        "libhardware_headers",
        "liblog_headers",
    ],
}}
"#,
            ))
            .add_raw_prefix(
                r#"
soong_namespace {
}
"#,
            )
            .print(ctx)
    }

    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        let relative_install = |module: SoongModule| -> SoongModule {
            for lib in [
                "libGLESv1_CM_mesa.so.1.1.0",
                "libGLESv2_mesa.so.2.0.0",
                "libEGL_mesa.so.1.0.0",
            ] {
                if target.ends_with(lib) {
                    return module
                        .add_prop("relative_install_path", SoongProp::Str(String::from("egl")));
                }
            }
            if target.ends_with("libvulkan_intel.so") {
                return module
                    .add_prop("relative_install_path", SoongProp::Str(String::from("hw")));
            }
            module
        };
        let module = relative_install(module);

        let mut libs = Vec::new();
        for lib in [
            "libgallium.a",
            "libpipe_loader_static.a",
            "libiris.a",
            "libdri.a",
            "libswkmsdri.a",
            "libintel_dev.a",
            "libanv_common.a",
            "libloader.a",
            "libmesa_util.a",
            "libpipe_loader_dynamic.a",
            "libpps.a",
            "libvulkan_instance.a",
            "libvulkan_lite_runtime.a",
            "libvulkan_wsi.a",
        ] {
            if target.ends_with(lib) {
                libs.push("libdrm_headers");
                break;
            }
        }
        for lib in ["libvulkan_runtime.a", "libvulkan_lite_runtime.a"] {
            if target.ends_with(lib) {
                libs.push("hwvulkan_headers");
            }
        }
        if target.ends_with("libEGL_mesa.so.1.0.0") {
            libs.push("libnativebase_headers");
        }

        let module = module.add_prop(
            "header_libs",
            SoongProp::VecStr(libs.into_iter().map(|lib| String::from(lib)).collect()),
        );

        let module = if target.ends_with("libvulkan_intel.so") {
            module.add_prop("afdo", SoongProp::Bool(true))
        } else {
            module
        };

        let mut cflags = vec!["-Wno-non-virtual-dtor", "-Wno-error"];
        if target.ends_with("libvulkan_lite_runtime.a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        let mut shared_libs = Vec::new();
        if target.ends_with("libdri.a")
            || target.ends_with("libanv_common.a")
            || target.ends_with("libvulkan_wsi.a")
            || target.ends_with("libvulkan_lite_runtime.a")
        {
            shared_libs.push("libsync");
        }
        if target.ends_with("libmesa_util.a") {
            shared_libs.push("libz");
        }
        if target.starts_with("src/intel/vulkan") || target.ends_with("libvulkan_lite_runtime.a") {
            shared_libs.push("libnativewindow");
        }
        let mut static_libs = Vec::new();
        if [
            "libmesa_util.a",
            "libintel-driver-ds.a",
            "libpps.a",
            "libpps-intel.a",
        ]
        .contains(&file_name(target).as_str())
        {
            static_libs.push("libperfetto_client_experimental");
        }
        if ![
            "libintel_decoder_brw.a",
            "libintel_decoder_elk.a",
            "libperfetto.a",
        ]
        .contains(&file_name(target).as_str())
        {
            module.add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
        } else {
            module.add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from(RAW_DEFAULTS)]),
            )
        }
        .extend_prop("cflags", cflags)?
        .extend_prop("static_libs", static_libs)?
        .extend_prop("shared_libs", shared_libs)
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
        let inc = path_to_string(include);
        !include.ends_with("android_stub")
            && !inc.contains(&path_to_string(&self.src_path.join("subprojects")))
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
