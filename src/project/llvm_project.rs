// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

const CMAKE_GENERATED: &str = "cmake_generated";

pub struct LlvmProject<'a> {
    src_dir: &'a str,
    build_dir: String,
    ndk_dir: &'a str,
    copy_generated_deps: bool,
}

impl<'a> LlvmProject<'a> {
    pub fn new(temp_dir: &'a str, ndk_dir: &'a str, llvm_project_dir: &'a str) -> Self {
        LlvmProject {
            src_dir: llvm_project_dir,
            build_dir: add_slash_suffix(temp_dir) + ProjectId::LlvmProject.str(),
            ndk_dir,
            copy_generated_deps: true,
        }
    }
}

impl<'a> crate::project::Project<'a> for LlvmProject<'a> {
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_dir,
            self.ndk_dir,
            &self.build_dir,
            ProjectId::LlvmProject.str(),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE.TXT",
        );
        package.generate(
            Deps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;

        let libclc_deps = Deps::LibclcBinaries.get(self, ProjectId::Clspv, projects_map);
        let include_dirs = package.get_include_dirs();
        let mut generated_deps = package.get_generated_deps();
        generated_deps.extend(libclc_deps.clone());
        let missing_generated_deps = vec![
            "include/llvm/Config/llvm-config.h",
            "include/llvm/Config/abi-breaking.h",
            "include/llvm/Config/config.h",
            "include/llvm/Config/Targets.def",
            "include/llvm/Config/AsmPrinters.def",
            "include/llvm/Config/AsmParsers.def",
            "include/llvm/Config/Disassemblers.def",
            "include/llvm/Config/TargetMCAs.def",
            "include/llvm/Support/Extension.def",
            "include/llvm/Support/VCSRevision.h",
            "tools/clang/lib/Basic/VCSVersion.inc",
            "tools/clang/include/clang/Basic/Version.inc",
            "tools/clang/include/clang/Config/config.h",
        ];
        for header in missing_generated_deps {
            generated_deps.insert(header.to_string());
        }

        let mut generated_deps_sorted = Vec::from_iter(&generated_deps);
        generated_deps_sorted.sort();
        write_file(
            &(add_slash_suffix(&get_tests_folder()?)
                + ProjectId::LlvmProject.str()
                + "/generated_deps.txt"),
            &format!("{generated_deps_sorted:#?}"),
        )?;
        if self.copy_generated_deps {
            remove_dir(add_slash_suffix(self.src_dir) + CMAKE_GENERATED)?;
            copy_files(
                generated_deps,
                &self.build_dir,
                &(add_slash_suffix(self.src_dir) + CMAKE_GENERATED),
            )?;
            touch_dirs(&include_dirs, &add_slash_suffix(self.src_dir))?;
        }

        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_LLVM,
            [
                "llvm/include".to_string(),
                CMAKE_GENERATED.to_string() + "/include",
            ]
            .into(),
        ));
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_CLANG,
            [
                "clang/include".to_string(),
                CMAKE_GENERATED.to_string() + "/tools/clang/include",
            ]
            .into(),
        ));

        for clang_header in Deps::ClangHeaders.get(self, ProjectId::Clspv, projects_map) {
            package.add_module(SoongModule::new_copy_genrule(
                clang_headers_name("clang", &clang_header),
                clang_header.clone(),
                clang_header.rsplit_once("/").unwrap().1.to_string(),
            ));
        }
        for file in libclc_deps {
            let file_path = add_slash_suffix(CMAKE_GENERATED) + &file;
            package.add_module(SoongModule::new_copy_genrule(
                llvm_project_headers_name(CMAKE_GENERATED, &file_path),
                file_path.clone(),
                file_path.rsplit_once("/").unwrap().1.to_string(),
            ));
        }

        Ok(package)
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::LlvmProject
    }

    fn get_build_dir(&mut self, projects_map: &ProjectsMap) -> Result<Option<String>, String> {
        if cmake_configure(
            &(self.src_dir.to_string() + "/llvm"),
            &self.build_dir,
            self.ndk_dir,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DLLVM_ENABLE_PROJECTS=clang;libclc",
                "-DLIBCLC_TARGETS_TO_BUILD=clspv--;clspv64--",
                "-DLLVM_TARGETS_TO_BUILD=",
            ],
        )? {
            let mut targets = Vec::new();
            targets.extend(Deps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map));
            targets.extend(Deps::LibclcBinaries.get(self, ProjectId::Clspv, projects_map));
            if !cmake_build(&self.build_dir, &targets)? {
                self.copy_generated_deps = false;
            }
        }
        Ok(Some(self.build_dir.clone()))
    }

    fn get_default_cflags(&self) -> HashSet<String> {
        [
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
        .into()
    }

    fn get_headers_to_copy(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        set
    }

    fn get_include(&self, include: &str) -> String {
        include.replace(&self.build_dir, CMAKE_GENERATED)
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk, ProjectId::Clspv]
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }

    fn ignore_target(&self, input: &String) -> bool {
        !input.starts_with("lib")
    }

    fn optimize_target_for_size(&self, _target: &String) -> bool {
        true
    }
}
