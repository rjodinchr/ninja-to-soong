use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;
use crate::target::BuildTarget;

#[derive(Debug)]
struct SoongPackage {
    name: String,
    single_string_map: HashMap<String, String>,
    list_string_map: HashMap<String, HashSet<String>>,
    optimize_for_size: bool,
}

impl SoongPackage {
    fn new(name: &str, optimize_for_size: bool) -> Self {
        Self {
            name: name.to_string(),
            single_string_map: HashMap::new(),
            list_string_map: HashMap::new(),
            optimize_for_size,
        }
    }

    fn add_single_string(&mut self, key: &str, value: String) {
        self.single_string_map.insert(key.to_string(), value);
    }

    fn add_list_string(&mut self, key: &str, set: HashSet<String>) {
        self.list_string_map.insert(key.to_string(), set);
    }

    fn print_single_string(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, value)) = self.single_string_map.remove_entry(entry) else {
            return result;
        };
        if value == "" {
            return result;
        }
        result += "    ";
        result += &key;
        result += ": \"";
        result += &value;
        result += "\",\n";

        return result;
    }

    fn print_list_string(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, mut set)) = self.list_string_map.remove_entry(entry) else {
            return result;
        };
        set.remove("");
        if set.len() == 0 {
            return result;
        }
        result += "    ";
        result += &key;
        result += ": ";

        result += "[\n";
        for value in set {
            result += "        \"";
            result += &value;
            result += "\",\n";
        }
        result += "    ],\n";
        return result;
    }

    fn print(mut self) -> Result<String, String> {
        if let Some(set) = self.list_string_map.get_mut("cflags") {
            set.insert("-Wno-error".to_string());
            set.insert("-Wno-unreachable-code-loop-increment".to_string());
        }
        let mut result = String::new();
        result += &self.name;
        result += " {\n";

        if !self.single_string_map.contains_key("name") {
            return error!(format!("no 'name' in soong package: '{self:#?}"));
        }
        result += &self.print_single_string("name");
        result += &self.print_list_string("srcs");
        result += &self.print_list_string("cflags");
        result += &self.print_list_string("ldflags");
        result += &self.print_single_string("version_script");
        result += &self.print_list_string("shared_libs");
        result += &self.print_list_string("static_libs");
        result += &self.print_list_string("local_include_dirs");
        result += &self.print_list_string("include_dirs");

        if self.single_string_map.len() > 0 || self.list_string_map.len() > 0 {
            return error!(format!("entries not consumed in: '{self:#?}"));
        }
        if self.optimize_for_size {
            result += "    optimize_for_size: true,\n";
        }

        result += "}\n\n";
        return Ok(result);
    }
}

#[derive(Debug)]
pub struct SoongFile {
    content: String,
    generated_headers: HashSet<String>,
    generated_directories: HashSet<String>,
}

impl SoongFile {
    fn new() -> Self {
        SoongFile {
            content: String::new(),
            generated_headers: HashSet::new(),
            generated_directories: HashSet::new(),
        }
    }
    fn generate_object(
        &mut self,
        name: &str,
        target: &BuildTarget,
        targets_map: &HashMap<String, &BuildTarget>,
        source_root: &str,
        native_lib_root: &str,
        build_root: &str,
        cmake_build_files_root: &str,
        optimize_for_size: bool,
    ) -> Result<(), String> {
        let mut package = SoongPackage::new(name, optimize_for_size);
        package.add_single_string("name", crate::target::rework_target_name(target.get_name()));

        let mut includes: HashSet<String> = HashSet::new();
        let mut defines: HashSet<String> = HashSet::new();
        let mut srcs: HashSet<String> = HashSet::new();
        for input in target.get_inputs() {
            let Some(target) = targets_map.get(input) else {
                return error!(format!("unsupported input for library: {input}"));
            };
            let (src, src_includes, src_defines) = match target.get_compiler_target_info(
                source_root,
                build_root,
                cmake_build_files_root,
            ) {
                Ok(return_values) => return_values,
                Err(err) => return Err(err),
            };
            for inc in src_includes {
                includes.insert(inc.clone());
                if inc.contains(cmake_build_files_root) {
                    self.generated_directories.insert(inc);
                }
            }
            for def in src_defines {
                defines.insert(String::from("-D") + &def);
            }
            srcs.insert(src);
        }
        package.add_list_string("srcs", srcs);
        package.add_list_string("local_include_dirs", includes);
        package.add_list_string("cflags", defines);

        let (version_script, link_flags) = target.get_link_flags(source_root);
        package.add_list_string("ldflags", link_flags);
        package.add_single_string("version_script", version_script);

        let (static_libs, shared_libs) = match target.get_link_libraries(native_lib_root) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };
        package.add_list_string("static_libs", static_libs);
        package.add_list_string("shared_libs", shared_libs);

        let generated_headers = match target.get_generated_headers(&targets_map) {
            Ok(return_value) => return_value,
            Err(err) => return Err(err),
        };
        for header in generated_headers {
            self.generated_headers.insert(header);
        }

        match package.print() {
            Ok(return_value) => self.content += &return_value,
            Err(err) => return Err(err),
        };
        Ok(())
    }

    pub fn finish(self) -> (String, HashSet<String>, HashSet<String>) {
        (
            self.content,
            self.generated_headers,
            self.generated_directories,
        )
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
    entry_targets: Vec<String>,
    targets: &Vec<BuildTarget>,
    source_root: &str,
    native_lib_root: &str,
    build_root: &str,
    cmake_build_files_root: &str,
) -> Result<SoongFile, String> {
    let mut target_seen: HashSet<String> = HashSet::new();
    let mut target_to_generate = entry_targets;
    let targets_map = create_map(targets);
    let mut soong_file = SoongFile::new();

    while let Some(input) = target_to_generate.pop() {
        //println!("target: {input}");
        if target_seen.contains(&input) {
            continue;
        }
        let Some(target) = targets_map.get(&input) else {
            continue;
        };

        for output in target.get_all_outputs() {
            target_seen.insert(output);
        }

        let rule = target.get_rule();
        let result = if rule.starts_with("CXX_SHARED_LIBRARY") {
            soong_file.generate_object(
                "cc_library_shared",
                target,
                &targets_map,
                source_root,
                native_lib_root,
                build_root,
                cmake_build_files_root,
                false,
            )
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            soong_file.generate_object(
                "cc_library_static",
                target,
                &targets_map,
                source_root,
                native_lib_root,
                build_root,
                cmake_build_files_root,
                true,
            )
        } else {
            continue;
        };
        match result {
            Ok(__) => target_to_generate.append(&mut target.get_all_inputs()),
            Err(err) => return Err(err),
        };
    }
    return Ok(soong_file);
}
