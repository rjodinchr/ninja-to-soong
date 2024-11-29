use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct SoongModule {
    name: String,
    str_map: HashMap<String, String>,
    set_map: HashMap<String, HashSet<String>>,
    bool_map: HashMap<String, bool>,
}

impl SoongModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            str_map: HashMap::new(),
            set_map: HashMap::new(),
            bool_map: HashMap::new(),
        }
    }

    pub fn new_cc_library_headers(name: &str, include_dirs: HashSet<String>) -> Self {
        let mut module = Self::new("cc_library_headers");
        module.add_str("name", name.to_string());
        module.add_set("export_include_dirs", include_dirs);
        module.add_set("visibility", ["//visibility:public".to_string()].into());
        module
    }

    pub fn new_copy_genrule(name: String, src: String, out: String) -> Self {
        let mut module = Self::new("genrule");
        module.add_str("name", name);
        module.add_set("visibility", ["//visibility:public".to_string()].into());
        module.add_set("srcs", [src].into());
        module.add_set("out", [out].into());
        module.add_str("cmd", "cp $(in) $(out)".to_string());
        module
    }

    pub fn add_str(&mut self, key: &str, value: String) {
        self.str_map.insert(key.to_string(), value);
    }

    pub fn add_set(&mut self, key: &str, value: HashSet<String>) {
        self.set_map.insert(key.to_string(), value);
    }

    pub fn add_bool(&mut self, key: &str, value: bool) {
        self.bool_map.insert(key.to_string(), value);
    }

    fn print_bool(&mut self, entry: &str) -> String {
        let mut result = String::new();
        let Some((key, value)) = self.bool_map.remove_entry(entry) else {
            return result;
        };
        result += "    ";
        result += &key;
        result += ": ";
        result += if value { "true" } else { "false" };
        result += ",\n";

        return result;
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
        let mut sorted = Vec::from_iter(set);
        sorted.sort();
        for value in sorted {
            result += "        \"";
            result += &value;
            result += "\",\n";
        }
        result += "    ],\n";
        return result;
    }

    pub fn print(mut self) -> String {
        let mut module = String::new();
        module += &self.name;
        module += " {\n";

        let strs = vec!["name", "stem", "version_script", "cmd"];
        for str in strs {
            module += &self.print_str(str);
        }
        let sets = vec![
            "srcs",
            "out",
            "tools",
            "cflags",
            "ldflags",
            "shared_libs",
            "static_libs",
            "local_include_dirs",
            "export_include_dirs",
            "header_libs",
            "generated_headers",
            "visibility",
        ];
        for set in sets {
            module += &self.print_set(set);
        }
        let bools = vec!["optimize_for_size", "use_clang_lld"];
        for bool in bools {
            module += &self.print_bool(bool);
        }

        module += "}\n\n";
        return module;
    }
}
