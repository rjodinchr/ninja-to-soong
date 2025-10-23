// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::str;

const DEFAULTS: &str = "libclc-defaults";

#[derive(Default)]
pub struct LibCLC {
    src_path: PathBuf,
    host_tools: Vec<String>,
}

impl Project for LibCLC {
    fn get_name(&self) -> &'static str {
        "libclc"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(Path::new("external/opencl/llvm-project").join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        self.src_path = ctx.get_android_path(self)?;
        let build_path = ctx.get_temp_path(Path::new(self.get_name()))?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&build_path),
                ]
            )?;
        }
        let mut package = SoongPackage::new(
            &[],
            "llvm-project_libclc_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE.TXT"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&Dep::LibclcBins.get_ninja_targets(projects_map)?)
                .push(target_typed!("utils/prepare_builtins", "cc_binary_host")),
            parse_build_ninja::<CmakeNinjaTarget>(&build_path)?,
            &self.src_path,
            Path::new("<no_sdk>"),
            &build_path,
            None,
            self,
            ctx,
        )?
        .add_visibilities(Dep::LibclcBins.get_visibilities(projects_map)?);
        self.host_tools = package
            .get_dep_tools_module()
            .into_iter()
            .map(|tool| path_to_string(strip_prefix(tool, "llvm-project")))
            .collect();

        package
            .add_raw_suffix(&format!(
                r#"
cc_genrule_defaults {{
    name: "{DEFAULTS}",
    srcs: [
        "CMakeLists.txt",
        "opencl/include/*.h",
        "opencl/include/**/*.h",
        "opencl/include/**/*.inc",
        "opencl/lib/generic/*.h",
        "opencl/lib/generic/**/*.h",
        "opencl/lib/generic/**/*.inc",
        "opencl/lib/clspv/**/*.inc",
        "clc/include/clc/*.h",
        "clc/include/clc/**/*.h",
        "clc/include/clc/**/*.inc",
        "clc/lib/generic/**/*.h",
        "clc/lib/generic/**/*.inc",
    ],
    vendor_available: true,
}}
"#
            ))
            .print(ctx)
    }

    fn get_deps(&self, _dep: Dep) -> Vec<NinjaTargetToGen> {
        self.host_tools
            .iter()
            .map(|host_tool| target!(host_tool.clone()))
            .collect()
    }

    fn extend_module(&self, _target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        Ok(module.add_prop("defaults", SoongProp::VecStr(vec![CcDefaults::Llvm.str()])))
    }
    fn extend_custom_command(
        &self,
        _target: &Path,
        mut module: SoongModule,
    ) -> Result<SoongModule, String> {
        let Some(cmd_prop) = module.get_prop("cmd") else {
            return Ok(module);
        };
        Ok(match cmd_prop.get_prop() {
            SoongProp::Str(mut cmd) => {
                let src_path = path_to_string(&self.src_path);
                let include = String::from("-I") + &src_path;
                while let Some(begin) = cmd.find(&include) {
                    let include = match str::from_utf8(&cmd.as_bytes()[begin..]).unwrap().find(" ")
                    {
                        Some(end) => {
                            str::from_utf8(&cmd.as_bytes()[(begin + 2)..(begin + end)]).unwrap()
                        }
                        None => str::from_utf8(&cmd.as_bytes()[(begin + 2)..]).unwrap(),
                    };
                    let local_include = strip_prefix(include, &self.src_path);
                    cmd = cmd.replace(
                        include,
                        &(String::from("$$(dirname $(location CMakeLists.txt))/")
                            + &path_to_string(&local_include)),
                    );
                }
                module.update_prop("cmd", |_| Ok(SoongProp::Str(cmd.clone())))?;
                module.add_prop("defaults", SoongProp::VecStr(vec![String::from(DEFAULTS)]))
            }
            _ => module,
        })
    }

    fn map_tool_module(&self, tool_module: &Path) -> Option<PathBuf> {
        let tool_module = path_to_string(tool_module);
        Some(PathBuf::from(if tool_module.contains("clang") {
            "llvm-project/bin/clang-22"
        } else if tool_module.contains("llvm-link") {
            "llvm-project/bin/llvm-link"
        } else if tool_module.contains("opt") {
            "llvm-project/bin/opt"
        } else {
            return None;
        }))
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_define(&self, _define: &str) -> bool {
        false
    }
    fn filter_include(&self, _include: &Path) -> bool {
        false
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
}
