// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

const TARGETS: [NinjaTargetToGen; 5] = [
    NinjaTargetToGen(
        "src/egl/libEGL_mesa.so.1.0.0",
        Some("libEGL_mesa_intel"),
        Some("libEGL_mesa"),
    ),
    NinjaTargetToGen(
        "src/mapi/es2api/libGLESv2_mesa.so.2.0.0",
        Some("libGLESv2_mesa_intel"),
        Some("libGLESv2_mesa"),
    ),
    NinjaTargetToGen(
        "src/mapi/es1api/libGLESv1_CM_mesa.so.1.1.0",
        Some("libGLESv1_CM_mesa_intel"),
        Some("libGLESv1_CM_mesa"),
    ),
    NinjaTargetToGen(
        "src/intel/vulkan/libvulkan_intel.so",
        Some("libvulkan_intel"),
        Some("vulkan.intel"),
    ),
    NinjaTargetToGen("src/tool/pps/pps-producer", Some("pps-producer"), None),
];

#[derive(Default)]
pub struct Mesa {
    src_path: PathBuf,
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
    ) -> Result<String, String> {
        self.src_path = if let Ok(path) = std::env::var("N2S_MESA_PATH") {
            PathBuf::from(path)
        } else {
            self.get_android_path(ctx)
        };
        let ndk_path = get_ndk_path(&ctx.temp_path)?;
        let build_path = ctx.temp_path.join(self.get_name());

        let intel_clc_path = if !ctx.skip_build {
            let intel_clc_build_path = ctx.temp_path.join("intel_clc");
            execute_cmd!(
                "bash",
                [
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
                [
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&build_path),
                    &path_to_string(intel_clc_path),
                    &path_to_string(&ndk_path)
                ]
            )?;
        }
        if !ctx.skip_build {
            execute_cmd!("meson", ["compile", "-C", &path_to_string(&build_path)])?;
        }

        const MESON_GENERATED: &str = "meson_generated";
        let mut package = SoongPackage::new(
            "//visibility:public",
            "mesa_licenses",
            &["SPDX-license-identifier-Apache-2.0"],
            &["docs/license.rst"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&TARGETS),
            parse_build_ninja::<MesonNinjaTarget>(&build_path)?,
            &self.src_path,
            &ndk_path,
            &build_path,
            Some(MESON_GENERATED),
            self,
        )?;

        let gen_deps = package
            .get_gen_deps()
            .into_iter()
            .filter(|include| !include.starts_with("subprojects"))
            .collect();
        package.filter_local_include_dirs(MESON_GENERATED, &gen_deps);
        common::copy_gen_deps(gen_deps, MESON_GENERATED, &build_path, ctx, self)?;

        Ok(package.print())
    }

    fn extend_module(&self, target: &Path, module: SoongModule) -> SoongModule {
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
        if target.ends_with("libanv_common.a") {
            libs.push("hwvulkan_headers");
        }
        module
            .add_prop(
                "header_libs",
                SoongProp::VecStr(libs.into_iter().map(|lib| String::from(lib)).collect()),
            )
            .add_prop(
                "arch",
                SoongNamedProp::new_prop(
                    "x86",
                    SoongNamedProp::new_prop("enabled", SoongProp::Bool(false)),
                ),
            )
    }
    fn extend_cflags(&self, target: &Path) -> Vec<String> {
        let mut cflags = vec!["-Wno-non-virtual-dtor", "-Wno-error"];
        if target.ends_with("libvulkan_lite_runtime.a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        cflags.into_iter().map(|flag| String::from(flag)).collect()
    }
    fn extend_shared_libs(&self, target: &Path) -> Vec<String> {
        let mut libs = Vec::new();
        if target.ends_with("libdri.a")
            || target.ends_with("libanv_common.a")
            || target.ends_with("libvulkan_wsi.a")
            || target.ends_with("libvulkan_lite_runtime.a")
        {
            libs.push("libsync");
        }
        if target.ends_with("libmesa_util.a") {
            libs.push("libz");
        }
        if target.starts_with("src/intel/vulkan") || target.ends_with("libvulkan_lite_runtime.a") {
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

    fn filter_cflag(&self, cflag: &str) -> bool {
        cflag == "-mclflushopt"
    }
    fn filter_define(&self, define: &str) -> bool {
        define != "WITH_LIBBACKTRACE" // b/120606663
    }
    fn filter_include(&self, include: &Path) -> bool {
        !include.ends_with("android_stub")
            && (!include.starts_with(self.src_path.join("subprojects"))
                || include.starts_with(self.src_path.join("subprojects/perfetto")))
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
        !file_name.ends_with(".o")
            && !file_name.ends_with(".def")
            && !file_name.contains("libdrm")
            && !target.starts_with("src/android_stub")
    }
}
