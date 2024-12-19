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
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_id().android_path(ctx);
        self.build_path = ctx.temp_path.join(self.get_id().str());
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(ctx.test_path.join(self.get_id().str()).join("gen-ninja.sh")),
                    &path_to_string(self.src_path.join("llvm")),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.ndk_path),
                    ANDROID_ABI,
                    ANDROID_PLATFORM,
                ]
            )?;
        }
        if !ctx.skip_build {
            let mut targets_to_build = Vec::new();
            targets_to_build.extend(GenDeps::TargetsToGen.get(self, ProjectId::Clvk, projects_map));
            targets_to_build.extend(GenDeps::LibclcBins.get(self, ProjectId::Clspv, projects_map));
            let mut args = vec![String::from("--build"), path_to_string(&self.build_path)];
            for target in targets_to_build {
                args.push(String::from("--target"));
                args.push(path_to_string(target));
            }
            execute_cmd!("cmake", args.iter().map(|target| target.as_str()).collect())?;
        }

        let targets = parse_build_ninja::<ninja_target::cmake::CmakeNinjaTarget>(&self.build_path)?;

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
        gen_deps.extend(missing_gen_deps.iter().map(|dep| PathBuf::from(dep)));

        let mut gen_deps_folders = HashSet::new();
        for gen_dep in &gen_deps {
            let folder = gen_dep.parent().unwrap();
            if let Some((include_folder, _)) = split_path(folder, "include") {
                gen_deps_folders.insert(include_folder);
            } else {
                gen_deps_folders.insert(PathBuf::from(folder));
            }
        }
        for module in package.get_modules() {
            let _ = module.update_prop("local_include_dirs", |prop| match prop {
                SoongProp::VecStr(dirs) => {
                    let mut new_dirs = Vec::new();
                    for dir in dirs {
                        if let Ok(strip) = Path::new(&dir).strip_prefix(CMAKE_GENERATED) {
                            if !gen_deps_folders.contains(strip) {
                                continue;
                            }
                        }
                        new_dirs.push(dir);
                    }
                    Ok(SoongProp::VecStr(new_dirs))
                }
                _ => error!("Expected local_include_dirs to be a VecStr"),
            });
        }

        let mut gen_deps_sorted = Vec::from_iter(gen_deps);
        gen_deps_sorted.sort();
        write_file(
            &ctx.test_path
                .join(self.get_id().str())
                .join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps_sorted),
        )?;

        if ctx.copy_to_aosp {
            let cmake_generated_path = self.get_id().android_path(ctx).join(CMAKE_GENERATED);
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
            vec![
                String::from("llvm/include"),
                path_to_string(cmake_generated_path.join("include")),
            ],
        ));
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::Clang,
            vec![
                String::from("clang/include"),
                path_to_string(cmake_generated_path.join("tools/clang/include")),
            ],
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

    fn get_default_cflags(&self, target: &str) -> Vec<String> {
        let mut cflags = vec!["-Wno-error", "-Wno-unreachable-code-loop-increment"];
        if target.ends_with("libLLVMSupport_a") {
            cflags.append(&mut vec![
                "-DBLAKE3_NO_AVX512",
                "-DBLAKE3_NO_AVX2",
                "-DBLAKE3_NO_SSE41",
                "-DBLAKE3_NO_SSE2",
            ]);
        }
        cflags.into_iter().map(|flag| String::from(flag)).collect()
    }

    fn get_include(&self, include: &Path) -> PathBuf {
        Path::new(CMAKE_GENERATED).join(strip_prefix(include, &self.build_path))
    }

    fn get_library_module(&self, module: &mut SoongModule) {
        module.add_prop("optimize_for_size", SoongProp::Bool(true));
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk, ProjectId::Clspv]
    }

    fn get_shared_libs(&self, target: &str) -> Vec<String> {
        if target.ends_with("libLLVMSupport_a") {
            vec![String::from("libz")]
        } else {
            Vec::new()
        }
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }

    fn filter_define(&self, _define: &str) -> bool {
        false
    }

    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }

    fn filter_target(&self, input: &Path) -> bool {
        input.starts_with("lib")
    }
}
