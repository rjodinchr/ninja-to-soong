// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Mesa3DDesktopPanVK {
    src_path: PathBuf,
}

const DEFAULTS: &str = "mesa3d-desktop-panvk-defaults";
const RAW_DEFAULTS: &str = "mesa3d-desktop-panvk-raw-defaults";

impl mesa3d_desktop::Mesa3dProject for Mesa3DDesktopPanVK {
    fn get_name(&self) -> &'static str {
        "mesa3d/desktop-panvk"
    }

    fn get_src_path(&self) -> PathBuf {
        self.src_path.clone()
    }

    fn asset_filter(&self, asset: &Path) -> bool {
        let asset = path_to_string(asset);
        // mesa_clc
        !asset.contains("libpan/libpan_v") && !asset.contains("libpan/libpan_shaders_v")
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
            "mesa3d_desktop_panvk_licenses",
            &[
                "SPDX-license-identifier-Apache-2.0",
                "SPDX-license-identifier-MIT",
                "SPDX-license-identifier-BSL-1.0",
            ],
            &["licenses/Apache-2.0", "licenses/MIT", "licenses/BSL-1.0"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[target!(
                "src/gallium/targets/lavapipe/libvulkan_lvp.so",
                "mesa3d_desktop_panvk_vulkan_lvp",
                "vulkan.lvp"
            )]),
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
            .add_props(package.get_props(
                "mesa3d_desktop-panvk_src_x11_libloader_x11_a",
                vec!["cflags"],
            )?)
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
    header_libs: ["libdrm_headers"],
    static_libs: ["libperfetto_client_experimental"],
}}
"#
        )
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> Result<SoongModule, String> {
        module.update_prop("generated_headers", |prop| match prop {
            SoongProp::VecStr(mut vec) => {
                vec.push(path_to_id(
                    Path::new(mesa3d_desktop::Mesa3dProject::get_name(self))
                        .join("src/util/shader_stats.h"),
                ));
                Ok(SoongProp::VecStr(vec))
            }
            _ => Ok(prop),
        })?;

        let mut defaults = vec![String::from(DEFAULTS)];
        if target.ends_with("libvulkan_lvp.so") {
            module = module
                .add_prop("relative_install_path", SoongProp::Str(String::from("hw")))
                .add_prop("afdo", SoongProp::Bool(true));
            defaults.push(CcDefaults::Llvm.str());
        }

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
        if target.ends_with("libmesa_util.a") {
            module = module.extend_prop("shared_libs", vec!["libz"])?;
        }
        module
            .add_prop("defaults", SoongProp::VecStr(defaults))
            .extend_prop("cflags", cflags)
    }
}
