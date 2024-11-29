use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use crate::ninja_target::NinjaTarget;
use crate::project::Project;
use crate::soong_module::SoongModule;
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
        default_visibility: &str,
        license_kinds: &str,
        license_text: &str,
    ) -> Self {
        let mut package = SoongPackage {
            package: String::new(),
            generated_deps: HashSet::new(),
            include_directories: HashSet::new(),
            src_root,
            ndk_root,
            build_root,
            target_prefix,
        };

        let license_name =
            target_prefix.to_string() + &license_text.replace(".", "_").to_lowercase();

        let mut package_module = SoongModule::new("package");
        package_module.add_set("default_applicable_licenses", [license_name.clone()].into());
        package_module.add_set(
            "default_visibility",
            [default_visibility.to_string()].into(),
        );
        package.add_module(package_module);

        let mut license_module = SoongModule::new("license");
        license_module.add_str("name", license_name.clone());
        license_module.add_set("visibility", [":__subpackages__".to_string()].into());
        license_module.add_set("license_kinds", [license_kinds.to_string()].into());
        license_module.add_set("license_text", [license_text.to_string()].into());
        package.add_module(license_module);

        return package;
    }

    pub fn add_module(&mut self, module: SoongModule) {
        self.package += &module.print();
    }

    pub fn write(self) -> Result<String, String> {
        let dir_path = self.src_root;
        let file_path = dir_path.to_string() + "/Android.bp";
        match File::create(&file_path) {
            Ok(mut file) => {
                if let Err(err) = file.write_fmt(format_args!("{0}", self.package)) {
                    return error!(format!("Could not write into '{dir_path}': '{err:#?}"));
                }
            }
            Err(err) => {
                return error!(format!("Could not create '{file_path}': '{err}'"));
            }
        }
        return Ok(format!("'{file_path}' created successfully!"));
    }

    pub fn get_generated_deps(&self) -> HashSet<String> {
        self.generated_deps.to_owned()
    }
    pub fn get_include_directories(&self) -> HashSet<String> {
        self.include_directories.to_owned()
    }

    fn generate_library(
        &mut self,
        name: &str,
        target: &NinjaTarget,
        target_map: &HashMap<String, &NinjaTarget>,
        project: &dyn Project,
    ) -> Result<String, String> {
        let mut cflags = project.get_default_cflags();
        let mut includes: HashSet<String> = HashSet::new();
        let mut srcs: HashSet<String> = HashSet::new();
        for input in target.get_inputs() {
            let Some(target) = target_map.get(input) else {
                return error!(format!("unsupported input for library: {input}"));
            };

            let target_srcs = target.get_inputs();
            if target_srcs.len() != 1 {
                return error!(format!("Too many inputs in target: {self:#?}"));
            }
            srcs.insert(target_srcs[0].replace(&add_slash_suffix(self.src_root), ""));

            for inc in target.get_includes(self.src_root, project) {
                includes.insert(inc.clone());
                self.include_directories.insert(inc);
            }

            for define in target.get_defines(project) {
                cflags.insert(String::from("-D") + &define);
            }
        }

        let (version_script, link_flags) = target.get_link_flags(self.src_root, project);

        let (static_libs, shared_libs) = match target.get_link_libraries(self.ndk_root, project) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };

        let generated_headers = match target.get_generated_headers(target_map) {
            Ok(return_value) => return_value,
            Err(err) => return Err(err),
        };
        self.generated_deps
            .extend(project.get_headers_to_copy(&generated_headers).into_iter());
        let generated_headers_filtered_raw = project.get_headers_to_generate(&generated_headers);
        let mut generated_headers_filtered = HashSet::new();
        for header in generated_headers_filtered_raw {
            generated_headers_filtered.insert(match target_map.get(&header) {
                Some(target_header) => target_header.get_name(self.target_prefix),
                None => return error!(format!("Could not find target for '{header}'")),
            });
        }
        let target_name = target.get_name(self.target_prefix);

        let mut module = crate::soong_module::SoongModule::new(name);
        module.add_str("stem", project.get_target_stem(&target_name));
        if project.optimize_target_for_size(&target_name) {
            module.add_bool("optimize_for_size", true);
        }
        module.add_set("header_libs", project.get_target_header_libs(&target_name));
        module.add_str("name", target_name);
        module.add_bool("use_clang_lld", true);
        module.add_set("srcs", srcs);
        module.add_set("local_include_dirs", includes);
        module.add_set("cflags", cflags);
        module.add_set("ldflags", link_flags);
        module.add_str("version_script", version_script);
        module.add_set("static_libs", static_libs);
        module.add_set("shared_libs", shared_libs);
        module.add_set("generated_headers", generated_headers_filtered);

        return Ok(module.print());
    }

    fn replace_output_in_command(
        &self,
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
            String::from("$(location ") + &project.rework_command_output(output) + ")";
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
            command = self.replace_output_in_command(command, output, project);
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
        target: &NinjaTarget,
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
                set.insert(project.rework_command_output(output));
                set
            });

        let command = self.rework_command(command, inputs, target_outputs, generated_deps, project);

        let mut module = crate::soong_module::SoongModule::new("cc_genrule");
        module.add_str("name", target.get_name(self.target_prefix));
        module.add_set("srcs", srcs_set);
        module.add_set("out", out_set);
        module.add_str("cmd", command.to_string());
        return Ok(module.print());
    }

    fn generate_target(
        &mut self,
        target: &NinjaTarget,
        target_map: &HashMap<String, &NinjaTarget>,
        project: &dyn Project,
    ) -> Result<(), String> {
        let rule = target.get_rule();
        let result = if rule.starts_with("CXX_SHARED_LIBRARY") {
            self.generate_library("cc_library_shared", target, target_map, project)
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            self.generate_library("cc_library_static", target, target_map, project)
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

    fn create_target_map(targets: &Vec<NinjaTarget>) -> HashMap<String, &NinjaTarget> {
        let mut map: HashMap<String, &NinjaTarget> = HashMap::new();
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
        targets: Vec<NinjaTarget>,
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

        let target_map = Self::create_target_map(&targets);

        while let Some(input) = target_to_generate.pop() {
            if target_seen.contains(&input) || project.ignore_target(&input) {
                continue;
            }
            let Some(target) = target_map.get(&input) else {
                continue;
            };

            target_to_generate.append(&mut target.get_all_inputs());
            for output in target.get_all_outputs() {
                target_seen.insert(output);
            }

            if let Err(err) = self.generate_target(target, &target_map, project) {
                return Err(err);
            }
        }
        return Ok(());
    }
}
