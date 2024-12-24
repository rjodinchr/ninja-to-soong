// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

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
    fn get_name(&self) -> &'static str {
        "llvm-project"
    }
    fn get_android_path(&self, ctx: &Context) -> PathBuf {
        ctx.android_path.join("external").join(self.get_name())
    }
    fn get_test_path(&self, ctx: &Context) -> PathBuf {
        ctx.test_path.join(self.get_name())
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_android_path(ctx);
        self.build_path = ctx.temp_path.join(self.get_name());
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(self.src_path.join("llvm")),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.ndk_path),
                    ANDROID_ABI,
                    ANDROID_PLATFORM,
                ]
            )?;
        }
        let targets_to_generate =
            projects_map.get_deps(ProjectId::Clvk, self.get_id(), GenDeps::TargetsToGen)?;
        let libclc_binaries =
            projects_map.get_deps(ProjectId::Clspv, self.get_id(), GenDeps::LibclcBins)?;
        if !ctx.skip_build {
            let mut targets_to_build = Vec::new();
            targets_to_build.extend(targets_to_generate.clone());
            targets_to_build.extend(libclc_binaries.clone());
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
            "//visibility:public",
            "llvm-project_license",
            vec!["SPDX-license-identifier-Apache-2.0"],
            vec!["LICENSE.TXT"],
        );
        package.generate(targets_to_generate, targets, self)?;

        let mut gen_deps = package.get_gen_deps();
        gen_deps.extend(libclc_binaries.clone());
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

        package.filter_local_include_dirs(CMAKE_GENERATED, &gen_deps);
        gen_deps.sort();
        write_file(
            &self.get_test_path(ctx).join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps),
        )?;
        if ctx.copy_to_aosp {
            copy_files(
                &self.build_path,
                &self.get_android_path(ctx).join(CMAKE_GENERATED),
                gen_deps,
            )?;
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

        for clang_header in
            projects_map.get_deps(ProjectId::Clspv, self.get_id(), GenDeps::ClangHeaders)?
        {
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
        for binary in libclc_binaries {
            let file_path = cmake_generated_path.join(binary);
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

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk, ProjectId::Clspv]
    }

    fn get_target_object_module(&self, _target: &str, mut module: SoongModule) -> SoongModule {
        module.add_prop("optimize_for_size", SoongProp::Bool(true));
        module
    }
    fn get_target_cflags(&self, target: &str) -> Vec<String> {
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
    fn get_target_shared_libs(&self, target: &str) -> Vec<String> {
        if target.ends_with("libLLVMSupport_a") {
            vec![String::from("libz")]
        } else {
            Vec::new()
        }
    }

    fn get_include(&self, include: &Path) -> PathBuf {
        Path::new(CMAKE_GENERATED).join(strip_prefix(include, &self.build_path))
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
