// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use super::*;

const MESON_GENERATED: &str = "meson_generated";

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
        android_path: &Path,
        temp_path: &Path,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        const MESA_PATH: &str = "N2S_MESA_PATH";
        let Ok(mesa_path) = std::env::var(MESA_PATH) else {
            return error!("{MESA_PATH} required but not defined");
        };

        self.src_path = PathBuf::from(mesa_path);
        self.build_path = self.src_path.join("build");
        self.ndk_path = get_ndk_path(temp_path)?;

        let targets = ninja_target::meson::get_targets(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "docs/license.rst",
        );
        let mut targets_to_generate = Vec::new();
        for target in [
            "src/egl/libEGL_mesa.so.1.0.0",
            "src/mapi/es2api/libGLESv2_mesa.so.2.0.0",
            "src/mapi/es1api/libGLESv1_CM_mesa.so.1.1.0",
            "src/mapi/shared-glapi/libglapi.so.0.0.0",
            "src/gallium/targets/dri/libgallium_dri.so",
            "src/intel/vulkan/libvulkan_intel.so",
            "src/tool/pps/pps-producer",
        ] {
            targets_to_generate.push(PathBuf::from(target));
        }
        package.generate(targets_to_generate, targets, self)?;

        let gen_deps = package.get_gen_deps();
        let mut gen_deps_folders = HashSet::new();
        for gen_dep in &gen_deps {
            let mut path = gen_dep.clone();
            while let Some(parent) = path.parent() {
                path = parent.to_path_buf();
                gen_deps_folders.insert(path.clone());
            }
        }
        for module in package.get_modules() {
            let _ = module.update_prop("local_include_dirs", |prop| match prop {
                SoongProp::VecStr(dirs) => {
                    let mut new_dirs = Vec::new();
                    for dir in dirs {
                        if let Ok(strip) = Path::new(&dir).strip_prefix(MESON_GENERATED) {
                            if !gen_deps_folders.contains(&strip.to_path_buf()) {
                                continue;
                            }
                        }
                        new_dirs.push(dir);
                    }
                    Ok(SoongProp::VecStr(new_dirs))
                }
                _ => error!("Expected local_include_dirs to be a VecStr"),
            });
        }

        let mut gen_deps_sorted = Vec::from_iter(gen_deps);
        gen_deps_sorted.sort();
        write_file(
            &get_tests_folder()?
                .join(self.get_id().str())
                .join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps_sorted),
        )?;

        let meson_generated_path = self
            .get_id()
            .android_path(android_path)
            .join(MESON_GENERATED);
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

        Ok(package)
    }

    fn get_default_cflags(&self, _target: &str) -> Vec<String> {
        let mut cflags = vec![
            "-Wno-non-virtual-dtor".to_string(),
            "-Wno-error".to_string(),
        ];
        if _target == "mesa_src_vulkan_runtime_libvulkan_lite_runtime_a" {
            cflags.push("-Wno-unreachable-code-loop-increment".to_string());
        }

        cflags
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
            PathBuf::from(library.file_stem().unwrap().to_str().unwrap())
        } else if library.starts_with("src") || library.starts_with("subprojects") {
            Path::new(self.get_id().str()).join(library)
        } else {
            library.to_path_buf()
        }
    }

    fn get_shared_libs(&self, target: &str) -> Vec<String> {
        let mut libs = Vec::new();
        if target == "mesa_src_gallium_frontends_dri_libdri_a"
            || target == "mesa_src_intel_vulkan_libanv_common_a"
            || target == "mesa_src_vulkan_wsi_libvulkan_wsi_a"
            || target == "mesa_src_vulkan_runtime_libvulkan_lite_runtime_a"
        {
            libs.push("libsync".to_string());
        }
        if target == "mesa_src_util_libmesa_util_a" {
            libs.push("libz".to_string());
        }
        if target.starts_with("mesa_src_intel_vulkan_")
            || target == "mesa_src_vulkan_runtime_libvulkan_lite_runtime_a"
        {
            libs.push("libnativewindow".to_string());
        }
        libs
    }

    fn get_source(&self, source: &Path) -> PathBuf {
        if source.starts_with(&self.build_path) {
            self.src_path
                .join(MESON_GENERATED)
                .join(strip_prefix(source, &self.build_path))
        } else {
            source.to_path_buf()
        }
    }

    fn get_target_alias(&self, target: &str) -> Option<String> {
        Some(String::from(
            if target == "mesa_src_egl_libEGL_mesa_so_1_0_0" {
                "libEGL_mesa"
            } else if target == "mesa_src_mapi_es2api_libGLESv2_mesa_so_2_0_0" {
                "libGLESv2_mesa"
            } else if target == "mesa_src_mapi_es1api_libGLESv1_CM_mesa_so_1_1_0" {
                "libGLESv1_CM_mesa"
            } else if target == "mesa_src_mapi_shared-glapi_libglapi_so_0_0_0" {
                "libglapi"
            } else if target == "mesa_src_gallium_targets_dri_libgallium_dri_so" {
                "libgallium_dri"
            } else if target == "mesa_src_intel_vulkan_libvulkan_intel_so" {
                "libvulkan_intel"
            } else if target == "mesa_src_tool_pps_pps-producer" {
                "pps-producer"
            } else {
                return None;
            },
        ))
    }

    fn get_target_header_libs(&self, target: &str) -> Vec<String> {
        let mut libs = Vec::new();
        if target == "mesa_src_gallium_auxiliary_libgallium_a"
            || target == "mesa_src_gallium_auxiliary_pipe-loader_libpipe_loader_static_a"
            || target == "mesa_src_gallium_drivers_iris_libiris_a"
            || target == "mesa_src_gallium_frontends_dri_libdri_a"
            || target == "mesa_src_gallium_winsys_sw_kms-dri_libswkmsdri_a"
            || target == "mesa_src_intel_dev_libintel_dev_a"
            || target == "mesa_src_intel_vulkan_libanv_common_a"
            || target == "mesa_src_loader_libloader_a"
            || target == "mesa_src_util_libmesa_util_a"
            || target == "mesa_src_tool_pps_libpps_a"
            || target == "mesa_src_vulkan_runtime_libvulkan_instance_a"
            || target == "mesa_src_vulkan_runtime_libvulkan_lite_runtime_a"
            || target == "mesa_src_vulkan_wsi_libvulkan_wsi_a"
        {
            libs.push("libdrm_headers".to_string());
        }
        if target == "mesa_src_intel_vulkan_libanv_common_a" {
            libs.push("hwvulkan_headers".to_string());
        }
        libs
    }

    fn ignore_cflag(&self, cflag: &str) -> bool {
        if cflag == "-mclflushopt" {
            false
        } else {
            true
        }
    }

    fn ignore_define(&self, define: &str) -> bool {
        define == "WITH_LIBBACKTRACE" // b/120606663
    }

    fn ignore_include(&self, include: &Path) -> bool {
        path_to_string(include)
            .starts_with(&path_to_string(self.src_path.join("subprojects/libdrm")))
            || include.ends_with("android_stub")
    }

    fn ignore_link_flag(&self, _flag: &str) -> bool {
        true
    }

    fn ignore_gen_header(&self, _header: &Path) -> bool {
        true
    }

    fn ignore_lib(&self, lib: &str) -> bool {
        lib.contains("libbacktrace")
    }

    fn ignore_target(&self, target: &Path) -> bool {
        let file_name = file_name(target);
        !(file_name.contains(".so")
            || file_name.contains(".a")
            || file_name.contains("pps-producer"))
            || file_name.contains("libdrm")
            || target.starts_with("src/android_stub")
    }
}
