// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::str;

use crate::ninja_target::*;
use crate::project::*;
use crate::soong_module::*;
use crate::utils::*;

pub struct SoongModuleGenerator<'a, T>
where
    T: NinjaTarget,
{
    src_path: &'a Path,
    ndk_path: &'a Path,
    build_path: &'a Path,
    gen_deps: &'a mut Vec<PathBuf>,
    gen_libs: &'a mut Vec<PathBuf>,
    targets_map: &'a NinjaTargetsMap<'a, T>,
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
        gen_deps: &'a mut Vec<PathBuf>,
        gen_libs: &'a mut Vec<PathBuf>,
        targets_map: &'a NinjaTargetsMap<'a, T>,
        project: &'a dyn Project,
    ) -> Self {
        Self {
            src_path,
            ndk_path,
            build_path,
            gen_deps,
            gen_libs,
            targets_map,
            project,
        }
    }

    fn get_defines(&self, defines: Vec<String>) -> Vec<String> {
        defines
            .iter()
            .filter(|def| {
                debug_project!("filter_define({def})");
                self.project.filter_define(def)
            })
            .map(|def| format!("-D{0}", self.project.map_define(def)))
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
        includes
            .iter()
            .filter(|include| {
                debug_project!("filter_include({include:#?})");
                self.project.filter_include(include)
            })
            .map(|inc| path_to_string(strip_prefix(self.project.map_include(&inc), self.src_path)))
            .collect()
    }
    fn get_libs(&mut self, libs: Vec<PathBuf>, module_name: &String) -> Vec<String> {
        libs.iter()
            .filter(|lib| {
                debug_project!("filter_lib({lib:#?})");
                self.project.filter_lib(&path_to_string(lib))
            })
            .map(|lib| {
                if lib.starts_with(&self.ndk_path) {
                    file_stem(lib)
                } else {
                    self.gen_libs.push(lib.clone());
                    path_to_id(self.project.get_target_name(&self.project.map_lib(&lib)))
                }
            })
            .filter(|lib| lib != module_name)
            .collect()
    }
    pub fn generate_object(&mut self, name: &str, target: &T) -> Result<SoongModule, String> {
        let target_name = target.get_name(self.project.get_name());
        let module_name = path_to_id(self.project.get_target_name(&target_name));
        let mut cflags = Vec::new();
        let mut includes = Vec::new();
        let mut sources = Vec::new();
        let mut static_libs = Vec::new();
        let mut shared_libs = Vec::new();
        let mut gen_deps = Vec::new();
        for input in target.get_inputs() {
            let Some(input_target) = self.targets_map.get(input) else {
                return error!("unsupported input for library: {input:#?}");
            };

            sources.extend(
                input_target
                    .get_sources(self.build_path)?
                    .iter()
                    .filter(|source| {
                        debug_project!("filter_source({source:#?})");
                        self.project.filter_source(source)
                    })
                    .map(|source| {
                        if source.starts_with(self.build_path) {
                            gen_deps.push(strip_prefix(&source, self.build_path));
                        }
                        path_to_string(strip_prefix(
                            self.project.map_source(&source),
                            self.src_path,
                        ))
                    }),
            );

            let (static_libraries, shared_libraries) = input_target.get_link_libraries()?;
            static_libs.extend(self.get_libs(static_libraries, &module_name));
            shared_libs.extend(self.get_libs(shared_libraries, &module_name));

            includes.extend(self.get_includes(input_target.get_includes(self.build_path)));
            cflags.extend(self.get_defines(input_target.get_defines()));
            cflags.extend(self.get_cflags(input_target.get_cflags()));
        }

        includes.extend(self.get_includes(target.get_includes(self.build_path)));
        cflags.extend(self.get_defines(target.get_defines()));
        cflags.extend(self.get_cflags(target.get_cflags()));
        cflags.extend(self.project.extend_cflags(&target_name));

        let (version_script, link_flags) = target.get_link_flags();
        let link_flags = link_flags
            .into_iter()
            .filter(|flag| {
                debug_project!("filter_link_flag({flag})");
                self.project.filter_link_flag(flag)
            })
            .collect();
        let (static_libraries, shared_libraries) = target.get_link_libraries()?;
        static_libs.extend(self.get_libs(static_libraries, &module_name));
        shared_libs.extend(self.get_libs(shared_libraries, &module_name));
        shared_libs.extend(self.project.extend_shared_libs(&target_name));

        let gen_headers = self
            .targets_map
            .traverse_from(
                target.get_outputs().clone(),
                Vec::new(),
                |gen_headers, rule, target| match rule {
                    NinjaRule::CustomCommand => {
                        if target.get_cmd()?.is_none() {
                            return Ok(());
                        }
                        gen_headers.extend(target.get_outputs().clone());
                        return Ok(());
                    }
                    _ => return Ok(()),
                },
                |_target_name| true,
            )?
            .iter()
            .filter(|header| {
                debug_project!("filter_gen_header({header:#?})");
                if !self.project.filter_gen_header(header) {
                    gen_deps.push(PathBuf::from(header));
                    false
                } else {
                    self.targets_map.get(&header).is_some()
                }
            })
            .map(|header| {
                path_to_id(
                    self.targets_map
                        .get(&header)
                        .unwrap()
                        .get_name(self.project.get_name()),
                )
            })
            .collect();

        self.gen_deps.extend(gen_deps);

        let mut module = SoongModule::new(name);
        module.add_prop("name", SoongProp::Str(module_name));
        if let Some(stem) = self.project.get_target_stem(&target_name) {
            module.add_prop("stem", SoongProp::Str(stem));
        }
        if let Some(vs) = version_script {
            module.add_prop(
                "version_script",
                SoongProp::Str(path_to_string(strip_prefix(vs, &self.src_path))),
            );
        }
        module.add_prop("srcs", SoongProp::VecStr(sources));
        module.add_prop("cflags", SoongProp::VecStr(cflags));
        module.add_prop("ldflags", SoongProp::VecStr(link_flags));
        module.add_prop("shared_libs", SoongProp::VecStr(shared_libs));
        module.add_prop("static_libs", SoongProp::VecStr(static_libs));
        module.add_prop("local_include_dirs", SoongProp::VecStr(includes));
        module.add_prop("generated_headers", SoongProp::VecStr(gen_headers));

        Ok(self.project.get_target_module(&target_name, module))
    }

    fn get_cmd(
        &self,
        rule_cmd: NinjaRuleCmd,
        inputs: Vec<PathBuf>,
        outputs: &Vec<PathBuf>,
        deps: HashMap<PathBuf, String>,
    ) -> String {
        let mut cmd = rule_cmd.command;
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
        for output in outputs {
            let marker = "<output>";
            let replace_output = path_to_string(self.project.map_cmd_output(output));
            cmd = cmd
                .replace(&path_to_string(output), marker)
                .replace(&format!(" {0}", file_name(output)), &format!(" {marker}"))
                .replace(marker, &format!("$(location {replace_output})"));
        }
        for input in inputs {
            let replace_input = path_to_string(strip_prefix(
                canonicalize_path(&input, self.build_path),
                self.src_path,
            ));
            cmd = cmd.replace(
                &path_to_string(&input),
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
            let rsp = format!("$(genDir)/{rsp_file}");
            cmd = format!(
                "echo \\\"{0}\\\" > {rsp} && {cmd}",
                rsp_content
                    .split(" ")
                    .filter(|file| !file.is_empty())
                    .map(|file| {
                        let file_path = path_to_string(strip_prefix(
                            canonicalize_path(file, self.build_path),
                            self.src_path,
                        ));
                        format!("$(location {file_path})",)
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            )
            .replace("${rspfile}", &rsp);
        }
        cmd
    }
    fn get_cmd_inputs(
        &self,
        inputs: &Vec<PathBuf>,
        deps: &mut HashMap<PathBuf, String>,
    ) -> Vec<PathBuf> {
        inputs
            .iter()
            .filter(|input| {
                for (prefix, dep) in self.project.get_deps_prefix() {
                    if input.starts_with(&prefix) {
                        deps.insert(
                            PathBuf::from(input),
                            dep.get_id(input, &prefix, self.build_path),
                        );
                        return false;
                    }
                }
                if canonicalize_path(&input, self.build_path).starts_with(self.build_path) {
                    deps.insert(
                        PathBuf::from(input),
                        path_to_id(Path::new(self.project.get_name()).join(input)),
                    );
                    return false;
                }
                true
            })
            .map(|input| PathBuf::from(input))
            .collect()
    }
    pub fn generate_custom_command(
        &mut self,
        target: &T,
        rule_cmd: NinjaRuleCmd,
    ) -> Result<SoongModule, String> {
        let mut inputs = Vec::new();
        let mut deps = HashMap::new();
        inputs.extend(self.get_cmd_inputs(target.get_inputs(), &mut deps));
        inputs.extend(self.get_cmd_inputs(target.get_implicit_deps(), &mut deps));
        let mut sources = inputs
            .clone()
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
            self.gen_deps.push(dep.clone());
        }
        let target_outputs = target.get_outputs();
        let cmd = self.get_cmd(rule_cmd, inputs, target_outputs, deps);
        let outputs = target_outputs
            .iter()
            .map(|output| path_to_string(self.project.map_cmd_output(output)))
            .collect();
        let module_name = path_to_id(target.get_name(self.project.get_name()));

        let mut module = SoongModule::new("cc_genrule");
        module.add_prop("name", SoongProp::Str(module_name));
        module.add_prop("cmd", SoongProp::Str(cmd));
        module.add_prop("srcs", SoongProp::VecStr(sources));
        module.add_prop("out", SoongProp::VecStr(outputs));

        Ok(module)
    }
}
