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
    fn ignore_path(&self, src: &Path) -> bool {
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
                return true;
            }
        }
        if src.starts_with(self.build_path.join("gen")) || src.starts_with("gen") {
            return true;
        }
        false
    }
}

impl Project for Angle {
    fn get_id(&self) -> ProjectId {
        ProjectId::Angle
    }

    fn generate_package(
        &mut self,
        _android_path: &Path,
        temp_path: &Path,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        const ANGLE_PATH: &str = "N2S_ANGLE_PATH";
        let Ok(angle_path) = std::env::var(ANGLE_PATH) else {
            return error!("{ANGLE_PATH} required but not defined");
        };
        self.src_path = PathBuf::from(angle_path);
        self.build_path = temp_path.join(self.get_id().str());
        self.ndk_path = self.src_path.join("third_party/android_toolchain/ndk");

        let gn_args = vec![
            "is_component_build=false",
            "is_debug=false",
            "dcheck_always_on=false",
            "symbol_level=0",
            "angle_standalone=false",
            "angle_build_all=false",
            "angle_expose_non_conformant_extensions_and_versions=true",
            // Target ndk API 26 to make sure ANGLE can use the Vulkan backend on Android
            "android32_ndk_api_level=26",
            "android64_ndk_api_level=26",
            // Disable all backends except Vulkan
            "angle_enable_vulkan=true",
            "angle_enable_gl=false",
            "angle_enable_d3d9=false",
            "angle_enable_d3d11=false",
            "angle_enable_null=false",
            "angle_enable_metal=false",
            "angle_enable_wgpu=false",
            // SwiftShader is loaded as the system Vulkan driver on Android, not compiled by ANGLE
            "angle_enable_swiftshader=false",
            // Disable all shader translator targets except desktop GL (for Vulkan)
            "angle_enable_essl=false",
            "angle_enable_glsl=false",
            "angle_enable_hlsl=false",
            "angle_enable_commit_id=false",
            // Disable histogram/protobuf support
            "angle_has_histograms=false",
            // Use system lib(std)c++, since the Chromium library breaks std::string
            "use_custom_libcxx=false",
            // rapidJSON is used for ANGLE's frame capture (among other things), which is unnecessary for AOSP builds.
            "angle_has_rapidjson=false",
            // TODO(b/279980674): re-enable end2end tests
            "build_angle_end2end_tests_aosp=true",
            "build_angle_trace_tests=false",
            "angle_test_enable_system_egl=true",
        ];

        let targets = ninja_target::gn::get_targets(&self.src_path, &self.build_path, gn_args)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        let mut targets_to_generate = Vec::new();
        for target in TARGETS {
            targets_to_generate.push(PathBuf::from(target));
        }
        package.generate(targets_to_generate, targets, self)?;

        Ok(package)
    }

    fn get_cmd_output(&self, output: &Path) -> PathBuf {
        PathBuf::from(file_name(output))
    }

    fn get_default_cflags(&self, _target: &str) -> Vec<String> {
        vec!["-Wno-nullability-completeness".to_string()]
    }

    fn get_library_module(&self, module: &mut SoongModule) {
        module.add_prop("stl", SoongProp::Str("libc++_static".to_string()));
        module.add_prop(
            "arch",
            SoongNamedProp::new_prop(
                "arm64",
                SoongNamedProp::new_prop(
                    "cflags",
                    SoongProp::VecStr(vec!["-D__ARM_NEON__=1".to_string()]),
                ),
            ),
        );
    }

    fn get_library_name(&self, library: &Path) -> PathBuf {
        if library.starts_with("obj/third_party/spirv-tools") {
            Path::new(ProjectId::SpirvTools.str()).join("source/libSPIRV-Tools.a")
        } else if library.starts_with("obj/third_party/zlib") {
            PathBuf::from("zlib_google_compression_utils_portable")
        } else if library.starts_with("obj/third_party/cpu_features") {
            PathBuf::from("cpufeatures")
        } else if library.starts_with("obj") {
            Path::new(self.get_id().str()).join(library)
        } else {
            for str in TARGETS {
                if "./".to_string() + str + "_angle.so" == path_to_string(library) {
                    return PathBuf::from("angle/./".to_string() + str + "_angle.so");
                }
            }
            library.to_path_buf()
        }
    }

    fn get_shared_libs(&self, target: &str) -> Vec<String> {
        if target.starts_with("angle_obj_lib") {
            return vec!["libnativewindow".to_string()];
        } else if target.starts_with("angle___libGLESv2_angle_so") {
            return vec!["libz".to_string()];
        }
        Vec::new()
    }

    fn get_target_alias(&self, target: &str) -> Option<String> {
        for str in TARGETS {
            if "angle___".to_string() + str + "_angle_so" == target {
                return Some(str.to_string() + "_angle");
            }
        }
        None
    }

    fn get_target_header_libs(&self, target: &str) -> Vec<String> {
        if target == "angle_obj_libtranslator_a" {
            vec![
                CcLibraryHeaders::SpirvHeaders.str(),
                CcLibraryHeaders::SpirvTools.str(),
            ]
        } else {
            Vec::new()
        }
    }

    fn ignore_cflag(&self, _cflag: &str) -> bool {
        !(_cflag.starts_with("-fvisibility"))
    }

    fn ignore_define(&self, define: &str) -> bool {
        define.contains("__ARM_NEON__")
    }

    fn ignore_gen_header(&self, header: &Path) -> bool {
        !header.starts_with("gen/angle")
    }

    fn ignore_include(&self, include: &Path) -> bool {
        self.ignore_path(include)
    }

    fn ignore_lib(&self, lib: &str) -> bool {
        lib.contains("llvm-build/Release+Asserts")
    }

    fn ignore_link_flag(&self, _flag: &str) -> bool {
        true
    }

    fn ignore_source(&self, source: &Path) -> bool {
        self.ignore_path(source)
    }

    fn ignore_target(&self, target: &Path) -> bool {
        if target.starts_with("obj/third_party")
            || target.starts_with("gen/third_party/spirv-tools")
        {
            return true;
        }
        false
    }
}
