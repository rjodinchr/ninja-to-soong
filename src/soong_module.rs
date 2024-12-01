// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

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
        module
    }

    pub fn new_copy_genrule(name: String, src: String, out: String) -> Self {
        let mut module = Self::new("genrule");
        module.add_str("name", name);
        module.add_set("srcs", [src].into());
        module.add_set("out", [out].into());
        module.add_str("cmd", "cp $(in) $(out)".to_string());
        module
    }

    pub fn add_str(&mut self, key: &str, str: String) {
        self.str_map.insert(key.to_string(), str);
    }

    pub fn add_set(&mut self, key: &str, set: HashSet<String>) {
        self.set_map.insert(key.to_string(), set);
    }

    pub fn add_bool(&mut self, key: &str, bool: bool) {
        self.bool_map.insert(key.to_string(), bool);
    }

    fn print_bool(&mut self, key: &str) -> String {
        let mut bool = String::new();
        let Some((key, value)) = self.bool_map.remove_entry(key) else {
            return bool;
        };
        bool += "    ";
        bool += &key;
        bool += ": ";
        bool += if value { "true" } else { "false" };
        bool += ",\n";

        bool
    }

    fn print_str(&mut self, key: &str) -> String {
        let mut str = String::new();
        let Some((key, value)) = self.str_map.remove_entry(key) else {
            return str;
        };
        if value == "" {
            return str;
        }
        str += "    ";
        str += &key;
        str += ": \"";
        str += &value;
        str += "\",\n";

        str
    }

    fn print_set(&mut self, key: &str) -> String {
        let mut set = String::new();
        let Some((key, mut hash_set)) = self.set_map.remove_entry(key) else {
            return set;
        };
        hash_set.remove("");
        if hash_set.len() == 0 {
            return set;
        }
        set += "    ";
        set += &key;
        set += ": ";

        if hash_set.len() == 1 {
            set += "[ \"";
            for value in hash_set {
                set += &value;
            }
            set += "\" ],\n";
        } else {
            set += "[\n";
            let mut sorted = Vec::from_iter(hash_set);
            sorted.sort();
            for value in sorted {
                set += "        \"";
                set += &value;
                set += "\",\n";
            }
            set += "    ],\n";
        }
        set
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
            "default_visibility",
            "default_applicable_licenses",
            "license_kinds",
            "license_text",
        ];
        for set in sets {
            module += &self.print_set(set);
        }
        let bools = vec!["optimize_for_size", "use_clang_lld"];
        for bool in bools {
            module += &self.print_bool(bool);
        }

        module += "}\n\n";
        module
    }
}
