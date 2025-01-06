// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

const TARGETS: [NinjaTargetToGen; 3] = [
    NinjaTargetToGen("./libEGL_angle.so", Some("libEGL_angle"), None),
    NinjaTargetToGen("./libGLESv2_angle.so", Some("libGLESv2_angle"), None),
    NinjaTargetToGen("./libGLESv1_CM_angle.so", Some("libGLESv1_CM_angle"), None),
];

#[derive(Default)]
pub struct Angle {
    src_path: PathBuf,
    build_path: PathBuf,
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
    ) -> Result<String, String> {
        self.src_path = if let Ok(path) = std::env::var("N2S_ANGLE_PATH") {
            PathBuf::from(path)
        } else {
            self.get_android_path(ctx)
        };
        self.build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = self.src_path.join("third_party/android_toolchain/ndk");

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                ]
            )?;
        }

        Ok(SoongPackage::new(
            "//visibility:public",
            "angle_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&TARGETS),
            parse_build_ninja::<GnNinjaTarget>(&self.build_path)?,
            &self.src_path,
            &ndk_path,
            &self.build_path,
            None,
            self,
        )?
        .print())
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> SoongModule {
        if target.ends_with("libtranslator.a") {
            module = module.add_prop(
                "header_libs",
                SoongProp::VecStr(vec![
                    CcLibraryHeaders::SpirvHeaders.str(),
                    CcLibraryHeaders::SpirvTools.str(),
                ]),
            );
        }
        module
            .add_prop("stl", SoongProp::Str(String::from("libc++_static")))
            .add_prop(
                "arch",
                SoongNamedProp::new_prop(
                    "arm64",
                    SoongNamedProp::new_prop(
                        "cflags",
                        SoongProp::VecStr(vec![String::from("-D__ARM_NEON__=1")]),
                    ),
                ),
            )
    }
    fn extend_cflags(&self, _target: &Path) -> Vec<String> {
        vec![String::from("-Wno-nullability-completeness")]
    }
    fn extend_shared_libs(&self, target: &Path) -> Vec<String> {
        if target.starts_with("obj") {
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
    fn map_lib(&self, library: &Path) -> Option<PathBuf> {
        if library.starts_with("obj/third_party/spirv-tools") {
            Some(PathBuf::from("SPIRV-Tools/source/libSPIRV-Tools.a"))
        } else if library.starts_with("obj/third_party/zlib") {
            Some(PathBuf::from("zlib_google_compression_utils_portable"))
        } else if library.starts_with("obj/third_party/cpu_features") {
            Some(PathBuf::from("cpufeatures"))
        } else if library.starts_with("obj") {
            None
        } else {
            Some(PathBuf::from(library))
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
