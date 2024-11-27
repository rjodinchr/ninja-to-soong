use std::collections::HashMap;
use std::collections::HashSet;

use crate::macros::error;

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

    pub fn print(mut self) -> Result<String, String> {
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
        let strs = vec!["name", "stem", "version_script", "cmd"];
        for entry in strs {
            result += &self.print_str(entry);
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
            "generated_headers",
        ];
        for entry in sets {
            result += &self.print_set(entry);
        }
        let bools = vec!["optimize_for_size", "host_supported", "use_clang_lld"];
        for entry in bools {
            result += &self.print_bool(entry);
        }

        if self.str_map.len() > 0 || self.set_map.len() > 0 {
            return error!(format!("entries not consumed in: '{self:#?}"));
        }

        result += "}\n\n";
        return Ok(result);
    }
}
