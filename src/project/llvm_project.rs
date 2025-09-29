// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct LlvmProject();

const DEFAULTS: &str = "llvm-project-defaults";
const RAW_DEFAULTS: &str = "llvm-project-raw-defaults";

impl Project for LlvmProject {
    fn get_name(&self) -> &'static str {
        "llvm-project"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(Path::new("external/opencl").join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = ctx.get_android_path(self)?;
        let build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
                    &path_to_string(src_path.join("llvm")),
                    &path_to_string(&build_path),
                    &path_to_string(&ndk_path),
                ]
            )?;
        }

        const CMAKE_GENERATED: &str = "cmake_generated";
        let cmake_generated_path = Path::new(CMAKE_GENERATED);
        let mut package = SoongPackage::new(
            &[],
            "llvm-project_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE.TXT"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&Dep::LlvmProjectTargets.get_ninja_targets(projects_map)?),
            parse_build_ninja::<CmakeNinjaTarget>(&build_path)?,
            &src_path,
            &ndk_path,
            &build_path,
            Some(CMAKE_GENERATED),
            self,
            ctx,
        )?
        .add_visibilities(Dep::ClangHeaders.get_visibilities(projects_map)?)
        .add_visibilities(Dep::LibclcBins.get_visibilities(projects_map)?)
        .add_visibilities(Dep::LlvmProjectTargets.get_visibilities(projects_map)?)
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
        ))
        .add_module(
            SoongModule::new("cc_defaults")
                .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
                .add_prop(
                    "local_include_dirs",
                    SoongProp::VecStr(vec![
                        String::from(CMAKE_GENERATED) + "/include",
                        String::from("llvm/include"),
                    ]),
                )
                .add_prop(
                    "defaults",
                    SoongProp::VecStr(vec![String::from(RAW_DEFAULTS)]),
                ),
        );
        for clang_header in Dep::ClangHeaders.get(projects_map)? {
            package = package.add_module(SoongModule::new_filegroup(
                Dep::ClangHeaders.get_id(&clang_header, Path::new("clang"), &build_path),
                vec![path_to_string(clang_header)],
            ));
        }
        let libclc_binaries = Dep::LibclcBins.get(projects_map)?;
        for binary in &libclc_binaries {
            let file_path = cmake_generated_path.join(binary);
            package = package.add_module(SoongModule::new_filegroup(
                Dep::LibclcBins.get_id(&file_path, cmake_generated_path, &build_path),
                vec![path_to_string(file_path)],
            ));
        }

        let mut gen_deps = package.get_gen_deps();
        gen_deps.extend(libclc_binaries);
        common::ninja_build(&build_path, &gen_deps, ctx)?;
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
        package.filter_local_include_dirs(CMAKE_GENERATED, &gen_deps)?;
        common::copy_gen_deps(gen_deps, CMAKE_GENERATED, &build_path, ctx, self)?;

        package
            .add_raw_suffix(&format!(
                r#"
cc_defaults {{
    name: "{RAW_DEFAULTS}",
    optimize_for_size: true,
    vendor_available: true,
    cflags: [
        "-Wno-error",
        "-Wno-unreachable-code-loop-increment",
    ],
}}
"#
            ))
            .print(ctx)
    }

    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        let (cflags, libs) = if target.ends_with("libLLVMSupport.a") {
            (
                vec![
                    "-DBLAKE3_NO_AVX512",
                    "-DBLAKE3_NO_AVX2",
                    "-DBLAKE3_NO_SSE41",
                    "-DBLAKE3_NO_SSE2",
                ],
                vec!["libz"],
            )
        } else {
            (Vec::new(), Vec::new())
        };
        module
            .add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
            .extend_prop("cflags", cflags)?
            .extend_prop("shared_libs", libs)
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
