// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

const LLVM_PROJECT_ID: ProjectId = ProjectId::LLVM;
const LLVM_PROJECT_NAME: &str = LLVM_PROJECT_ID.str();
const CMAKE_GENERATED: &str = "cmake_generated";

pub struct LLVM<'a> {
    src_root: &'a str,
    build_root: String,
    ndk_root: &'a str,
    copy_generated_deps: bool,
}

impl<'a> LLVM<'a> {
    pub fn new(temp_dir: &'a str, ndk_root: &'a str, llvm_project_root: &'a str) -> Self {
        LLVM {
            src_root: llvm_project_root,
            build_root: add_slash_suffix(temp_dir) + LLVM_PROJECT_NAME,
            ndk_root,
            copy_generated_deps: true,
        }
    }
}

impl<'a> crate::project::Project<'a> for LLVM<'a> {
    fn get_id(&self) -> ProjectId {
        LLVM_PROJECT_ID
    }

    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        project_map: &ProjectMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            &self.build_root,
            LLVM_PROJECT_NAME,
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE.TXT",
        );
        package.generate(
            get_dependency(
                self,
                ProjectId::CLVK,
                Dependency::TargetToGenerate,
                project_map,
            ),
            targets,
            self,
        )?;

        let clpsv_deps = get_dependency(
            self,
            ProjectId::CLSPV,
            Dependency::LLVMGenerated,
            project_map,
        );
        let include_directories = package.get_include_directories();
        let mut generated_deps = package.get_generated_deps();
        generated_deps.extend(clpsv_deps.clone());
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
            &(add_slash_suffix(&get_tests_folder()?) + LLVM_PROJECT_NAME + "/generated_deps.txt"),
            &format!("{generated_deps_sorted:#?}"),
        )?;
        if self.copy_generated_deps {
            remove_directory(add_slash_suffix(self.src_root) + CMAKE_GENERATED)?;
            copy_files(
                generated_deps,
                &self.build_root,
                &(add_slash_suffix(self.src_root) + CMAKE_GENERATED),
            )?;
            touch_directories(&include_directories, &add_slash_suffix(self.src_root))?;
        }

        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIB_HEADERS_LLVM,
            [
                "llvm/include".to_string(),
                CMAKE_GENERATED.to_string() + "/include",
            ]
            .into(),
        ));
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIB_HEADERS_CLANG,
            [
                "clang/include".to_string(),
                CMAKE_GENERATED.to_string() + "/tools/clang/include",
            ]
            .into(),
        ));

        for header in get_dependency(
            self,
            ProjectId::CLSPV,
            Dependency::CLANGHeaders,
            project_map,
        ) {
            package.add_module(SoongModule::new_copy_genrule(
                clang_headers_name("clang", &header),
                header.clone(),
                header.rsplit_once("/").unwrap().1.to_string(),
            ));
        }
        for file in clpsv_deps {
            let file_path = add_slash_suffix(CMAKE_GENERATED) + &file;
            package.add_module(SoongModule::new_copy_genrule(
                llvm_headers_name(CMAKE_GENERATED, &file_path),
                file_path.clone(),
                file_path.rsplit_once("/").unwrap().1.to_string(),
            ));
        }

        Ok(package)
    }

    fn get_build_directory(&mut self, _project_map: &ProjectMap) -> Result<Option<String>, String> {
        if cmake_configure(
            &(self.src_root.to_string() + "/llvm"),
            &self.build_root,
            self.ndk_root,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DLLVM_ENABLE_PROJECTS=clang;libclc",
                "-DLIBCLC_TARGETS_TO_BUILD=clspv--;clspv64--",
                "-DLLVM_TARGETS_TO_BUILD=",
            ],
        )? {
            if !cmake_build(
                &self.build_root,
                vec![
                    "clang",
                    "tools/libclc/clspv--.bc",
                    "tools/libclc/clspv64--.bc",
                ],
            )? {
                self.copy_generated_deps = false;
            }
        }
        Ok(Some(self.build_root.clone()))
    }

    fn get_default_cflags(&self) -> HashSet<String> {
        [
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
        .into()
    }

    fn ignore_target(&self, input: &String) -> bool {
        !input.starts_with("lib")
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }

    fn get_include(&self, include: &str) -> String {
        include.replace(&self.build_root, CMAKE_GENERATED)
    }

    fn get_headers_to_copy(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        set
    }

    fn optimize_target_for_size(&self, _target: &String) -> bool {
        true
    }

    fn get_project_dependencies(&self) -> Vec<ProjectId> {
        vec![ProjectId::CLVK, ProjectId::CLSPV]
    }
}
