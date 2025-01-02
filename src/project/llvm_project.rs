// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct LlvmProject {
    build_path: PathBuf,
}

impl Project for LlvmProject {
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
    ) -> Result<String, String> {
        let src_path = self.get_android_path(ctx);
        self.build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(src_path.join("llvm")),
                    &path_to_string(&self.build_path),
                    &path_to_string(&ndk_path),
                ]
            )?;
        }
        let targets_to_generate = Dep::LlvmProjectTargets.get(projects_map)?;
        let libclc_binaries = Dep::LibclcBins.get(projects_map)?;
        if !ctx.skip_build {
            let mut targets_to_build = Vec::new();
            targets_to_build.extend(targets_to_generate.clone());
            targets_to_build.extend(libclc_binaries.clone());
            let mut args = vec![String::from("--build"), path_to_string(&self.build_path)];
            for target in targets_to_build {
                args.push(String::from("--target"));
                args.push(path_to_string(target));
            }
            let args: Vec<&str> = args.iter().map(|target| target.as_str()).collect();
            execute_cmd!("cmake", &args)?;
        }

        const CMAKE_GENERATED: &str = "cmake_generated";
        let cmake_generated_path = Path::new(CMAKE_GENERATED);
        let mut package = SoongPackage::new(
            "//visibility:public",
            "llvm-project_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE.TXT"],
        )
        .generate(
            targets_to_generate,
            parse_build_ninja::<CmakeNinjaTarget>(&self.build_path)?,
            &src_path,
            &ndk_path,
            &self.build_path,
            Some(CMAKE_GENERATED),
            self,
        )?
        .add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::Llvm,
            vec![
                String::from("llvm/include"),
                path_to_string(cmake_generated_path.join("include")),
            ],
        ))
        .add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::Clang,
            vec![
                String::from("clang/include"),
                path_to_string(cmake_generated_path.join("tools/clang/include")),
            ],
        ));
        for clang_header in Dep::ClangHeaders.get(projects_map)? {
            package = package.add_module(SoongModule::new_copy_genrule(
                Dep::ClangHeaders.get_id(&clang_header, Path::new("clang"), &self.build_path),
                &clang_header,
            ));
        }
        for binary in &libclc_binaries {
            let file_path = cmake_generated_path.join(binary);
            package = package.add_module(SoongModule::new_copy_genrule(
                Dep::LibclcBins.get_id(&file_path, cmake_generated_path, &self.build_path),
                &file_path,
            ));
        }

        let mut gen_deps = package.get_gen_deps();
        gen_deps.extend(libclc_binaries);
        gen_deps.extend(
            [
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
            ]
            .map(|dep| PathBuf::from(dep)),
        );
        package.filter_local_include_dirs(CMAKE_GENERATED, &gen_deps);
        common::copy_gen_deps(gen_deps, CMAKE_GENERATED, &self.build_path, ctx, self)?;

        Ok(package.print())
    }

    fn get_target_module(&self, _target: &Path, module: SoongModule) -> SoongModule {
        module.add_prop("optimize_for_size", SoongProp::Bool(true))
    }

    fn extend_cflags(&self, target: &Path) -> Vec<String> {
        let mut cflags = vec!["-Wno-error", "-Wno-unreachable-code-loop-increment"];
        if target.ends_with("libLLVMSupport.a") {
            cflags.extend([
                "-DBLAKE3_NO_AVX512",
                "-DBLAKE3_NO_AVX2",
                "-DBLAKE3_NO_SSE41",
                "-DBLAKE3_NO_SSE2",
            ]);
        }
        cflags.into_iter().map(|flag| String::from(flag)).collect()
    }
    fn extend_shared_libs(&self, target: &Path) -> Vec<String> {
        if target.ends_with("libLLVMSupport.a") {
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
