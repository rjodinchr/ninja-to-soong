// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use super::*;

const MESON_GENERATED: &str = "meson_generated";
const TARGETS: [&str; 7] = [
    "src/egl/libEGL_mesa.so.1.0.0",
    "src/mapi/es2api/libGLESv2_mesa.so.2.0.0",
    "src/mapi/es1api/libGLESv1_CM_mesa.so.1.1.0",
    "src/mapi/shared-glapi/libglapi.so.0.0.0",
    "src/gallium/targets/dri/libgallium_dri.so",
    "src/intel/vulkan/libvulkan_intel.so",
    "src/tool/pps/pps-producer",
];

#[derive(Default)]
pub struct Mesa {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
}

impl Project for Mesa {
    fn get_id(&self) -> ProjectId {
        ProjectId::Mesa
    }

    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = if let Ok(path) = std::env::var("N2S_MESA_PATH") {
            PathBuf::from(path)
        } else {
            self.get_id().android_path(ctx)
        };
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;
        self.build_path = ctx.temp_path.join(self.get_id().str());

        let mesa_test_path = ctx.test_path.join(self.get_id().str());
        let intel_clc_path = if !ctx.skip_build {
            let intel_clc_build_path = ctx.temp_path.join("intel_clc");
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(mesa_test_path.join("build_intel_clc.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&intel_clc_build_path)
                ]
            )?;
            intel_clc_build_path.join("src/intel/compiler")
        } else {
            mesa_test_path.clone()
        };

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(&mesa_test_path.join("gen-ninja.sh")),
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

        let targets = parse_build_ninja::<ninja_target::meson::MesonNinjaTarget>(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "docs/license.rst",
        );
        let targets_to_generate = TARGETS.iter().map(|target| PathBuf::from(target)).collect();
        package.generate(targets_to_generate, targets, self)?;

        let gen_deps = package.get_gen_deps();
        let mut gen_deps_folders = HashSet::new();
        for gen_dep in &gen_deps {
            let mut path = gen_dep.clone();
            while let Some(parent) = path.parent() {
                path = PathBuf::from(parent);
                gen_deps_folders.insert(path.clone());
            }
        }
        for module in package.get_modules() {
            module.update_prop("local_include_dirs", |prop| match prop {
                SoongProp::VecStr(dirs) => SoongProp::VecStr(
                    dirs.into_iter()
                        .filter(|dir| {
                            if let Ok(strip) = Path::new(&dir).strip_prefix(MESON_GENERATED) {
                                if !gen_deps_folders.contains(strip) {
                                    return false;
                                }
                            }
                            return true;
                        })
                        .collect(),
                ),
                _ => prop,
            });
        }

        let mut gen_deps_sorted = Vec::from_iter(gen_deps);
        gen_deps_sorted.sort();
        write_file(
            &ctx.test_path
                .join(self.get_id().str())
                .join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps_sorted),
        )?;

        if ctx.copy_to_aosp {
            let meson_generated_path = self.get_id().android_path(ctx).join(MESON_GENERATED);
            for dep in gen_deps_sorted {
                let from = self.build_path.join(&dep);
                let to = meson_generated_path.join(&dep);
                let to_path = to.parent().unwrap();
                if let Err(err) = std::fs::create_dir_all(to_path) {
                    return error!("create_dir_all({to_path:#?}) failed: {err}");
                }
                copy_file(&from, &to)?;
            }
            print_verbose!(
                "Files copied from {0:#?} to {meson_generated_path:#?}",
                &self.build_path
            );
        }

        Ok(package)
    }

    fn get_default_cflags(&self, target: &str) -> Vec<String> {
        let mut cflags = vec!["-Wno-non-virtual-dtor", "-Wno-error"];
        if target.ends_with("libvulkan_lite_runtime_a") {
            cflags.push("-Wno-unreachable-code-loop-increment");
        }
        cflags.into_iter().map(|flag| String::from(flag)).collect()
    }

    fn get_define(&self, define: &str) -> String {
        define
            .replace(&path_to_string(&self.build_path), MESON_GENERATED)
            .replace(&path_to_string_with_separator(&self.src_path), "")
    }

    fn get_include(&self, include: &Path) -> PathBuf {
        Path::new(MESON_GENERATED).join(strip_prefix(include, &self.build_path))
    }

    fn get_library_module(&self, module: &mut SoongModule) {
        module.add_prop(
            "arch",
            SoongNamedProp::new_prop(
                "x86",
                SoongNamedProp::new_prop("enabled", SoongProp::Bool(false)),
            ),
        );
    }

    fn get_library_name(&self, library: &Path) -> PathBuf {
        let file_name = file_name(library);
        if file_name == "libdrm.so" {
            PathBuf::from("libdrm")
        } else if file_name == "libglapi.so.0.0.0" {
            PathBuf::from("libglapi")
        } else if file_name == "libgallium_dri.so" {
            PathBuf::from("libgallium_dri")
        } else if library.starts_with("src/android_stub") {
            PathBuf::from(file_stem(library))
        } else if library.starts_with("src") || library.starts_with("subprojects") {
            Path::new(self.get_id().str()).join(library)
        } else {
            PathBuf::from(library)
        }
    }

    fn get_shared_libs(&self, target: &str) -> Vec<String> {
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
            || target == "mesa_src_vulkan_runtime_libvulkan_lite_runtime_a"
        {
            libs.push("libnativewindow");
        }
        libs.into_iter().map(|lib| String::from(lib)).collect()
    }

    fn get_source(&self, source: &Path) -> PathBuf {
        if let Ok(strip) = source.strip_prefix(&self.build_path) {
            self.src_path.join(MESON_GENERATED).join(strip)
        } else {
            PathBuf::from(source)
        }
    }

    fn get_target_alias(&self, target: &str) -> Option<String> {
        for target_str in TARGETS {
            let target_path = Path::new(target_str);
            if target == path_to_id(Path::new("mesa").join(target_path)) {
                return Some(file_stem(target_path));
            }
        }
        None
    }

    fn get_target_header_libs(&self, target: &str) -> Vec<String> {
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
        libs.into_iter().map(|lib| String::from(lib)).collect()
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
