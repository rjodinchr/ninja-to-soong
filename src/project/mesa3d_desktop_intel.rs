// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DDesktopIntel {
    src_path: PathBuf,
}

const DEFAULTS: &str = "mesa3d-desktop-intel-defaults";
const RAW_DEFAULTS: &str = "mesa3d-desktop-intel-raw-defaults";

impl mesa3d_desktop::Mesa3dProject for Mesa3DDesktopIntel {
    fn get_name(&self) -> &'static str {
        "mesa3d/desktop-intel"
    }

    fn get_subprojects_path(&self) -> String {
        path_to_string(&self.src_path.join("subprojects"))
    }

    fn asset_filter(&self, asset: &Path) -> bool {
        let asset = path_to_string(asset);
        for name in [
            // unsupported command arguments (XML sources)
            "api_beginend_init.h",
            "api_exec_decl.h",
            "api_exec_init.c",
            "api_hw_select_init.h",
            "api_save.h",
            "api_save_init.h",
            "dispatch.h",
            "es1_glapi_mapi_tmp.h",
            "es2_glapi_mapi_tmp.h",
            "genX_bits.h",
            "get_hash.h",
            "shared_glapi_mapi_tmp.h",
            "unmarshal_table.c",
            // unsupported command arguments
            "brw_nir_lower_fsign.c",
            "brw_nir_trig_workarounds.c",
            "brw_nir_workarounds.c",
            "isl_format_layout.c",
            "tr_util.c",
            "tr_util.h",
            "u_tracepoints.c",
            "u_tracepoints.h",
            "vk_synchronization_helpers.c",
            ".def",
            // different include paths
            "brw_device_sha1_gen.c",
            "nir_intrinsics.c",
            "nir_intrinsics.h",
            "nir_intrinsics_indices.h",
            "intel_perf_metrics.h",
            "intel_tracepoints.c",
            "intel_tracepoints.h",
            "intel_tracepoints_perfetto.h",
            "intel_wa.c",
            "intel_wa.h",
            // bison
            "glcpp-parse.c",
            "glcpp-parse.h",
            "glsl_parser.cpp",
            "glsl_parser.h",
            "program_parse.tab.h",
            "program_parse.tab.c",
            // flex
            "glcpp-lex.c",
            "glsl_lexer.cpp",
            "lex.yy.c",
            // vtn_bindgen2
            "_shaders_binding.cpp",
            "_shaders_binding.h",
            // ModuleNotFoundError: No module named 'license'
            "enums.c",
        ] {
            if asset.ends_with(name) {
                return false;
            }
        }
        // unsupported command arguments (XML sources)
        !asset.contains("marshal_generated")
    }

    fn create_package(
        &mut self,
        ctx: &Context,
        src_path: &Path,
        build_path: &Path,
        ndk_path: &Path,
        meson_generated: &str,
    ) -> Result<SoongPackage, String> {
        self.src_path = PathBuf::from(src_path);
        SoongPackage::new(
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
            Some(meson_generated),
            self,
            ctx,
        )
    }

    fn get_default_module(&self, package: &SoongPackage) -> Result<SoongModule, String> {
        Ok(SoongModule::new("cc_defaults")
            .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
            .add_props(package.get_props("mesa3d_desktop-intel_pps-producer", vec!["cflags"])?)
            .add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from(RAW_DEFAULTS)]),
            ))
    }

    fn get_raw_suffix(&self) -> String {
        format!(
            r#"
cc_defaults {{
    name: "{RAW_DEFAULTS}",
    soc_specific: true,
    static_libs: ["libperfetto_client_experimental"],
    header_libs: [
        "libcutils_headers",
        "libhardware_headers",
        "liblog_headers",
        "libdrm_headers",
    ],
}}
"#,
        )
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> Result<SoongModule, String> {
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
        module = relative_install(module);

        if target.ends_with("libvulkan_intel.so") {
            module = module.add_prop("afdo", SoongProp::Bool(true))
        }

        let mut cflags = vec!["-Wno-non-virtual-dtor", "-Wno-error"];
        if target.ends_with("libvulkan_lite_runtime.a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        if target.ends_with("libmesa_util.a") {
            module = module.extend_prop("shared_libs", vec!["libz"])?;
        }
        if !["libintel_decoder_brw.a", "libintel_decoder_elk.a"]
            .contains(&file_name(target).as_str())
        {
            module.add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
        } else {
            module.add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from(RAW_DEFAULTS)]),
            )
        }
        .extend_prop("cflags", cflags)
    }
}
