// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct OpenclCts();

const DEFAULTS: &str = "OpenCL-CTS-defaults";
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
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;

        let spirv_tools_path = if !ctx.skip_build {
            ctx.temp_path.clone()
        } else {
            self.get_test_path(ctx)
        };

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&build_path),
                    &path_to_string(&ndk_path),
                    &path_to_string(&spirv_tools_path),
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
            .add_props(
                package.get_props("OpenCL-CTS-test_api", vec!["cflags", "local_include_dirs"])?,
            )
            .add_prop(
                "defaults",
                SoongProp::VecStr(vec![String::from("OpenCL-CTS-manual-defaults")]),
            );
        package
            .add_module(default_module)
            .add_raw_suffix(&format!(
                r#"
python_defaults {{
    name: "opencl_cts_data",
    data: [
{0}
    ],
}}

build = ["AndroidManual.bp"]
"#,
                tests
                    .iter()
                    .map(|(_, test)| String::from("        \":") + test + "\",")
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
            .print()
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
                    String::from(CMAKE_GENERATED) + "/test_conformance/spirv_new/spirv_bin/*",
                ]),
            )
        }
        module
            .add_prop("rtti", SoongProp::Bool(is_test_spir))
            .add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
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
