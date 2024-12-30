// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

const TARGETS: [&'static str; 3] = ["libEGL", "libGLESv2", "libGLESv1_CM"];

#[derive(Default)]
pub struct Angle {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
}

impl Angle {
    fn filter_path(&self, src: &Path) -> bool {
        for ignore_path in [
            "buildtools",
            "third_party/cpu_features",
            "third_party/libc++",
            "third_party/libc++abi",
            "third_party/libunwind",
            "third_party/llvm-libc",
            "third_party/spirv-headers",
            "third_party/spirv-tools",
            "third_party/zlib",
        ] {
            if src.starts_with(self.src_path.join(ignore_path)) {
                return false;
            }
        }
        if src.starts_with(self.build_path.join("gen")) || src.starts_with("gen") {
            return false;
        }
        true
    }
}

impl Project for Angle {
    fn get_name(&self) -> &'static str {
        "angle"
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
        self.src_path = if let Ok(path) = std::env::var("N2S_ANGLE_PATH") {
            PathBuf::from(path)
        } else {
            self.get_android_path(ctx)
        };
        self.build_path = ctx.temp_path.join(self.get_name());
        self.ndk_path = self.src_path.join("third_party/android_toolchain/ndk");

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                    ANDROID_CPU,
                ]
            )?;
        }
        let targets = parse_build_ninja::<GnNinjaTarget>(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            "//visibility:public",
            "angle_license",
            vec!["SPDX-license-identifier-Apache-2.0"],
            vec!["LICENSE"],
        );
        let targets_to_generate = TARGETS
            .into_iter()
            .map(|target| PathBuf::from(target))
            .collect();
        package.generate(targets_to_generate, targets, self)?;

        Ok(package)
    }

    fn get_target_name(&self, target: &Path) -> PathBuf {
        for str in TARGETS {
            if target.ends_with(format!("{str}_angle.so")) {
                return PathBuf::from(file_stem(target));
            }
        }
        PathBuf::from(target)
    }
    fn get_target_module(&self, target: &Path, mut module: SoongModule) -> SoongModule {
        if target.ends_with("libtranslator.a") {
            module.add_prop(
                "header_libs",
                SoongProp::VecStr(vec![
                    CcLibraryHeaders::SpirvHeaders.str(),
                    CcLibraryHeaders::SpirvTools.str(),
                ]),
            );
        }
        module.add_prop("stl", SoongProp::Str(String::from("libc++_static")));
        module.add_prop(
            "arch",
            SoongNamedProp::new_prop(
                "arm64",
                SoongNamedProp::new_prop(
                    "cflags",
                    SoongProp::VecStr(vec![String::from("-D__ARM_NEON__=1")]),
                ),
            ),
        );
        module
    }

    fn extend_cflags(&self, _target: &Path) -> Vec<String> {
        vec![String::from("-Wno-nullability-completeness")]
    }
    fn extend_shared_libs(&self, target: &Path) -> Vec<String> {
        if target.starts_with("angle/obj") {
            vec![String::from("libnativewindow")]
        } else if target.ends_with("libGLESv2_angle.so") {
            vec![String::from("libz")]
        } else {
            Vec::new()
        }
    }

    fn map_cmd_output(&self, output: &Path) -> PathBuf {
        PathBuf::from(file_name(output))
    }
    fn map_lib(&self, library: &Path) -> PathBuf {
        if library.starts_with("obj/third_party/spirv-tools") {
            PathBuf::from("SPIRV-Tools/source/libSPIRV-Tools.a")
        } else if library.starts_with("obj/third_party/zlib") {
            PathBuf::from("zlib_google_compression_utils_portable")
        } else if library.starts_with("obj/third_party/cpu_features") {
            PathBuf::from("cpufeatures")
        } else if library.starts_with("obj") {
            Path::new(self.get_name()).join(library)
        } else {
            PathBuf::from(library)
        }
    }

    fn filter_cflag(&self, cflag: &str) -> bool {
        cflag.starts_with("-fvisibility")
    }
    fn filter_define(&self, define: &str) -> bool {
        !define.contains("__ARM_NEON__")
    }
    fn filter_gen_header(&self, header: &Path) -> bool {
        header.starts_with("gen/angle")
    }
    fn filter_include(&self, include: &Path) -> bool {
        self.filter_path(include)
    }
    fn filter_lib(&self, lib: &str) -> bool {
        !lib.contains("llvm-build/Release+Asserts")
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
    fn filter_source(&self, source: &Path) -> bool {
        self.filter_path(source)
    }
    fn filter_target(&self, target: &Path) -> bool {
        !(target.starts_with("obj/third_party")
            || target.starts_with("gen/third_party/spirv-tools"))
    }
}
