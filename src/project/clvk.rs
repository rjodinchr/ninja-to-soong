// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_package::SoongPackage;

pub struct CLVK<'a> {
    src_root: &'a str,
    build_root: String,
    ndk_root: &'a str,
    clspv_root: &'a str,
    llvm_project_root: &'a str,
    spirv_tools_root: &'a str,
    spirv_headers_root: &'a str,
    generated_libraries: HashSet<String>,
}

const CLVK_PROJECT_ID: ProjectId = ProjectId::CLVK;
const CLVK_PROJECT_NAME: &str = CLVK_PROJECT_ID.str();

impl<'a> CLVK<'a> {
    pub fn new(
        temp_dir: &'a str,
        ndk_root: &'a str,
        clvk_root: &'a str,
        clspv_root: &'a str,
        llvm_project_root: &'a str,
        spirv_tools_root: &'a str,
        spirv_headers_root: &'a str,
    ) -> Self {
        CLVK {
            src_root: clvk_root,
            build_root: add_slash_suffix(temp_dir) + CLVK_PROJECT_NAME,
            ndk_root,
            clspv_root,
            llvm_project_root,
            spirv_tools_root,
            spirv_headers_root,
            generated_libraries: HashSet::new(),
        }
    }
}

impl<'a> crate::project::Project<'a> for CLVK<'a> {
    fn get_id(&self) -> ProjectId {
        CLVK_PROJECT_ID
    }

    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        _project_map: &ProjectMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            &self.build_root,
            CLVK_PROJECT_NAME,
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(vec!["libOpenCL.so".to_string()], targets, self)?;

        self.generated_libraries = package.get_generated_libraries();
        Ok(package)
    }

    fn get_build_directory(&mut self, _project_map: &ProjectMap) -> Result<Option<String>, String> {
        let spirv_headers_dir = "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + self.spirv_headers_root;
        let spirv_tools_dir = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string() + self.spirv_tools_root;
        let clspv_dir = "-DCLSPV_SOURCE_DIR=".to_string() + self.clspv_root;
        let llvm_project_dir = self.llvm_project_root;
        let llvm_dir = "-DCLSPV_LLVM_SOURCE_DIR=".to_string() + &llvm_project_dir + "/llvm";
        let clang_dir = "-DCLSPV_CLANG_SOURCE_DIR=".to_string() + &llvm_project_dir + "/clang";
        let libclc_dir = "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string() + &llvm_project_dir + "/libclc";
        let vulkan_library = "-DVulkan_LIBRARY=".to_string()
            + self.ndk_root
            + "/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/"
            + ANDROID_ISA
            + "-linux-android/"
            + ANDROID_PLATFORM
            + "/libvulkan.so";
        cmake_configure(
            self.src_root,
            &self.build_root,
            self.ndk_root,
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
        Ok(Some(self.build_root.clone()))
    }

    fn ignore_target(&self, target: &String) -> bool {
        target.starts_with("external/")
    }

    fn ignore_include(&self, _include: &str) -> bool {
        true
    }

    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [
            CC_LIB_HEADERS_SPIRV_TOOLS.to_string(),
            CC_LIB_HEADERS_SPIRV_HEADERS.to_string(),
            CC_LIB_HEADERS_CLSPV.to_string(),
            "OpenCL-Headers".to_string(),
        ]
        .into()
    }

    fn get_library_name(&self, library: &str) -> String {
        library
            .replace("external/clspv/third_party/llvm", "llvm-project")
            .replace("external/", "")
            .replace("/", "_")
            .replace(".", "_")
    }

    fn handle_link_flag(&self, flag: &str, link_flags: &mut HashSet<String>) {
        if flag == "-Wl,-Bsymbolic" {
            link_flags.insert(flag.to_string());
        }
    }

    fn get_target_stem(&self, target: &String) -> String {
        if target == "clvk_libOpenCL_so" {
            "libclvk".to_string()
        } else {
            String::new()
        }
    }

    fn get_generated_deps(&self, project: ProjectId) -> ProjectDeps {
        let mut deps: ProjectDeps = HashMap::new();
        let mut libs: HashSet<String> = HashSet::new();
        for library in &self.generated_libraries {
            let prefix = if project == ProjectId::LLVM {
                "external/clspv/third_party/llvm/".to_string()
            } else {
                "external/".to_string() + &add_slash_suffix(project.str())
            };
            if let Some(lib) = library.strip_prefix(&prefix) {
                libs.insert(lib.to_string());
            }
        }
        deps.insert(Dependency::TargetToGenerate, libs);
        deps
    }
}
