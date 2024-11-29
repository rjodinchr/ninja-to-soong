use std::collections::HashMap;
use std::collections::HashSet;

use crate::project::Project;
use crate::soongmodule::SoongModule;
use crate::target::BuildTarget;
use crate::utils::*;

#[derive(Debug)]
pub struct SoongPackage<'a> {
    package: String,
    generated_deps: HashSet<String>,
    include_directories: HashSet<String>,
    src_root: &'a str,
    ndk_root: &'a str,
    build_root: &'a str,
    target_prefix: &'a str,
}

impl<'a> SoongPackage<'a> {
    pub fn new(
        src_root: &'a str,
        ndk_root: &'a str,
        build_root: &'a str,
        target_prefix: &'a str,
    ) -> Self {
        SoongPackage {
            package: String::new(),
            generated_deps: HashSet::new(),
            include_directories: HashSet::new(),
            src_root,
            ndk_root,
            build_root,
            target_prefix,
        }
    }

    pub fn add_module(&mut self, module: SoongModule) -> Result<(), String> {
        self.package += &match module.print() {
            Ok(module) => module,
            Err(err) => return Err(err),
        };
        return Ok(());
    }

    pub fn write(self, path: &str) -> Result<String, String> {
        crate::filesystem::write_file(&(path.to_string() + "/Android.bp"), self.package)
    }

    pub fn get_generated_deps(&self) -> HashSet<String> {
        self.generated_deps.to_owned()
    }
    pub fn get_include_directories(&self) -> HashSet<String> {
        self.include_directories.to_owned()
    }

    fn generate_object(
        &mut self,
        name: &str,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        project: &dyn Project,
    ) -> Result<String, String> {
        let mut defines = project.get_default_defines();
        let mut includes: HashSet<String> = HashSet::new();
        let mut srcs: HashSet<String> = HashSet::new();
        for input in target.get_inputs() {
            let Some(target) = targets_map.get(input) else {
                return error!(format!("unsupported input for library: {input}"));
            };
            let (src, src_includes, src_defines) =
                match target.get_compiler_target_info(self.src_root, project) {
                    Ok(return_values) => return_values,
                    Err(err) => return Err(err),
                };
            for inc in src_includes {
                includes.insert(inc.clone());
                self.include_directories.insert(inc);
            }
            for def in src_defines {
                defines.insert(String::from("-D") + &def);
            }
            srcs.insert(src);
        }

        let (version_script, link_flags) = target.get_link_flags(self.src_root, project);

        let (static_libs, shared_libs) = match target.get_link_libraries(self.ndk_root, project) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };

        let generated_headers = match target.get_generated_headers(targets_map) {
            Ok(return_value) => return_value,
            Err(err) => return Err(err),
        };
        self.generated_deps
            .extend(project.get_headers_to_copy(&generated_headers).into_iter());
        let generated_headers_filtered_raw = project.get_headers_to_generate(&generated_headers);
        let mut generated_headers_filtered = HashSet::new();
        for header in generated_headers_filtered_raw {
            generated_headers_filtered.insert(match targets_map.get(&header) {
                Some(target_header) => target_header.get_name(self.target_prefix),
                None => return error!(format!("Could not find target for '{header}'")),
            });
        }
        let target_name = target.get_name(self.target_prefix);

