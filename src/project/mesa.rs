// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

const MESON_GENERATED: &str = "meson_generated";
const TARGETS: [(&str, Option<&str>, Option<&str>); 7] = [
    (
        "src/egl/libEGL_mesa.so.1.0.0",
        Some("libEGL_mesa"),
        Some("libEGL_mesa_intel"),
    ),
    (
        "src/mapi/es2api/libGLESv2_mesa.so.2.0.0",
        Some("libGLESv2_mesa"),
        Some("libGLESv2_mesa_intel"),
    ),
    (
        "src/mapi/es1api/libGLESv1_CM_mesa.so.1.1.0",
        Some("libGLESv1_CM_mesa"),
        Some("libGLESv1_CM_mesa_intel"),
    ),
    ("src/mapi/shared-glapi/libglapi.so.0.0.0", None, None),
    ("src/gallium/targets/dri/libgallium_dri.so", None, None),
    (
        "src/intel/vulkan/libvulkan_intel.so",
        Some("vulkan.intel"),
        Some("libvulkan_intel"),
    ),
    ("src/tool/pps/pps-producer", None, Some("pps-producer")),
];

#[derive(Default)]
pub struct Mesa {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
}

impl Project for Mesa {
    fn get_name(&self) -> &'static str {
        "mesa"
    }
    fn get_android_path(&self, ctx: &Context) -> PathBuf {
        ctx.android_path.join("external").join(self.get_name())
    }
    fn get_test_path(&self, ctx: &Context) -> PathBuf {
        ctx.test_path.join(self.get_name())
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = if let Ok(path) = std::env::var("N2S_MESA_PATH") {
            PathBuf::from(path)
        } else {
            self.get_android_path(ctx)
        };
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;
        self.build_path = ctx.temp_path.join(self.get_name());

        let intel_clc_path = if !ctx.skip_build {
            let intel_clc_build_path = ctx.temp_path.join("intel_clc");
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(self.get_test_path(ctx).join("build_intel_clc.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&intel_clc_build_path)
                ]
            )?;
            intel_clc_build_path.join("src/intel/compiler")
        } else {
            self.get_test_path(ctx)
        };
        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(&self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(intel_clc_path),
                    ANDROID_PLATFORM,
                    &path_to_string(&self.ndk_path)
                ]
            )?;
        }
        if !ctx.skip_build {
            execute_cmd!(
                "meson",
                vec!["compile", "-C", &path_to_string(&self.build_path)]
            )?;
        }

        let targets = parse_build_ninja::<MesonNinjaTarget>(&self.build_path)?;
        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            "//visibility:public",
            "mesa_licenses",
            vec!["SPDX-license-identifier-Apache-2.0"],
            vec!["docs/license.rst"],
        );
        let targets_to_generate = TARGETS
            .iter()
            .map(|(target, _, _)| PathBuf::from(target))
            .collect();
        package.generate(targets_to_generate, targets, self)?;

        let gen_deps = package.get_gen_deps();
        package.filter_local_include_dirs(MESON_GENERATED, &gen_deps);
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &self.build_path, ctx, self)?;

        Ok(package)
    }

    fn get_target_name(&self, target: &str) -> String {
        for (target_str, _, some_alias) in TARGETS {
            if target == path_to_id(Path::new(self.get_name()).join(target_str)) {
                if let Some(alias) = some_alias {
                    return String::from(alias);
                }
            }
        }
        String::from(target)
    }
    fn get_target_stem(&self, target: &str) -> Option<String> {
        for (target_str, some_stem, _) in TARGETS {
            if target == path_to_id(Path::new(self.get_name()).join(target_str)) {
                if let Some(stem) = some_stem {
                    return Some(String::from(stem));
                }
            }
        }
        None
    }
    fn get_target_object_module(&self, target: &str, mut module: SoongModule) -> SoongModule {
        let mut libs = Vec::new();
        for lib in [
            "libgallium_a",
            "libpipe_loader_static_a",
            "libiris_a",
            "libdri_a",
            "libswkmsdri_a",
            "libintel_dev_a",
            "libanv_common_a",
            "libloader_a",
            "libmesa_util_a",
            "libpps_a",
            "libvulkan_instance_a",
            "libvulkan_lite_runtime_a",
            "libvulkan_wsi_a",
        ] {
            if target.ends_with(lib) {
                libs.push("libdrm_headers");
                break;
            }
        }
        if target.ends_with("libanv_common_a") {
            libs.push("hwvulkan_headers");
        }
        module.add_prop(
            "header_libs",
            SoongProp::VecStr(libs.into_iter().map(|lib| String::from(lib)).collect()),
        );

        module.add_prop(
            "arch",
            SoongNamedProp::new_prop(
                "x86",
                SoongNamedProp::new_prop("enabled", SoongProp::Bool(false)),
            ),
        );
        module
    }
    fn get_target_cflags(&self, target: &str) -> Vec<String> {
        let mut cflags = vec!["-Wno-non-virtual-dtor", "-Wno-error"];
        if target.ends_with("libvulkan_lite_runtime_a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        cflags.into_iter().map(|flag| String::from(flag)).collect()
    }
    fn get_target_shared_libs(&self, target: &str) -> Vec<String> {
        let mut libs = Vec::new();
        if target.ends_with("libdri_a")
            || target.ends_with("libanv_common_a")
            || target.ends_with("libvulkan_wsi_a")
            || target.ends_with("libvulkan_lite_runtime_a")
        {
            libs.push("libsync");
        }
        if target.ends_with("libmesa_util_a") {
            libs.push("libz");
        }
        if target.starts_with("mesa_src_intel_vulkan_")
            || target.ends_with("libvulkan_lite_runtime_a")
        {
            libs.push("libnativewindow");
        }
        libs.into_iter().map(|lib| String::from(lib)).collect()
    }

    fn get_define(&self, define: &str) -> String {
        define
            .replace(&path_to_string(&self.build_path), MESON_GENERATED)
            .replace(&path_to_string_with_separator(&self.src_path), "")
    }
    fn get_include(&self, include: &Path) -> PathBuf {
        Path::new(MESON_GENERATED).join(strip_prefix(include, &self.build_path))
    }
    fn get_lib(&self, library: &Path) -> PathBuf {
        for (target, _, alias) in TARGETS {
            if let Some(alias) = alias {
                if library == Path::new(target) {
                    return PathBuf::from(alias);
                }
            }
        }
        if file_name(library) == "libdrm.so" {
            PathBuf::from("libdrm")
        } else if library.starts_with("src/android_stub") {
            PathBuf::from(file_stem(library))
        } else if library.starts_with("src") || library.starts_with("subprojects") {
            Path::new(self.get_name()).join(library)
        } else {
            PathBuf::from(library)
        }
    }
    fn get_source(&self, source: &Path) -> PathBuf {
        if let Ok(strip) = source.strip_prefix(&self.build_path) {
            self.src_path.join(MESON_GENERATED).join(strip)
        } else {
            PathBuf::from(source)
        }
    }

    fn filter_cflag(&self, cflag: &str) -> bool {
        cflag == "-mclflushopt"
    }
    fn filter_define(&self, define: &str) -> bool {
        define != "WITH_LIBBACKTRACE" // b/120606663
    }
    fn filter_include(&self, include: &Path) -> bool {
        !(path_to_string(include)
            .starts_with(&path_to_string(self.src_path.join("subprojects/libdrm")))
            || include.ends_with("android_stub"))
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_lib(&self, lib: &str) -> bool {
        !lib.contains("libbacktrace")
    }
    fn filter_target(&self, target: &Path) -> bool {
        let file_name = file_name(target);
        (file_name.contains(".so")
            || file_name.contains(".a")
            || file_name.contains("pps-producer"))
            && !file_name.contains("libdrm")
            && !target.starts_with("src/android_stub")
    }
}
