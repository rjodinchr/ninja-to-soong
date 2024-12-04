// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_package::SoongPackage;

#[derive(Default)]
pub struct Clvk {
    src_dir: String,
    build_dir: String,
    ndk_dir: String,
    clspv_dir: String,
    llvm_project_dir: String,
    spirv_tools_dir: String,
    spirv_headers_dir: String,
    generated_libraries: HashSet<String>,
}

impl Project for Clvk {
    fn init(&mut self, android_dir: &str, ndk_dir: &str, temp_dir: &str) {
        self.src_dir = self.get_id().android_path(android_dir);
        self.build_dir = add_slash_suffix(temp_dir) + self.get_id().str();
        self.ndk_dir = ndk_dir.to_string();
        self.clspv_dir = ProjectId::Clspv.android_path(android_dir);
        self.spirv_headers_dir = ProjectId::SpirvHeaders.android_path(android_dir);
        self.spirv_tools_dir = ProjectId::SpirvTools.android_path(android_dir);
        self.llvm_project_dir = ProjectId::LlvmProject.android_path(android_dir);
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::Clvk
    }

    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            &self.src_dir,
            &self.ndk_dir,
            &self.build_dir,
            self.get_id().str(),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(vec!["libOpenCL.so".to_string()], targets, self)?;

        self.generated_libraries = package.get_generated_libraries();
        Ok(package)
    }

    fn get_ninja_file_path(
        &mut self,
        _projects_map: &ProjectsMap,
    ) -> Result<Option<String>, String> {
        let spirv_headers_dir = "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + &self.spirv_headers_dir;
        let spirv_tools_dir = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string() + &self.spirv_tools_dir;
        let clspv_dir = "-DCLSPV_SOURCE_DIR=".to_string() + &self.clspv_dir;
        let llvm_dir = "-DCLSPV_LLVM_SOURCE_DIR=".to_string() + &self.llvm_project_dir + "/llvm";
        let clang_dir = "-DCLSPV_CLANG_SOURCE_DIR=".to_string() + &self.llvm_project_dir + "/clang";
        let libclc_dir =
            "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string() + &self.llvm_project_dir + "/libclc";
        let vulkan_library = "-DVulkan_LIBRARY=".to_string()
            + &self.ndk_dir
            + "/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/"
            + ANDROID_ISA
            + "-linux-android/"
            + ANDROID_PLATFORM
            + "/libvulkan.so";
        let (ninja_file_path, _) = cmake_configure(
            &self.src_dir,
            &self.build_dir,
            &self.ndk_dir,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DCLVK_CLSPV_ONLINE_COMPILER=1",
                "-DCLVK_ENABLE_SPIRV_IL=OFF",
                "-DCLVK_BUILD_TESTS=OFF",
                &spirv_headers_dir,
                &spirv_tools_dir,
                &clspv_dir,
                &llvm_dir,
                &clang_dir,
                &libclc_dir,
                &vulkan_library,
            ],
        )?;
        Ok(Some(ninja_file_path))
    }

    fn get_gen_deps(&self, project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        let mut libs: HashSet<String> = HashSet::new();
        let prefix = add_slash_suffix(project.str());
        for library in &self.generated_libraries {
            if let Some(lib) = self.get_library_name(library).strip_prefix(&prefix) {
                libs.insert(lib.to_string());
            }
        }
        deps.insert(GenDeps::TargetsToGenerate, libs);
        deps
    }

    fn get_library_name(&self, library: &str) -> String {
        library
            .replace(
                "external/clspv/third_party/llvm",
                ProjectId::LlvmProject.str(),
            )
            .replace("external/", "")
    }

    fn get_target_header_libs(&self, _target: &str) -> HashSet<String> {
        [
            CC_LIBRARY_HEADERS_SPIRV_TOOLS.to_string(),
            CC_LIBRARY_HEADERS_SPIRV_HEADERS.to_string(),
            CC_LIBRARY_HEADERS_CLSPV.to_string(),
            "OpenCL-Headers".to_string(),
        ]
        .into()
    }

    fn get_target_alias(&self, target: &str) -> Option<String> {
        if target == "clvk_libOpenCL_so" {
            Some("libclvk".to_string())
        } else {
            None
        }
    }

    fn ignore_gen_header(&self, _header: &str) -> bool {
        true
    }

    fn ignore_include(&self, _include: &str) -> bool {
        true
    }

    fn ignore_link_flag(&self, flag: &str) -> bool {
        flag != "-Wl,-Bsymbolic"
    }

    fn ignore_target(&self, target: &str) -> bool {
        target.starts_with("external/")
    }
}
