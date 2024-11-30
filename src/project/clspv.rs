// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

const CLSPV_PROJECT_ID: ProjectId = ProjectId::CLSPV;
const CLSPV_PROJECT_NAME: &str = CLSPV_PROJECT_ID.str();
const LLVM_PREFIX: &str = "third_party/llvm";

pub struct CLSPV<'a> {
    src_root: &'a str,
    build_root: String,
    ndk_root: &'a str,
    spirv_headers_root: &'a str,
    spirv_tools_root: &'a str,
    llvm_project_root: &'a str,
}

impl<'a> CLSPV<'a> {
    pub fn new(
        temp_dir: &'a str,
        ndk_root: &'a str,
        clspv_root: &'a str,
        llvm_project_root: &'a str,
        spirv_tools_root: &'a str,
        spirv_headers_root: &'a str,
    ) -> Self {
        CLSPV {
            src_root: clspv_root,
            build_root: temp_dir.to_string() + "/" + CLSPV_PROJECT_NAME,
            ndk_root,
            llvm_project_root,
            spirv_tools_root,
            spirv_headers_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for CLSPV<'a> {
    fn get_id(&self) -> ProjectId {
        CLSPV_PROJECT_ID
    }
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        _dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            &self.build_root,
            CLSPV_PROJECT_NAME,
            "//external/clvk",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(vec!["libclspv_core.a"], targets, self)?;
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIB_HEADERS_CLSPV,
            ["include".to_string()].into(),
        ));
        return Ok(package);
    }

    fn get_build_directory(
        &mut self,
        _dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<String, String> {
        let spirv_headers_dir = "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + self.spirv_headers_root;
        let spirv_tools_dir = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string() + self.spirv_tools_root;
        let llvm_dir = "-DCLSPV_LLVM_SOURCE_DIR=".to_string() + self.llvm_project_root + "/llvm";
        let clang_dir = "-DCLSPV_CLANG_SOURCE_DIR=".to_string() + self.llvm_project_root + "/clang";
        let libclc_dir =
            "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string() + self.llvm_project_root + "/libclc";
        cmake_configure(
            self.src_root,
            &self.build_root,
            self.ndk_root,
            vec![
                &spirv_headers_dir,
                &spirv_tools_dir,
                &llvm_dir,
                &clang_dir,
                &libclc_dir,
            ],
        )?;
        return Ok(self.build_root.clone());
    }

    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        let mut srcs: HashSet<String> = HashSet::new();
        let mut filtered_inputs: HashSet<String> = HashSet::new();
        let mut generated_deps: HashSet<(String, String)> = HashSet::new();
        let clang_root = &(self.llvm_project_root.to_string() + "/clang");

        for input in inputs {
            if input.contains(self.spirv_headers_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(self.spirv_headers_root, input),
                ));
            } else if input.contains(clang_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &clang_headers_name(clang_root, input),
                ));
            } else if input.contains(LLVM_PREFIX) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &llvm_headers_name(LLVM_PREFIX, input),
                ));
            } else if !input.contains(self.src_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string()
                        + CLSPV_PROJECT_NAME
                        + "_"
                        + &rework_name(input.replace(&self.build_root, "")),
                ));
            } else {
                filtered_inputs.insert(input.clone());
            }
        }
        for input in &filtered_inputs {
            srcs.insert(input.replace(&add_slash_suffix(self.src_root), ""));
        }
        for (_, dep) in &generated_deps {
            srcs.insert(dep.clone());
        }
        return Ok((srcs, filtered_inputs, generated_deps));
    }
    fn get_default_cflags(&self) -> HashSet<String> {
        return ["-Wno-unreachable-code-loop-increment".to_string()].into();
    }
    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
    fn ignore_target(&self, target: &String) -> bool {
        target.starts_with("third_party/")
    }
    fn ignore_include(&self, include: &str) -> bool {
        include.contains(&self.build_root)
            || include.contains(self.spirv_headers_root)
            || include.contains(self.llvm_project_root)
    }
    fn get_headers_to_generate(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            if !header.contains(LLVM_PREFIX) {
                set.insert(header.clone());
            }
        }
        return set;
    }
    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [
            CC_LIB_HEADERS_SPIRV_HEADERS.to_string(),
            CC_LIB_HEADERS_LLVM.to_string(),
            CC_LIB_HEADERS_CLANG.to_string(),
        ]
        .into()
    }
    fn rework_command_output(&self, output: &str) -> String {
        if let Some(split) = output.split_once("include/") {
            split.1
        } else if !output.contains("libclc") {
            output.split("/").last().unwrap()
        } else {
            output
        }
        .to_string()
    }
    fn optimize_target_for_size(&self, _target: &String) -> bool {
        true
    }
}
