// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;

#[derive(Default)]
pub struct Clvk {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    generated_libraries: Vec<PathBuf>,
}

impl Project for Clvk {
    fn get_id(&self) -> ProjectId {
        ProjectId::Clvk
    }

    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_id().android_path(ctx)?;
        self.build_path = ctx.temp_path.join(self.get_id().str());
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;

        let llvm_project_path = ProjectId::LlvmProject.android_path(ctx)?;
        let spirv_headers_path = "-DSPIRV_HEADERS_SOURCE_DIR=".to_string()
            + &path_to_string(ProjectId::SpirvHeaders.android_path(ctx)?);
        let spirv_tools_path = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string()
            + &path_to_string(ProjectId::SpirvTools.android_path(ctx)?);
        let clspv_path = "-DCLSPV_SOURCE_DIR=".to_string()
            + &path_to_string(ProjectId::Clspv.android_path(ctx)?);
        let llvm_path = "-DCLSPV_LLVM_SOURCE_DIR=".to_string()
            + &path_to_string(llvm_project_path.join("llvm"));
        let clang_path = "-DCLSPV_CLANG_SOURCE_DIR=".to_string()
            + &path_to_string(llvm_project_path.join("clang"));
        let libclc_path = "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string()
            + &path_to_string(llvm_project_path.join("libclc"));
        let vulkan_library = "-DVulkan_LIBRARY=".to_string()
            + &path_to_string(
                self.ndk_path
                    .join("toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib")
                    .join(ANDROID_ISA.to_string() + "-linux-android")
                    .join(ANDROID_PLATFORM)
                    .join("libvulkan.so"),
            );

        let (targets, _) = ninja_target::cmake::get_targets(
            &self.src_path,
            &self.build_path,
            &self.ndk_path,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DCLVK_CLSPV_ONLINE_COMPILER=1",
                "-DCLVK_ENABLE_SPIRV_IL=OFF",
                "-DCLVK_BUILD_TESTS=OFF",
                &spirv_headers_path,
                &spirv_tools_path,
                &clspv_path,
                &llvm_path,
                &clang_path,
                &libclc_path,
                &vulkan_library,
            ],
            None,
            ctx,
        )?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(vec![PathBuf::from("libOpenCL.so")], targets, self)?;

        self.generated_libraries = Vec::from_iter(package.get_generated_libraries());
        Ok(package)
    }

    fn get_gen_deps(&self, project: ProjectId) -> GenDepsMap {
        let mut deps = HashMap::new();
        let mut libs = Vec::new();
        let prefix = project.str();
        for library in &self.generated_libraries {
            let library_path = PathBuf::from(library);
            if let Ok(lib) = self.get_library_name(&library_path).strip_prefix(prefix) {
                libs.push(lib.to_path_buf());
            }
        }
        deps.insert(GenDeps::TargetsToGen, libs);
        deps
    }

    fn get_library_name(&self, library: &Path) -> PathBuf {
        let strip =
            if let Ok(strip) = library.strip_prefix(Path::new("external/clspv/third_party/llvm")) {
                Path::new(ProjectId::LlvmProject.str()).join(strip)
            } else {
                library.to_path_buf()
            };
        strip_prefix(&strip, "external")
    }

    fn get_target_header_libs(&self, _target: &str) -> Vec<String> {
        vec!["OpenCL-Headers".to_string()]
    }

    fn get_target_alias(&self, target: &str) -> Option<String> {
        if target == "clvk_libOpenCL_so" {
            Some("libclvk".to_string())
        } else {
            None
        }
    }

    fn ignore_cflag(&self, _cflag: &str) -> bool {
        true
    }

    fn ignore_gen_header(&self, _header: &Path) -> bool {
        true
    }

    fn ignore_include(&self, _include: &Path) -> bool {
        true
    }

    fn ignore_lib(&self, lib: &str) -> bool {
        lib == "libatomic"
    }

    fn ignore_link_flag(&self, flag: &str) -> bool {
        flag != "-Wl,-Bsymbolic"
    }

    fn ignore_target(&self, target: &Path) -> bool {
        target.starts_with("external")
    }
}
