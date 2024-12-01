// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_package::SoongPackage;

pub struct Clvk<'a> {
    src_dir: &'a str,
    build_dir: String,
    ndk_dir: &'a str,
    clspv_dir: &'a str,
    llvm_project_dir: &'a str,
    spirv_tools_dir: &'a str,
    spirv_headers_dir: &'a str,
    generated_libraries: HashSet<String>,
}

impl<'a> Clvk<'a> {
    pub fn new(
        temp_dir: &'a str,
        ndk_dir: &'a str,
        clvk_dir: &'a str,
        clspv_dir: &'a str,
        llvm_project_dir: &'a str,
        spirv_tools_dir: &'a str,
        spirv_headers_dir: &'a str,
    ) -> Self {
        Clvk {
            src_dir: clvk_dir,
            build_dir: add_slash_suffix(temp_dir) + ProjectId::Clvk.str(),
            ndk_dir,
            clspv_dir,
            llvm_project_dir,
            spirv_tools_dir,
            spirv_headers_dir,
            generated_libraries: HashSet::new(),
        }
    }
}

impl<'a> crate::project::Project<'a> for Clvk<'a> {
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        _projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_dir,
            self.ndk_dir,
            &self.build_dir,
            ProjectId::Clvk.str(),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(vec!["libOpenCL.so".to_string()], targets, self)?;

        self.generated_libraries = package.get_generated_libraries();
        Ok(package)
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::Clvk
    }

    fn get_build_dir(&mut self, _projects_map: &ProjectsMap) -> Result<Option<String>, String> {
        let spirv_headers_dir = "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + self.spirv_headers_dir;
        let spirv_tools_dir = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string() + self.spirv_tools_dir;
        let clspv_dir = "-DCLSPV_SOURCE_DIR=".to_string() + self.clspv_dir;
        let llvm_project_dir = self.llvm_project_dir;
        let llvm_dir = "-DCLSPV_LLVM_SOURCE_DIR=".to_string() + &llvm_project_dir + "/llvm";
        let clang_dir = "-DCLSPV_CLANG_SOURCE_DIR=".to_string() + &llvm_project_dir + "/clang";
        let libclc_dir = "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string() + &llvm_project_dir + "/libclc";
        let vulkan_library = "-DVulkan_LIBRARY=".to_string()
            + self.ndk_dir
            + "/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/"
            + ANDROID_ISA
            + "-linux-android/"
            + ANDROID_PLATFORM
            + "/libvulkan.so";
        cmake_configure(
            self.src_dir,
            &self.build_dir,
            self.ndk_dir,
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
        Ok(Some(self.build_dir.clone()))
    }

    fn get_generated_deps(&self, project: ProjectId) -> DepsMap {
        let mut deps: DepsMap = HashMap::new();
        let mut libs: HashSet<String> = HashSet::new();
        for library in &self.generated_libraries {
            let prefix = if project == ProjectId::LlvmProject {
                "external/clspv/third_party/llvm/".to_string()
            } else {
                "external/".to_string() + &add_slash_suffix(project.str())
            };
            if let Some(lib) = library.strip_prefix(&prefix) {
                libs.insert(lib.to_string());
            }
        }
        deps.insert(Deps::TargetsToGenerate, libs);
        deps
    }

    fn get_library_name(&self, library: &str) -> String {
        library
            .replace("external/clspv/third_party/llvm", "llvm-project")
            .replace("external/", "")
            .replace("/", "_")
            .replace(".", "_")
    }

    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [
            CC_LIBRARY_HEADERS_SPIRV_TOOLS.to_string(),
            CC_LIBRARY_HEADERS_SPIRV_HEADERS.to_string(),
            CC_LIBRARY_HEADERS_CLSPV.to_string(),
            "OpenCL-Headers".to_string(),
        ]
        .into()
    }

    fn get_target_alias(&self, target: &String) -> String {
        if target == "clvk_libOpenCL_so" {
            "libclvk".to_string()
        } else {
            String::new()
        }
    }

    fn ignore_include(&self, _include: &str) -> bool {
        true
    }

    fn ignore_target(&self, target: &String) -> bool {
        target.starts_with("external/")
    }

    fn update_link_flags(&self, flag: &str, link_flags: &mut HashSet<String>) {
        if flag == "-Wl,-Bsymbolic" {
            link_flags.insert(flag.to_string());
        }
    }
}
