use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;
use crate::target::BuildTarget;

#[derive(Debug)]
struct SoongPackage {
    name: String,
    str_map: HashMap<String, String>,
    set_map: HashMap<String, HashSet<String>>,
    optimize_for_size: bool,
}

impl SoongPackage {
    fn new(name: &str, optimize_for_size: bool) -> Self {
        Self {
            name: name.to_string(),
            str_map: HashMap::new(),
            set_map: HashMap::new(),
            optimize_for_size,
        }
    }

    fn add_str(&mut self, key: &str, value: String) {
        self.str_map.insert(key.to_string(), value);
    }

    fn add_set(&mut self, key: &str, set: HashSet<String>) {
        self.set_map.insert(key.to_string(), set);
    }

    fn print_str(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, value)) = self.str_map.remove_entry(entry) else {
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

    fn print_set(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, mut set)) = self.set_map.remove_entry(entry) else {
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
        if let Some(set) = self.set_map.get_mut("cflags") {
            set.insert("-Wno-error".to_string());
            set.insert("-Wno-unreachable-code-loop-increment".to_string());
        }
        let mut result = String::new();
        result += &self.name;
        result += " {\n";

        if !self.str_map.contains_key("name") {
            return error!(format!("no 'name' in soong package: '{self:#?}"));
        }
        result += &self.print_str("name");
        result += &self.print_set("srcs");
        result += &self.print_set("cflags");
        result += &self.print_set("ldflags");
        result += &self.print_str("version_script");
        result += &self.print_set("shared_libs");
        result += &self.print_set("static_libs");
        result += &self.print_set("local_include_dirs");

        if self.str_map.len() > 0 || self.set_map.len() > 0 {
            return error!(format!("entries not consumed in: '{self:#?}"));
        }
        if self.optimize_for_size {
            result += "    optimize_for_size: true,\n";
        }
        result += "    use_clang_lld: true,\n";

        result += "}\n\n";
        return Ok(result);
    }
}

#[derive(Debug)]
struct SoongFile<'a> {
    content: String,
    sources: HashSet<String>,
    generated_headers: HashSet<String>,
    include_directories: HashSet<String>,
    targets_map: &'a HashMap<String, &'a BuildTarget>,
    src_root: &'a str,
    ndk_root: &'a str,
    build_root: &'a str,
}

impl<'a> SoongFile<'a> {
    fn new(
        targets_map: &'a HashMap<String, &'a BuildTarget>,
        src_root: &'a str,
        ndk_root: &'a str,
        build_root: &'a str,
    ) -> Self {
        SoongFile {
            content: String::new(),
            sources: HashSet::new(),
            generated_headers: HashSet::new(),
            include_directories: HashSet::new(),
            targets_map,
            src_root,
            ndk_root,
            build_root,
        }
    }
    fn generate_object(
        &mut self,
        name: &str,
        target: &BuildTarget,
        optimize_for_size: bool,
    ) -> Result<(), String> {
        let mut package = SoongPackage::new(name, optimize_for_size);
        package.add_str("name", target.get_name());

        let mut includes: HashSet<String> = HashSet::new();
        let mut defines: HashSet<String> = HashSet::new();
        let mut srcs: HashSet<String> = HashSet::new();
        for input in target.get_inputs() {
            let Some(target) = self.targets_map.get(input) else {
                return error!(format!("unsupported input for library: {input}"));
            };
            let (src, src_includes, src_defines) =
                match target.get_compiler_target_info(self.src_root, self.build_root) {
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
            self.sources.insert(src.clone());
            srcs.insert(src);
        }
        package.add_set("srcs", srcs);
        package.add_set("local_include_dirs", includes);
        package.add_set("cflags", defines);

        let (version_script, link_flags) = target.get_link_flags(self.src_root);
        package.add_set("ldflags", link_flags);
        if let Some(vs) = version_script {
            self.sources.insert(vs.clone());
            package.add_str("version_script", vs);
        }

        let (static_libs, shared_libs) = match target.get_link_libraries(self.ndk_root) {
            Ok(return_values) => return_values,
            Err(err) => return Err(err),
        };
        package.add_set("static_libs", static_libs);
        package.add_set("shared_libs", shared_libs);

        let generated_headers = match target.get_generated_headers(self.targets_map) {
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

    fn finish(self) -> (String, HashSet<String>, HashSet<String>, HashSet<String>) {
        (
            self.content,
            self.sources,
            self.generated_headers,
            self.include_directories,
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
    src_root: &str,
    ndk_root: &str,
    build_root: &str,
) -> Result<(String, HashSet<String>, HashSet<String>, HashSet<String>), String> {
    let mut target_seen: HashSet<String> = HashSet::new();
    let mut target_to_generate = entry_targets;
    let targets_map = create_map(targets);
    let mut soong_file = SoongFile::new(&targets_map, src_root, ndk_root, build_root);

    while let Some(input) = target_to_generate.pop() {
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
            soong_file.generate_object("cc_library_shared", target, false)
        } else if rule.starts_with("CXX_STATIC_LIBRARY") {
            soong_file.generate_object("cc_library_static", target, true)
        } else {
            continue;
        };
        match result {
            Ok(__) => target_to_generate.append(&mut target.get_all_inputs()),
            Err(err) => return Err(err),
        };
    }
    return Ok(soong_file.finish());
}
