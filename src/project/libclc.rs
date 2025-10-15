// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::str;

#[derive(Default)]
pub struct LibCLC {
    src_path: PathBuf,
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
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        self.src_path = ctx.get_android_path(self)?;
        let build_path = ctx.temp_path.join(self.get_name());

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
        SoongPackage::new(
            &[],
            "llvm-project_libclc_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE.TXT"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[
                target!("clspv--.bc"),
                target!("clspv64--.bc"),
                target_typed!("utils/prepare_builtins", "cc_binary_host"),
            ]),
            parse_build_ninja::<CmakeNinjaTarget>(&build_path)?,
            &self.src_path,
            Path::new("<no_sdk>"),
            &build_path,
            None,
            self,
            ctx,
        )?
        .print(ctx)
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
                let mut includes = Vec::new();
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
                    includes.push(local_include);
                }
                module.update_prop("cmd", |_| Ok(SoongProp::Str(cmd.clone())))?;
                module.update_prop("srcs", |prop| {
                    Ok(match prop {
                        SoongProp::VecStr(mut srcs) => {
                            if includes.len() > 0 {
                                srcs.push(String::from("CMakeLists.txt"));
                            }
                            for include in &includes {
                                srcs.push(path_to_string(include) + "/*");
                                srcs.push(path_to_string(include) + "/**/*");
                            }
                            SoongProp::VecStr(srcs)
                        }
                        _ => prop,
                    })
                })?;
                module
            }
            _ => module,
        })
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_include(&self, _include: &Path) -> bool {
        false
    }
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
}