        let mut module = crate::soongmodule::SoongModule::new(name);
        module.add_str("name", target_name.clone());
        if target_name == "clvk_libOpenCL_so" {
            module.add_str("stem", "libclvk".to_string());
        }
        if name == "cc_library_static" {
            module.add_bool("optimize_for_size", true);
        }
        module.add_bool("use_clang_lld", true);
        module.add_set("srcs", srcs);
        module.add_set("local_include_dirs", includes);
        module.add_set("cflags", defines);
        module.add_set("ldflags", link_flags);
        module.add_str("version_script", version_script);
        module.add_set("static_libs", static_libs);
        module.add_set("shared_libs", shared_libs);
        module.add_set("generated_headers", generated_headers_filtered);
        module.add_set("header_libs", project.get_object_header_libs());
        return module.print();
    }

    fn replace_output_in_command(
        command: String,
        output: &String,
        project: &dyn Project,
    ) -> String {
        let marker = "<output>";
        let space_and_marker = String::from(" ") + marker;
        let space_and_last_output = String::from(" ") + output.split("/").last().unwrap();
        let command = command.replace(output, marker);
        let command = command.replace(&space_and_last_output, &space_and_marker);
        let replace_output =
            String::from("$(location ") + &project.rework_output_path(output) + ")";
        return command.replace(marker, &replace_output);
    }
    fn replace_input_in_command(&self, command: String, input: String) -> String {
        let replace_input = String::from("$(location ")
            + &input.replace(&add_slash_suffix(self.src_root), "")
            + ")";
        return command.replace(&input, &replace_input);
    }
    fn replace_dep_in_command(
        &self,
        command: String,
        tool: String,
        tool_target_name: String,
        prefix: &str,
    ) -> String {
        let replace_tool = "$(location ".to_string() + &tool_target_name + ")";
        let tool_with_prefix = String::from(prefix) + &tool;
        return command
            .replace(&tool_with_prefix, &replace_tool)
            .replace(&tool, &replace_tool);
    }

    fn rework_command(
        &self,
        command: String,
        inputs: HashSet<String>,
        outputs: &Vec<String>,
        generated_deps: HashSet<(String, String)>,
        project: &dyn Project,
    ) -> String {
        let mut command = command.replace("/usr/bin/python3 ", "");
        command = command.replace(&(self.build_root.to_string() + "/"), "");
        for output in outputs {
            command = Self::replace_output_in_command(command, output, project);
        }
        for input in inputs.clone() {
            command = self.replace_input_in_command(command, input);
        }
        for (tool, tool_target_name) in generated_deps {
            command =
                self.replace_dep_in_command(command, tool, tool_target_name, self.target_prefix);
        }
        return command;
    }

    fn generate_custom_command(
        &mut self,
        target: &BuildTarget,
        command: String,
        project: &dyn Project,
    ) -> Result<String, String> {
        let (srcs_set, inputs, generated_deps) =
            match project.parse_custom_command_inputs(target.get_inputs()) {
                Ok(return_values) => return_values,
                Err(err) => return Err(err),
            };
        for (dep, _) in &generated_deps {
            self.generated_deps.insert(dep.clone());
        }
        let target_outputs = target.get_outputs();
        let out_set = target_outputs
            .into_iter()
            .fold(HashSet::new(), |mut set, output| {
                set.insert(project.rework_output_path(output));
                set
            });

        let command = self.rework_command(command, inputs, target_outputs, generated_deps, project);

        let mut module = crate::soongmodule::SoongModule::new("cc_genrule");
        module.add_str("name", target.get_name(self.target_prefix));
        module.add_set("srcs", srcs_set);
        module.add_set("out", out_set);
        module.add_str("cmd", command.to_string());
        return module.print();
    }

    fn generate_target(
        &mut self,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        project: &dyn Project,
    ) -> Result<(), String> {
        let rule = target.get_rule();
        let result = if rule.starts_with("CXX_SHARED_LIBRARY") {
            self.generate_object("cc_library_shared", target, targets_map, project)
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            self.generate_object("cc_library_static", target, targets_map, project)
        } else if rule.starts_with("CXX_EXECUTABLE") {
            self.generate_object("cc_binary", target, targets_map, project)
        } else if rule.starts_with("CUSTOM_COMMAND") {
            let command = match target.get_command() {
                Ok(option) => match option {
                    Some(command) => command,
                    None => return Ok(()),
                },
                Err(err) => return Err(err),
            };
            self.generate_custom_command(target, command, project)
        } else if rule.starts_with("CXX_COMPILER")
            || rule.starts_with("C_COMPILER")
            || rule.starts_with("ASM_COMPILER")
            || rule == "phony"
        {
            return Ok(());
        } else {
            error!(format!("unsupported rule ({rule}) for target: {target:#?}"))
        };
        match result {
            Ok(module) => {
                self.package += &module;
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }

    fn create_map(targets: &Vec<BuildTarget>) -> HashMap<String, &BuildTarget> {
        let mut map: HashMap<String, &BuildTarget> = HashMap::new();
        for target in targets {
            for output in &target.get_all_outputs() {
                map.insert(output.clone(), target);
            }
        }

        return map;
    }

    pub fn generate(
        &mut self,
        entry_targets: Vec<&str>,
        targets: Vec<BuildTarget>,
        project: &dyn Project,
    ) -> Result<(), String> {
        let mut target_seen: HashSet<String> = HashSet::new();
        let mut target_to_generate =
            entry_targets
                .into_iter()
                .fold(Vec::new(), |mut vec, element| {
                    vec.push(element.to_string());
                    vec
                });

        let targets_map = Self::create_map(&targets);

        while let Some(input) = target_to_generate.pop() {
            if target_seen.contains(&input) || project.ignore_target(&input) {
                continue;
            }
            let Some(target) = targets_map.get(&input) else {
                continue;
            };

            target_to_generate.append(&mut target.get_all_inputs());
            for output in target.get_all_outputs() {
                target_seen.insert(output);
            }

            if let Err(err) = self.generate_target(target, &targets_map, project) {
                return Err(err);
            }
        }
        return Ok(());
    }
}
