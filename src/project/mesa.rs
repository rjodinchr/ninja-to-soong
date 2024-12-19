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
        self.src_path = self.get_id().android_path(ctx);
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;
        self.build_path = ctx.temp_path.join(self.get_id().str());

        let meson_local_path = match std::env::var("HOME") {
            Ok(home) => Path::new(&home).join(".local/share/meson/cross"),
            Err(err) => return error!("Could not get HOME env: {err}"),
        };
        if let Err(err) = std::fs::create_dir_all(&meson_local_path) {
            return error!("Could not create {meson_local_path:#?}: {err:#?}");
        }
        const AOSP_X86_64_TEMPLATE: &str = "aosp-x86_64";
        let aosp_x86_64_template_path = meson_local_path.join(AOSP_X86_64_TEMPLATE);
        write_file(
            &aosp_x86_64_template_path,
            &read_file(
                &ctx.test_path
                    .join(self.get_id().str())
                    .join("aosp-x86_64.template"),
            )?
            .replace("${NDK_PATH}", &path_to_string(&self.ndk_path))
            .replace("${ANDROID_PLATFORM}", ANDROID_PLATFORM),
        )?;
        print_verbose!("{aosp_x86_64_template_path:#?} created");

        let new_path = format!(
            "{0}:{1}",
            if !ctx.skip_build {
                print_verbose!("Building intel_clc...");
                let intel_clc_path = ctx.temp_path.join("intel_clc");
                ninja_target::meson::meson_setup(
                    &self.src_path,
                    &intel_clc_path,
                    vec![
                        "-Dplatforms=",
                        "-Dglx=disabled",
                        "-Dtools=",
                        "-Dbuild-tests=false",
                        "-Dvulkan-drivers=",
                        "-Dgallium-drivers=",
                        "-Dgallium-rusticl=false",
                        "-Dgallium-va=auto",
                        "-Dgallium-xa=disabled",
                        "-Dbuildtype=release",
                        "-Dintel-clc=enabled",
                        "-Dstrip=true",
                        "--reconfigure",
                        "--wipe",
                    ],
                    None,
                )?;
                ninja_target::meson::meson_compile(&intel_clc_path)?;
                path_to_string(intel_clc_path.join("src/intel/compiler"))
            } else {
                path_to_string(&ctx.test_path.join(self.get_id().str()))
            },
            match std::env::var("PATH") {
                Ok(env) => env,
                Err(err) => return error!("Could not get PATH env: {err}"),
            }
        );

        let targets = ninja_target::meson::get_targets(
            &self.src_path,
            &self.build_path,
            vec![
                "--cross-file",
                AOSP_X86_64_TEMPLATE,
                "--libdir",
                "lib64",
                "--sysconfdir=/system/vendor/etc",
                "-Ddri-search-path=/system/lib64/dri:/system/vendor/lib64/dri",
                "-Dllvm=disabled",
                "-Ddri3=disabled",
                "-Dglx=disabled",
                "-Dgbm=disabled",
                "-Degl=enabled",
                &format!("-Dplatform-sdk-version={ANDROID_PLATFORM}"),
                "-Dandroid-stub=true",
                "-Dplatforms=android",
                "-Dperfetto=true",
                "-Degl-lib-suffix=_mesa",
                "-Dgles-lib-suffix=_mesa",
                "-Dcpp_rtti=false",
                "-Dtools=",
                "-Dvulkan-drivers=intel",
                "-Dgallium-drivers=iris",
                "-Dgallium-rusticl=false",
                "-Dgallium-va=disabled",
                "-Dgallium-xa=disabled",
                "-Dbuildtype=release",
                "-Dintel-clc=system",
                "-Dintel-rt=enabled",
                "-Dstrip=true",
                "--reconfigure",
                "--wipe",
            ],
            Some(vec![("PATH", new_path.as_str())]),
            ctx,
        )?;

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
            let _ = module.update_prop("local_include_dirs", |prop| match prop {
                SoongProp::VecStr(dirs) => {
                    let mut new_dirs = Vec::new();
                    for dir in dirs {
                        if let Ok(strip) = Path::new(&dir).strip_prefix(MESON_GENERATED) {
                            if !gen_deps_folders.contains(strip) {
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
            &ctx.test_path
                .join(self.get_id().str())
                .join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps_sorted),
        )?;

        if !ctx.skip_build {
            let meson_generated_path = self.src_path.join(MESON_GENERATED);
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
            .replace(
                &format!(
                    "{0}{1}",
                    path_to_string(&self.src_path),
                    std::path::MAIN_SEPARATOR
                ),
                "",
            )
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

    fn ignore_cflag(&self, cflag: &str) -> bool {
        cflag != "-mclflushopt"
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
