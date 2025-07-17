// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

extern crate ninja_to_soong;
use ninja_to_soong::context::*;
use ninja_to_soong::ninja_parser::*;
use ninja_to_soong::ninja_target::*;
use ninja_to_soong::project::common;
use ninja_to_soong::project::*;
use ninja_to_soong::soong_module::*;
use ninja_to_soong::soong_package::*;
use ninja_to_soong::utils::*;

#[no_mangle]
pub fn get_project() -> Box<dyn Project> {
    Box::new(OpenclCts::default())
}

#[derive(Default)]
pub struct OpenclCts();

const DEFAULTS: &str = "OpenCL-CTS-defaults";
const DEFAULTS_MANUAL: &str = "OpenCL-CTS-manual-defaults";
const CMAKE_GENERATED: &str = "cmake_generated";

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
        if line.is_empty() || line.starts_with("#") || line.starts_with("OpenCL-GL") {
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
    fn get_android_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx
            .get_android_path()?
            .join("external")
            .join(self.get_name()))
    }
    fn get_test_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(PathBuf::from(
            ctx.get_external_project_path()?.parent().unwrap(),
        ))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = self.get_android_path(ctx)?;
        let build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;

        if !ctx.skip_gen_ninja {
            common::cmake_configure(
                &src_path,
                &build_path,
                &ndk_path,
                None,
                None,
                None,
                &[
                    &format!(
                        "-DCL_INCLUDE_DIR={}/../OpenCL-Headers",
                        path_to_string(&src_path)
                    ),
                    &format!("-DCL_LIB_DIR={}", path_to_string(&src_path)),
                    "-DOPENCL_LIBRARIES=-lOpenCL",
                ],
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
            .map(|(test, name)| target_typed!(test, "cc_test", name))
            .collect::<Vec<_>>();
        let mut package = SoongPackage::new(
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
            Some(CMAKE_GENERATED),
            self,
            ctx,
        )?;

        let gen_deps = package.get_gen_deps();
        if !ctx.skip_build {
            common::cmake_build(&build_path, &gen_deps)?;
        }
        common::copy_gen_deps(gen_deps, CMAKE_GENERATED, &build_path, ctx, self)?;

        let default_module = SoongModule::new("cc_defaults")
            .add_prop("name", SoongProp::Str(String::from(DEFAULTS)))
            .add_props(package.get_props(
                "OpenCL-CTS-test_api",
                vec!["cflags", "local_include_dirs", "static_libs", "shared_libs"],
            )?)
            .add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from(DEFAULTS_MANUAL)]),
            );
        package
            .add_module(default_module)
            .add_raw_suffix(&format!(
                r#"
cc_defaults {{
    name: "{DEFAULTS_MANUAL}",
    header_libs: ["OpenCL-Headers"],
    compile_multilib: "both",
    multilib: {{
        lib32: {{
            suffix: "32",
        }},
        lib64: {{
            suffix: "64",
        }},
    }},
    cflags: [
        "-Wno-error",
        "-Wno-c++11-narrowing",
        "-Wno-non-virtual-dtor",
        "-Wno-string-concatenation",
        "-fexceptions",
    ],
    gtest: false,
}}

python_test {{
    name: "opencl_cts_n2s",
    main: "android/ninja-to-soong/test_opencl_cts.py",
    srcs: ["android/ninja-to-soong/test_opencl_cts.py"],
    data: [
{0}
    ],
    test_config: "android/ninja-to-soong/test_opencl_cts.xml",
    test_options: {{
        unit_test: false,
    }},
}}

build = ["AndroidLegacy.bp"]
"#,
                tests
                    .iter()
                    .map(|(_, test)| String::from("        \":") + test + "\",")
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
            .print()
    }

    fn extend_module(&self, target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        let mut data = Vec::new();
        let is_test_spir = target.ends_with("test_spir");
        let spirv_bin = format!("{CMAKE_GENERATED}/test_conformance/spirv_new/spirv_bin/*");
        if target.ends_with("test_compiler") {
            data.push("test_conformance/compiler/includeTestDirectory/testIncludeFile.h");
            data.push("test_conformance/compiler/secondIncludeTestDirectory/testIncludeFile.h")
        } else if is_test_spir {
            data.push("test_conformance/spir/*.zip");
        } else if target.ends_with("test_spirv_new") {
            data.push(&spirv_bin);
        }
        let defaults = if target.ends_with("libharness.a") {
            DEFAULTS_MANUAL
        } else {
            DEFAULTS
        };
        module
            .add_prop("rtti", SoongProp::Bool(is_test_spir))
            .extend_prop("defaults", vec![defaults])?
            .extend_prop("data", data)
    }

    fn map_lib(&self, lib: &Path) -> Option<PathBuf> {
        if lib.ends_with("libOpenCL") {
            Some(PathBuf::from("//external/OpenCL-ICD-Loader:libOpenCL"))
        } else {
            None
        }
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
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
    fn filter_target(&self, target: &Path) -> bool {
        !target.starts_with("test_conformance/spirv_new/spirv_bin")
    }
}
