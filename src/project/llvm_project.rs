// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::fs::File;

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

const CMAKE_GENERATED: &str = "cmake_generated";

#[derive(Default)]
pub struct LlvmProject {
    src_dir: String,
    build_dir: String,
    ndk_dir: String,
    copy_gen_deps: bool,
}

impl Project for LlvmProject {
    fn init(&mut self, android_dir: &str, ndk_dir: &str, temp_dir: &str) {
        self.src_dir = self.get_id().android_path(android_dir);
        self.build_dir = add_slash_suffix(temp_dir) + self.get_id().str();
        self.ndk_dir = ndk_dir.to_string();
        self.copy_gen_deps = false;
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::LlvmProject
    }

    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            &self.src_dir,
            &self.ndk_dir,
            &self.build_dir,
            self.get_id().str(),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE.TXT",
        );
        package.generate(
            GenDeps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;

        let libclc_deps = GenDeps::LibclcBinaries.get(self, ProjectId::Clspv, projects_map);
        let mut gen_deps = package.get_gen_deps();
        gen_deps.extend(libclc_deps.clone());
        let missing_gen_deps = vec![
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
        for header in missing_gen_deps {
            gen_deps.insert(header.to_string());
        }

        let mut gen_deps_folders: HashSet<String> = HashSet::new();
        for gen_dep in &gen_deps {
            let folder = gen_dep.rsplit_once("/").unwrap().0;
            let include_str = "include";
            if let Some(include) = folder.split_once(include_str) {
                gen_deps_folders.insert(include.0.to_string() + include_str);
            } else {
                gen_deps_folders.insert(folder.to_string());
            }
        }
        for module in package.get_modules() {
            module.filter_set("local_include_dirs", |include| {
                if let Some(strip) = include.strip_prefix(&add_slash_suffix(CMAKE_GENERATED)) {
                    gen_deps_folders.contains(strip)
                } else {
                    true
                }
            });
        }

        let mut gen_deps_sorted = Vec::from_iter(gen_deps);
        gen_deps_sorted.sort();
        write_file(
            &(add_slash_suffix(&get_tests_folder()?) + self.get_id().str() + "/generated_deps.txt"),
            &format!("{0:#?}", &gen_deps_sorted),
        )?;

        let cmake_generated_dir = add_slash_suffix(&self.src_dir) + CMAKE_GENERATED;
        if self.copy_gen_deps && File::open(&cmake_generated_dir).is_ok() {
            if let Err(err) = std::fs::remove_dir_all(&cmake_generated_dir) {
                return error!("remove_dir_all failed: {err}");
            }
            print_verbose!("'{cmake_generated_dir}' removed");
            for file in gen_deps_sorted {
                let from = add_slash_suffix(&self.build_dir) + &file;
                let to = add_slash_suffix(&cmake_generated_dir) + &file;
                let to_dir = to.rsplit_once("/").unwrap().0;
                if let Err(err) = std::fs::create_dir_all(to_dir) {
                    return error!("create_dir_all({to_dir}) failed: {err}");
                }
                copy_file(&from, &to)?;
            }
            print_verbose!(
                "Files copied from '{0}' to '{cmake_generated_dir}'",
                &self.build_dir
            );
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

        for clang_header in GenDeps::ClangHeaders.get(self, ProjectId::Clspv, projects_map) {
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

    fn get_ninja_file_path(
        &mut self,
        projects_map: &ProjectsMap,
    ) -> Result<Option<String>, String> {
        let (ninja_file_path, configured) = cmake_configure(
            &(self.src_dir.to_string() + "/llvm"),
            &self.build_dir,
            &self.ndk_dir,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DLLVM_ENABLE_PROJECTS=clang;libclc",
                "-DLIBCLC_TARGETS_TO_BUILD=clspv--;clspv64--",
                "-DLLVM_TARGETS_TO_BUILD=",
            ],
        )?;
        if configured {
            let mut targets = Vec::new();
            targets.extend(GenDeps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map));
            targets.extend(GenDeps::LibclcBinaries.get(self, ProjectId::Clspv, projects_map));
            if cmake_build(&self.build_dir, &targets)? {
                self.copy_gen_deps = true;
            }
        }
        Ok(Some(ninja_file_path))
    }

    fn get_default_cflags(&self) -> HashSet<String> {
        [
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
        .into()
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

    fn ignore_gen_header(&self, _header: &str) -> bool {
        true
    }

    fn ignore_target(&self, input: &str) -> bool {
        !input.starts_with("lib")
    }

    fn optimize_target_for_size(&self, _target: &str) -> bool {
        true
    }
}
