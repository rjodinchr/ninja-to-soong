// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct OpenclCts();

fn parse_test(line: &str) -> Option<String> {
    let split_comma = line.split(",");
    let Some(cmd) = split_comma.last() else {
        return None;
    };
    let mut split_space = cmd.trim().split(" ");
    let Some(binary) = split_space.next() else {
        return None;
    };
    Some(String::from("test_conformance/") + binary)
}

fn parse_tests(src_path: &Path) -> Result<Vec<String>, String> {
    let mut tests = Vec::new();
    let content = read_file(&src_path.join("test_conformance/opencl_conformance_tests_full.csv"))?;
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if line.is_empty() || line.starts_with("#") {
            continue;
        }
        if let Some(test) = parse_test(line) {
            tests.push(test);
        }
    }
    tests.sort_unstable();
    tests.dedup();

    Ok(tests)
}

impl Project for OpenclCts {
    fn get_name(&self) -> &'static str {
        "OpenCL-CTS"
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
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = self.get_android_path(ctx);
        let build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&build_path),
                    &path_to_string(&ndk_path),
                ]
            )?;
        }
        let tests = parse_tests(&src_path)?
            .into_iter()
            .map(|test| {
                (
                    test.clone(),
                    String::from(self.get_name()) + "-" + test.split("/").last().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let targets = tests
            .iter()
            .map(|(test, name)| NinjaTargetToGen(test, Some(name), None))
            .collect::<Vec<_>>();
        let package = SoongPackage::new(
            &["//visibility:public"],
            "external_OpenCL-CTS_license",
            &[
                "SPDX-license-identifier-Apache-2.0",
                "SPDX-license-identifier-BSD",
                "SPDX-license-identifier-MIT",
                "SPDX-license-identifier-Unlicense",
            ],
            &["LICENSE.txt"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&targets),
            parse_build_ninja::<CmakeNinjaTarget>(&build_path)?,
            &src_path,
            &ndk_path,
            &build_path,
            None,
            self,
        )?
        .add_raw_suffix(
            r###"
cc_defaults {
    name: "ocl-defaults",
    header_libs: ["OpenCL-Headers"],
    compile_multilib: "64",
    multilib: {
        lib64: {
            suffix: "64",
        },
    },
    cflags: [
        "-Wno-#warnings",
        "-Wno-c++11-narrowing",
        "-Wno-date-time",
        "-Wno-deprecated-declarations",
        "-Wno-format",
        "-Wno-ignored-qualifiers",
        "-Wno-implicit-fallthrough",
        "-Wno-missing-braces",
        "-Wno-missing-field-initializers",
        "-Wno-non-virtual-dtor",
        "-Wno-overloaded-virtual",
        "-Wno-reorder-ctor",
        "-Wno-sometimes-uninitialized",
        "-Wno-unused-parameter",
        "-fexceptions",
    ],
}

python_test_host {
    name: "opencl_cts",
    main: "scripts/test_opencl_cts.py",
    srcs: ["scripts/test_opencl_cts.py"],
    data: ["scripts/test_opencl_cts.xml"],
    test_config: "scripts/test_opencl_cts.xml",
    test_options: {
        unit_test: false,
    },
}

python_test {
    name: "run_conformance",
    main: "test_conformance/run_conformance.py",
    srcs: ["test_conformance/run_conformance.py"],
}
"###,
        );

        Ok(package.print())
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> SoongModule {
        if target.ends_with("test_compiler") {
            module = module.add_prop(
                "data",
                SoongProp::VecStr(vec![
                    String::from(
                        "test_conformance/compiler/includeTestDirectory/testIncludeFile.h",
                    ),
                    String::from(
                        "test_conformance/compiler/secondIncludeTestDirectory/testIncludeFile.h",
                    ),
                ]),
            );
        }
        let is_test_spir = target.ends_with("test_spir");
        if is_test_spir {
            module = module.add_prop(
                "data",
                SoongProp::VecStr(vec![String::from("test_conformance/spir/*.zip")]),
            );
        }
        if target.ends_with("test_spirv_new") {
            module = module.add_prop(
                "data",
                SoongProp::VecStr(vec![
                    String::from("test_conformance/spirv_new/spirv_asm/*"),
                    String::from("test_conformance/spirv_new/spirv_bin/*"),
                ]),
            )
        }
        if target.starts_with("test_conformance") {
            module = module.add_prop("gtest", SoongProp::Bool(false))
        }
        module
            .add_prop("rtti", SoongProp::Bool(is_test_spir))
            .add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from("ocl-defaults")]),
            )
    }

    fn map_lib(&self, lib: &Path) -> Option<PathBuf> {
        if lib.ends_with("libOpenCL") {
            Some(PathBuf::from("//external/OpenCL-ICD-Loader:libOpenCL"))
        } else {
            None
        }
    }
    fn map_module_name(&self, _target: &Path, module_name: &str) -> String {
        String::from(if module_name == "cc_binary" {
            "cc_test"
        } else {
            module_name
        })
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_include(&self, include: &Path) -> bool {
        !include.ends_with("OpenCL-Headers")
    }
    fn filter_lib(&self, lib: &str) -> bool {
        !lib.contains("atomic")
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
}
