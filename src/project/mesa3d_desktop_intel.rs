// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DDesktopIntel {
    src_path: PathBuf,
}

const DEFAULTS: &str = "mesa3d-desktop-intel-defaults";
const RAW_DEFAULTS: &str = "mesa3d-desktop-intel-raw-defaults";

impl Mesa3DDesktopIntel {
    fn get_intel_tools(&self, build_path: &Path) -> Result<Vec<String>, std::io::Error> {
        let intel_tools_path = build_path.join("src/intel/tools");
        let mut intel_tools: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(intel_tools_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(stem) = path.file_stem() {
                    let stem_str = stem.to_str().unwrap_or("");
                    if stem_str.starts_with("lib") || stem_str.is_empty() {
                        continue;
                    }
                    intel_tools.push(stem_str.to_string());
                }
            }
        }
        Ok(intel_tools)
    }

    fn get_intel_tools_targets(&self, build_path: &Path) -> Result<Vec<NinjaTargetToGen>, String> {
        let intel_tools_names = self
            .get_intel_tools(build_path)
            .map_err(|e| e.to_string())?;
        let mut targets: Vec<NinjaTargetToGen> = Vec::new();
        for name in intel_tools_names {
            targets.push(target!(
                format!("src/intel/tools/{}", name),
                format!("mesa3d_desktop-intel_tools_{}", name),
                name
            ));
        }
        Ok(targets)
    }
}

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
            // vtn_bindgen2
            "_shaders_binding.cpp",
            "_shaders_binding.h",
        ] {
            if asset.ends_with(name) {
                return false;
            }
        }
        if asset.contains("expat") {
            return false;
        }

        true
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

        let intel_tools_targets = self.get_intel_tools_targets(build_path)?;

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
            NinjaTargetsToGenMap::from(
                [
                    vec![
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
                    ],
                    intel_tools_targets,
                ]
                .concat()
                .as_slice(),
            ),
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
    static_libs: [
        "libperfetto_client_experimental",
    ],
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

        // Add intel tools dependencies.
        if target.ends_with("intel_hang_replay")
            || target.ends_with("aubinator_error_decode")
            || target.ends_with("aubinator")
        {
            module = module.extend_prop("static_libs", vec!["libexpat"])?;
        }
        if target.ends_with("libintel_decoder.a") || target.ends_with("libgallium_dri.so") {
            module = module
                .extend_prop("static_libs", vec!["libexpat"])?
                .extend_prop("shared_libs", vec!["libz"])?;
        }

        // Add intel tools dependencies if libvulkan_intel.so depends on the real decoder library.
        if target.ends_with("libvulkan_intel.so")
            && module
                .get_prop("static_libs")
                .is_some_and(|prop| prop.is_any_str_contain("libintel_decoder_a"))
        {
            module = module
                .extend_prop("static_libs", vec!["libexpat"])?
                .extend_prop("shared_libs", vec!["libz"])?;
        }

        let mut cflags = vec!["-Wno-non-virtual-dtor", "-Wno-error"];
        if target.ends_with("libvulkan_lite_runtime.a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        if target.ends_with("libmesa_util.a") {
            module = module.extend_prop("shared_libs", vec!["libz"])?;
        }
        if ![
            "libintel_decoder_brw.a",
            "libintel_decoder_elk.a",
            "libintel_decoder_stub_brw.a",
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
        .extend_prop("cflags", cflags)
    }
}
