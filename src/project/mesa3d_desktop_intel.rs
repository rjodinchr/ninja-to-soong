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
    fn get_intel_tools_targets(&self, build_path: &Path) -> Result<Vec<NinjaTargetToGen>, String> {
        Ok(ls_dir(&build_path.join("src/intel/tools"))?
            .into_iter()
            .filter_map(|entry| {
                let name = file_stem(&entry);
                if name.starts_with("lib") {
                    return None;
                }
                Some(target!(
                    format!("src/intel/tools/{name}"),
                    format!("mesa3d_desktop-intel_tools_{name}"),
                    name
                ))
            })
            .collect::<Vec<_>>())
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

        let mut targets = vec![
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
        ];
        targets.extend(self.get_intel_tools_targets(build_path)?);
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
            NinjaTargetsToGenMap::from(&targets),
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
    cflags: ["-Wno-error"],
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
        if target.ends_with("libvulkan_intel.so") {
            module = module
                .add_prop("relative_install_path", SoongProp::Str(String::from("hw")))
                .add_prop("afdo", SoongProp::Bool(true));
        }

        if target.ends_with("libintel_decoder.a") {
            module = module
                .extend_prop("static_libs", vec!["libexpat"])?
                .extend_prop("shared_libs", vec!["libz"])?;
        }

        if target.ends_with("libmesa_util.a") {
            module = module.extend_prop("shared_libs", vec!["libz"])?;
        }
        module = if ![
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
        };
        Ok(module)
    }
}
