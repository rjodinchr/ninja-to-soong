// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

pub struct Clspv<'a> {
    src_dir: &'a str,
    build_dir: String,
    ndk_dir: &'a str,
    spirv_headers_dir: &'a str,
    spirv_tools_dir: &'a str,
    llvm_project_dir: &'a str,
    gen_deps: HashSet<String>,
}

impl<'a> Clspv<'a> {
    pub fn new(
        temp_dir: &'a str,
        ndk_dir: &'a str,
        clspv_dir: &'a str,
        llvm_project_dir: &'a str,
        spirv_tools_dir: &'a str,
        spirv_headers_dir: &'a str,
    ) -> Self {
        Clspv {
            src_dir: clspv_dir,
            build_dir: add_slash_suffix(temp_dir) + ProjectId::Clspv.str(),
            ndk_dir,
            llvm_project_dir,
            spirv_tools_dir,
            spirv_headers_dir,
            gen_deps: HashSet::new(),
        }
    }
}

impl<'a> crate::project::Project<'a> for Clspv<'a> {
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_dir,
            self.ndk_dir,
            &self.build_dir,
            ProjectId::Clspv.str(),
            "//external/clvk",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(
            GenDeps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_CLSPV,
            ["include".to_string()].into(),
        ));

        self.gen_deps = package.get_gen_deps();

        Ok(package)
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::Clspv
    }

    fn get_build_dir(&mut self, _projects_map: &ProjectsMap) -> Result<Option<String>, String> {
        let spirv_headers_dir = "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + self.spirv_headers_dir;
        let spirv_tools_dir = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string() + self.spirv_tools_dir;
        let llvm_project_dir =
            "-DCLSPV_LLVM_SOURCE_DIR=".to_string() + self.llvm_project_dir + "/llvm";
        let clang_dir = "-DCLSPV_CLANG_SOURCE_DIR=".to_string() + self.llvm_project_dir + "/clang";
        let libclc_dir =
            "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string() + self.llvm_project_dir + "/libclc";
        cmake_configure(
            self.src_dir,
            &self.build_dir,
            self.ndk_dir,
            vec![
                &spirv_headers_dir,
                &spirv_tools_dir,
                &llvm_project_dir,
                &clang_dir,
                &libclc_dir,
            ],
        )?;
        Ok(Some(self.build_dir.clone()))
    }

    fn get_cmd_inputs_and_deps(
        &self,
        target_inputs: &Vec<String>,
    ) -> Result<CmdInputAndDeps, String> {
        let mut inputs: HashSet<String> = HashSet::new();
        let mut deps: HashSet<(String, String)> = HashSet::new();
        let clang_dir = &(self.llvm_project_dir.to_string() + "/clang");

        for input in target_inputs {
            if input.contains(self.spirv_headers_dir) {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(self.spirv_headers_dir, input),
                ));
            } else if input.contains(clang_dir) {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &clang_headers_name(clang_dir, input),
                ));
            } else if input.contains("third_party/llvm") {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &llvm_project_headers_name("third_party/llvm", input),
                ));
            } else if !input.contains(self.src_dir) {
                deps.insert((
                    input.clone(),
                    ":".to_string()
                        + ProjectId::Clspv.str()
                        + "_"
                        + &rework_name(input.replace(&self.build_dir, "")),
                ));
            } else {
                inputs.insert(input.clone());
            }
        }
        Ok((inputs, deps))
    }

    fn get_cmd_output(&self, output: &str) -> String {
        if let Some(split) = output.split_once("include/") {
            split.1
        } else if !output.contains("libclc") {
            output.split("/").last().unwrap()
        } else {
            output
        }
        .to_string()
    }

    fn get_default_cflags(&self) -> HashSet<String> {
        ["-Wno-unreachable-code-loop-increment".to_string()].into()
    }

    fn get_gen_deps(&self, project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        match project {
            ProjectId::SpirvHeaders => {
                let mut files: HashSet<String> = HashSet::new();
                for dep in &self.gen_deps {
                    if dep.starts_with(self.spirv_headers_dir) {
                        files.insert(dep.clone());
                    }
                }
                deps.insert(GenDeps::SpirvHeadersFiles, files);
            }
            ProjectId::LlvmProject => {
                let mut clang_headers: HashSet<String> = HashSet::new();
                let mut libclc_binaries: HashSet<String> = HashSet::new();
                for dep in &self.gen_deps {
                    if let Some(strip) = dep.strip_prefix(&add_slash_suffix(self.llvm_project_dir))
                    {
                        clang_headers.insert(strip.to_string());
                    } else if dep.starts_with("third_party/llvm/tools/libclc/clspv")
                        && dep.ends_with(".bc")
                    {
                        libclc_binaries.insert(
                            dep.strip_prefix(&add_slash_suffix("third_party/llvm"))
                                .unwrap()
                                .to_string(),
                        );
                    }
                }
                deps.insert(GenDeps::ClangHeaders, clang_headers);
                deps.insert(GenDeps::LibclcBinaries, libclc_binaries);
            }
            _ => (),
        };
        deps
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }

    fn get_target_header_libs(&self, _target: &str) -> HashSet<String> {
        [
            CC_LIBRARY_HEADERS_SPIRV_HEADERS.to_string(),
            CC_LIBRARY_HEADERS_LLVM.to_string(),
            CC_LIBRARY_HEADERS_CLANG.to_string(),
        ]
        .into()
    }

    fn ignore_defines(&self) -> bool {
        true
    }

    fn ignore_gen_header(&self, header: &str) -> bool {
        header.contains("third_party/llvm")
    }

    fn ignore_include(&self, include: &str) -> bool {
        include.contains(&self.build_dir)
            || include.contains(self.spirv_headers_dir)
            || include.contains(self.llvm_project_dir)
    }

    fn ignore_target(&self, target: &str) -> bool {
        target.starts_with("third_party/")
    }

    fn optimize_target_for_size(&self, _target: &str) -> bool {
        true
    }
}
