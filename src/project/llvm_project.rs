// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::fs::File;

use crate::project::*;

const CMAKE_GENERATED: &str = "cmake_generated";

#[derive(Default)]
pub struct LlvmProject {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
}

impl Project for LlvmProject {
    fn get_id(&self) -> ProjectId {
        ProjectId::LlvmProject
    }

    fn generate_package(
        &mut self,
        android_path: &Path,
        temp_path: &Path,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_id().android_path(android_path);
        self.build_path = temp_path.join(self.get_id().str());
        self.ndk_path = get_ndk_path(temp_path)?;

        let mut targets_to_build = Vec::new();
        targets_to_build.extend(GenDeps::TargetsToGen.get(self, ProjectId::Clvk, projects_map));
        targets_to_build.extend(GenDeps::LibclcBins.get(self, ProjectId::Clspv, projects_map));
        let (targets, built) = ninja_target::cmake::get_targets(
            &self.src_path.join("llvm"),
            &self.build_path,
            &self.ndk_path,
            vec![
                LLVM_DISABLE_ZLIB,
                "-DLLVM_ENABLE_PROJECTS=clang;libclc",
                "-DLIBCLC_TARGETS_TO_BUILD=clspv--;clspv64--",
                "-DLLVM_TARGETS_TO_BUILD=",
            ],
            Some(targets_to_build),
        )?;
        let copy_gen_deps = if built { true } else { false };

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE.TXT",
        );
        package.generate(
            GenDeps::TargetsToGen.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;

        let libclc_deps = GenDeps::LibclcBins.get(self, ProjectId::Clspv, projects_map);
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
            gen_deps.insert(PathBuf::from(header));
        }

        let mut gen_deps_folders: HashSet<PathBuf> = HashSet::new();
        for gen_dep in &gen_deps {
            let folder = gen_dep.parent().unwrap();
            if let Some((include_folder, _)) = split_path(folder, "include") {
                gen_deps_folders.insert(include_folder);
            } else {
                gen_deps_folders.insert(folder.to_path_buf());
            }
        }
        for module in package.get_modules() {
            module.filter_vec("local_include_dirs", |include| {
                if let Ok(strip) = Path::new(include).strip_prefix(CMAKE_GENERATED) {
                    gen_deps_folders.contains(strip)
                } else {
                    true
                }
            });
        }

        let mut gen_deps_sorted = Vec::from_iter(gen_deps);
        gen_deps_sorted.sort();
        write_file(
            &get_tests_folder()?
                .join(self.get_id().str())
                .join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps_sorted),
        )?;

        if copy_gen_deps {
            let cmake_generated_path = self.src_path.join(CMAKE_GENERATED);
            if File::open(&cmake_generated_path).is_ok() {
                if let Err(err) = std::fs::remove_dir_all(&cmake_generated_path) {
                    return error!("remove_dir_all failed: {err}");
                }

                print_verbose!("{cmake_generated_path:#?} removed");
            }
            for file in gen_deps_sorted {
                let from = self.build_path.join(&file);
                let to = cmake_generated_path.join(file);
                let to_path = to.parent().unwrap();
                if let Err(err) = std::fs::create_dir_all(to_path) {
                    return error!("create_dir_all({to_path:#?}) failed: {err}");
                }
                copy_file(&from, &to)?;
            }
            print_verbose!(
                "Files copied from {0:#?} to {cmake_generated_path:#?}",
                &self.build_path
            );
        }

        let cmake_generated_path = Path::new(CMAKE_GENERATED);
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::Llvm,
            [
                "llvm/include".to_string(),
                path_to_string(cmake_generated_path.join("include")),
            ]
            .into(),
        ));
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::Clang,
            [
                "clang/include".to_string(),
                path_to_string(cmake_generated_path.join("tools/clang/include")),
            ]
            .into(),
        ));

        for clang_header in GenDeps::ClangHeaders.get(self, ProjectId::Clspv, projects_map) {
            package.add_module(SoongModule::new_copy_genrule(
                dep_name(
                    &clang_header,
                    "clang",
                    GenDeps::ClangHeaders.str(),
                    &self.build_path,
                ),
                path_to_string(&clang_header),
                file_name(&clang_header),
            ));
        }
        for file in libclc_deps {
            let file_path = cmake_generated_path.join(file);
            package.add_module(SoongModule::new_copy_genrule(
                dep_name(
                    &file_path,
                    cmake_generated_path,
                    GenDeps::LibclcBins.str(),
                    &self.build_path,
                ),
                path_to_string(&file_path),
                file_name(&file_path),
            ));
        }

        Ok(package)
    }

    fn get_default_cflags(&self) -> Vec<String> {
        vec![
            "-Wno-error".to_string(),
            "-Wno-unreachable-code-loop-increment".to_string(),
        ]
    }

    fn get_include(&self, include: &Path) -> PathBuf {
        Path::new(CMAKE_GENERATED).join(strip_prefix(include, &self.build_path))
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk, ProjectId::Clspv]
    }

    fn ignore_cflag(&self, _cflag: &str) -> bool {
        true
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }

    fn ignore_gen_header(&self, _header: &Path) -> bool {
        true
    }

    fn ignore_target(&self, input: &Path) -> bool {
        !input.starts_with("lib")
    }

    fn optimize_target_for_size(&self, _target: &str) -> bool {
        true
    }
}
