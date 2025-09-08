// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::str;

use crate::context::*;
use crate::ninja_target::*;
use crate::project::*;
use crate::soong_module::*;
use crate::utils::*;

#[derive(Default)]
pub struct SoongModuleGeneratorInternals {
    pub deps: Vec<PathBuf>,
    pub libs: Vec<PathBuf>,
    python_binaries: std::collections::HashSet<String>,
}

pub struct SoongModuleGenerator<'a, T>
where
    T: NinjaTarget,
{
    internals: SoongModuleGeneratorInternals,
    src_path: &'a Path,
    ndk_path: &'a Path,
    build_path: &'a Path,
    gen_build_prefix: Option<&'a str>,
    targets_map: &'a NinjaTargetsMap<'a, T>,
    targets_to_gen: &'a NinjaTargetsToGenMap,
    project: &'a dyn Project,
}

impl<'a, T> SoongModuleGenerator<'a, T>
where
    T: NinjaTarget,
{
    pub fn new(
        src_path: &'a Path,
        ndk_path: &'a Path,
        build_path: &'a Path,
        gen_build_prefix: Option<&'a str>,
        targets_map: &'a NinjaTargetsMap<'a, T>,
        targets_to_gen: &'a NinjaTargetsToGenMap,
        project: &'a dyn Project,
    ) -> Self {
        Self {
            internals: SoongModuleGeneratorInternals::default(),
            src_path,
            ndk_path,
            build_path,
            gen_build_prefix,
            targets_map,
            targets_to_gen,
            project,
        }
    }
    pub fn delete(self) -> SoongModuleGeneratorInternals {
        self.internals
    }

    pub fn filter_target(&self, target: &T) -> bool {
        let target_name = target.get_name();
        debug_project!("filter_target({target_name:#?})");
        self.project.filter_target(&target_name)
    }

    fn replace_path(&self, iter: impl Iterator<Item = String>) -> Vec<String> {
        let iter = iter.map(|path| {
            path.replace(&path_to_string_with_separator(self.src_path), "")
                .replace(&path_to_string(self.src_path), "")
        });
        if let Some(prefix) = self.gen_build_prefix {
            iter.map(|path| path.replace(&path_to_string(self.build_path), prefix))
                .collect()
        } else {
            iter.collect()
        }
    }
    fn get_sources(&self, sources: Vec<PathBuf>) -> Vec<String> {
        self.replace_path(sources.iter().filter_map(|source| {
            debug_project!("filter_source({source:#?})");
            if !self.project.filter_source(source) {
                return None;
            }
            Some(path_to_string(source))
        }))
    }
    fn get_defines(&self, defines: Vec<String>) -> Vec<String> {
        self.replace_path(defines.into_iter().filter(|def| {
            debug_project!("filter_define({def})");
            self.project.filter_define(&def)
        }))
        .iter()
        .map(|def| format!("-D{def}"))
        .collect()
    }
    fn get_cflags(&self, cflags: Vec<String>) -> Vec<String> {
        cflags
            .into_iter()
            .filter(|cflag| {
                debug_project!("filter_cflags({cflag})");
                self.project.filter_cflag(cflag)
            })
            .collect()
    }
    fn get_includes(&self, includes: Vec<PathBuf>) -> Vec<String> {
        self.replace_path(includes.iter().filter_map(|include| {
            debug_project!("filter_include({include:#?})");
            if !self.project.filter_include(include) {
                return None;
            }
            Some(path_to_string(include))
        }))
    }
    fn get_libs(&mut self, libs: Vec<PathBuf>, module_name: &String) -> Vec<String> {
        libs.into_iter()
            .filter_map(|lib| {
                debug_project!("filter_lib({lib:#?})");
                if !self.project.filter_lib(&path_to_string(&lib)) {
                    return None;
                }
                Some(if lib.starts_with(&self.ndk_path) {
                    file_stem(&lib)
                } else {
                    let lib_id = path_to_id(match self.project.map_lib(&lib) {
                        Some(map_lib) => match self.targets_to_gen.get_name(&map_lib) {
                            Some(name) => name,
                            None => map_lib,
                        },
                        None => match self.targets_to_gen.get_name(&lib) {
                            Some(name) => name,
                            None => Path::new(self.project.get_name()).join(&lib),
                        },
                    });
                    if lib_id == *module_name {
                        return None;
                    }
                    self.internals.libs.push(lib);
                    lib_id
                })
            })
            .collect()
    }
    fn get_link_flags(&self, link_flags: Vec<String>) -> Vec<String> {
        link_flags
            .into_iter()
            .filter(|flag| {
                debug_project!("filter_link_flag({flag})");
                self.project.filter_link_flag(flag)
            })
            .collect()
    }
    fn get_generated_headers(&mut self, target: &T) -> Result<Vec<String>, String> {
        let mut gen_headers = Vec::new();
        self.targets_map
            .traverse_from(target.get_outputs().clone(), |target| {
                match target.get_rule()? {
                    NinjaRule::CustomCommand(_) => {
                        gen_headers.extend(target.get_outputs().clone());
                        Ok(true)
                    }
                    _ => Ok(true),
                }
            })?;
        Ok(gen_headers
            .iter()
            .filter_map(|header| {
                debug_project!("filter_gen_header({header:#?})");
                if !self.project.filter_gen_header(header) {
                    self.internals.deps.push(PathBuf::from(header));
                    return None;
                } else if self.targets_map.get(header).is_none() {
                    return None;
                }
                Some(match self.targets_to_gen.get_name(header) {
                    Some(name) => path_to_string(name),
                    None => path_to_id(
                        Path::new(self.project.get_name())
                            .join(self.targets_map.get(header).unwrap().get_name()),
                    ),
                })
            })
            .collect())
    }
    fn defines_conflict(
        defines: &mut std::collections::HashMap<String, String>,
        cflags: &Vec<String>,
    ) -> bool {
        let new_defines = cflags
            .iter()
            .filter_map(|cflag| {
                let Some(def) = cflag.strip_prefix("-D") else {
                    return None;
                };
                let Some((var, val)) = def.split_once("=") else {
                    return None;
                };
                Some((String::from(var), String::from(val)))
            })
            .collect::<Vec<_>>();
        if new_defines.iter().any(|(var, val)| {
            let Some(ref_val) = defines.get(var) else {
                return false;
            };
            val != ref_val
        }) {
            return true;
        }
        defines.extend(new_defines);
        false
    }
    pub fn generate_object(
        &mut self,
        module_type: &str,
        target: &T,
        ctx: &Context,
    ) -> Result<Vec<SoongModule>, String> {
        let target_name = target.get_name();
        let module_name = path_to_id(match self.targets_to_gen.get_name(&target_name) {
            Some(name) => name,
            None => Path::new(self.project.get_name()).join(&target_name),
        });
        let mut modules = Vec::new();
        let mut cflags = Vec::new();
        let mut includes = Vec::new();
        let mut sources = Vec::new();
        let mut whole_static_libs = Vec::new();
        let mut static_libs = Vec::new();
        let mut shared_libs = Vec::new();
        let mut defines = std::collections::HashMap::new();
        for input in target.get_inputs() {
            let Some(input_target) = self.targets_map.get(input) else {
                sources.push(path_to_string(strip_prefix(
                    canonicalize_path(input, self.build_path),
                    self.src_path,
                )));
                continue;
            };

            let mut input_cflags = self.get_defines(input_target.get_defines());
            input_cflags.extend(self.get_cflags(input_target.get_cflags()));
            if self.project.filter_input_target(input)
                && !Self::defines_conflict(&mut defines, &input_cflags)
            {
                whole_static_libs
                    .extend(self.get_libs(input_target.get_libs_static_whole(), &module_name));
                static_libs.extend(self.get_libs(input_target.get_libs_static(), &module_name));
                shared_libs.extend(self.get_libs(input_target.get_libs_shared(), &module_name));
                sources.extend(self.get_sources(input_target.get_sources(self.build_path)?));
                includes.extend(self.get_includes(input_target.get_includes(self.build_path)));
                cflags.extend(input_cflags);
            } else {
                modules.extend(self.generate_object("cc_library_static", input_target, ctx)?);
                whole_static_libs.push(path_to_id(Path::new(self.project.get_name()).join(input)));
                continue;
            }
        }
        includes.extend(self.get_includes(target.get_includes(self.build_path)));
        cflags.extend(self.get_defines(target.get_defines()));
        cflags.extend(self.get_cflags(target.get_cflags()));

        let generated_headers = self.get_generated_headers(target)?;
        let (version_script, link_flags) = target.get_link_flags();
        let link_flags = self.get_link_flags(link_flags);
        whole_static_libs.extend(self.get_libs(target.get_libs_static_whole(), &module_name));
        static_libs.extend(self.get_libs(target.get_libs_static(), &module_name));
        shared_libs.extend(self.get_libs(target.get_libs_shared(), &module_name));

        let module_type = match self.targets_to_gen.get_module_name(&target_name) {
            Some(module_type) => module_type,
            None => String::from(module_type),
        };
        let mut module =
            SoongModule::new(&module_type).add_prop("name", SoongProp::Str(module_name));
        if let Some(stem) = self.targets_to_gen.get_stem(&target_name) {
            module = module.add_prop("stem", SoongProp::Str(stem));
        }
        if let Some(vs) = version_script {
            module = module.add_prop(
                "version_script",
                SoongProp::Str(path_to_string(strip_prefix(vs, &self.src_path))),
            );
        }
        let mut srcs_prop = SoongNamedProp::new("srcs", SoongProp::VecStr(sources));
        if ctx.wildcardize_paths {
            srcs_prop.enable_wildcard(&self.src_path)?;
        }
        module = module
            .add_named_prop(srcs_prop)
            .add_prop("cflags", SoongProp::VecStr(cflags))
            .add_prop("ldflags", SoongProp::VecStr(link_flags))
            .add_prop("shared_libs", SoongProp::VecStr(shared_libs))
            .add_prop("static_libs", SoongProp::VecStr(static_libs))
            .add_prop("whole_static_libs", SoongProp::VecStr(whole_static_libs))
            .add_prop("local_include_dirs", SoongProp::VecStr(includes))
            .add_prop("generated_headers", SoongProp::VecStr(generated_headers));

        modules.push(self.project.extend_module(&target_name, module)?);
        Ok(modules)
    }

    fn get_cmd(
        &self,
        mut cmd: String,
        rule_cmd: NinjaRuleCmd,
        inputs: Vec<PathBuf>,
        outputs: &Vec<PathBuf>,
        deps: Vec<(PathBuf, String)>,
    ) -> String {
        for output in outputs {
            let marker = "<output>";
            let replace_output = path_to_string(self.project.map_cmd_output(output));
            cmd = cmd
                .replace(&path_to_string(output), marker)
                .replace(&format!(" {0}", file_name(output)), &format!(" {marker}"))
                .replace(marker, &format!("$(location {replace_output})"));
        }
        for input in &inputs {
            let replace_input = path_to_string(strip_prefix(
                canonicalize_path(input, self.build_path),
                self.src_path,
            ));
            cmd = cmd.replace(
                &path_to_string(input),
                &format!("$(location {replace_input})"),
            )
        }
        for (dep, dep_target_name) in deps {
            cmd = cmd.replace(
                &path_to_string(&dep),
                &format!("$(location :{dep_target_name})"),
            )
        }
        if let Some((rsp_file, rsp_content)) = rule_cmd.rsp_info {
            let rsp_inputs = rsp_content
                .split(" ")
                .filter_map(|file| {
                    if file.is_empty() {
                        return None;
                    }
                    Some(PathBuf::from(file))
                })
                .collect::<Vec<PathBuf>>();
            let rsp_inputs_string = if inputs == rsp_inputs {
                String::from("$(in)")
            } else {
                rsp_inputs
                    .iter()
                    .map(|file| {
                        let file_path = path_to_string(strip_prefix(
                            canonicalize_path(file, self.build_path),
                            self.src_path,
                        ));
                        format!("$(location {file_path})")
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            };
            let rsp = format!("$(genDir)/{rsp_file}");
            cmd = format!("echo \\\"{rsp_inputs_string}\\\" > {rsp} && {cmd}")
                .replace("${rspfile}", &rsp);
        }
        cmd
    }
    fn get_cmd_inputs(
        &self,
        inputs: Vec<PathBuf>,
        deps: &mut Vec<(PathBuf, String)>,
    ) -> Vec<PathBuf> {
        inputs
            .into_iter()
            .filter_map(|input| {
                for (prefix, dep) in self.project.get_deps_prefix() {
                    if input.starts_with(&prefix) {
                        let dep_id = dep.get_id(&input, &prefix, self.build_path);
                        deps.push((input, dep_id));
                        return None;
                    }
                }
                if canonicalize_path(&input, self.build_path).starts_with(self.build_path) {
                    let dep_id = path_to_id(Path::new(self.project.get_name()).join(&input));
                    deps.push((input, dep_id));
                    return None;
                }
                Some(input)
            })
            .collect()
    }
    fn get_tool_module(
        &mut self,
        tool: &str,
    ) -> Result<Option<(String, Option<Vec<SoongModule>>)>, String> {
        if !tool.ends_with(".py") {
            return Ok(None);
        }
        let tool_module = path_to_id(Path::new(self.project.get_name()).join(&tool));
        if !self.internals.python_binaries.contains(&tool_module) {
            let Some(module) = self.project.extend_python_binary_host(
                &self.src_path.join(&tool),
                SoongModule::new("python_binary_host")
                    .add_prop("name", SoongProp::Str(tool_module.clone()))
                    .add_prop("main", SoongProp::Str(String::from(tool)))
                    .add_prop("srcs", SoongProp::VecStr(vec![String::from(tool)])),
            )?
            else {
                return Ok(None);
            };
            self.internals.python_binaries.insert(tool_module.clone());
            return Ok(Some((tool_module, Some(vec![module]))));
        }
        Ok(Some((tool_module, None)))
    }
    fn get_tools(
        &mut self,
        mut cmd: String,
        inputs: &mut Vec<PathBuf>,
    ) -> Result<(Vec<String>, Vec<String>, Vec<SoongModule>, String), String> {
        if cmd.starts_with("cp") {
            return Ok((Vec::new(), Vec::new(), Vec::new(), cmd));
        }
        while let Some(index) = cmd.find("python") {
            let begin = str::from_utf8(&cmd.as_bytes()[0..index])
                .unwrap()
                .rfind(" ")
                .unwrap_or_default();
            cmd = match str::from_utf8(&cmd.as_bytes()[index..]).unwrap().find(" ") {
                Some(end) => cmd.replace(
                    str::from_utf8(&cmd.as_bytes()[begin..index + end + 1]).unwrap(),
                    "",
                ),
                None => cmd.replace(str::from_utf8(&cmd.as_bytes()[begin..]).unwrap(), ""),
            };
        }
        cmd = cmd.replace(&path_to_string_with_separator(self.build_path), "");
        let tool_location = String::from("$(location) ");
        let (tool, mut cmd) = if let Some((tool, cmd)) = cmd.split_once(" ") {
            (String::from(tool), tool_location + cmd)
        } else {
            (String::from(cmd), tool_location)
        };
        for idx in 0..inputs.len() {
            if path_to_string(&inputs[idx]) == tool {
                inputs.remove(idx);
                break;
            }
        }
        *inputs = inputs
            .into_iter()
            .filter(|input| !input.ends_with("python3"))
            .map(|input| input.clone())
            .collect();
        if tool.ends_with(".py") {
            cmd = String::from("python3 ") + &cmd;
        }
        let tool = path_to_string(strip_prefix(
            canonicalize_path(&tool, self.build_path),
            self.src_path,
        ));

        if let Some((tool_module, some_modules)) = self.get_tool_module(&tool)? {
            if let Some(modules) = some_modules {
                Ok((Vec::new(), vec![tool_module], modules, cmd))
            } else {
                Ok((Vec::new(), vec![tool_module], Vec::new(), cmd))
            }
        } else {
            Ok((vec![tool], Vec::new(), Vec::new(), cmd))
        }
    }
    pub fn generate_custom_command(
        &mut self,
        target: &T,
        rule_cmd: NinjaRuleCmd,
    ) -> Result<Vec<SoongModule>, String> {
        let mut inputs = Vec::new();
        let mut deps = Vec::new();
        inputs.extend(self.get_cmd_inputs(target.get_inputs().clone(), &mut deps));
        inputs.extend(self.get_cmd_inputs(target.get_implicit_deps().clone(), &mut deps));
        let (tool_files, tool_modules, mut modules, cmd) =
            self.get_tools(rule_cmd.command.clone(), &mut inputs)?;
        let mut sources = inputs
            .iter()
            .map(|input| {
                path_to_string(strip_prefix(
                    canonicalize_path(input, self.build_path),
                    self.src_path,
                ))
            })
            .collect::<Vec<String>>();
        for (dep, dep_target_name) in &deps {
            sources.push(format!(":{dep_target_name}"));
            self.internals.deps.push(dep.clone());
        }
        let target_outputs = target.get_outputs();
        let cmd = self.get_cmd(cmd, rule_cmd, inputs, target_outputs, deps);
        let outputs = target_outputs
            .iter()
            .map(|output| path_to_string(self.project.map_cmd_output(output)))
            .collect();
        let target_name = target.get_name();
        let module_name = match self.targets_to_gen.get_name(&target_name) {
            Some(name) => path_to_string(name),
            None => path_to_id(Path::new(self.project.get_name()).join(target_name)),
        };

        modules.push(
            self.project.extend_custom_command(
                &target.get_name(),
                SoongModule::new("cc_genrule")
                    .add_prop("name", SoongProp::Str(module_name))
                    .add_prop("cmd", SoongProp::Str(cmd))
                    .add_prop("srcs", SoongProp::VecStr(sources))
                    .add_prop("out", SoongProp::VecStr(outputs))
                    .add_prop("tools", SoongProp::VecStr(tool_modules))
                    .add_prop("tool_files", SoongProp::VecStr(tool_files)),
            )?,
        );
        Ok(modules)
    }
}
