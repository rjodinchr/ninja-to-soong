// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::CcLibraryHeaders;

#[derive(Debug)]
pub struct SoongModule {
    name: String,
    str_map: HashMap<String, String>,
    vec_map: HashMap<String, Vec<String>>,
    bool_map: HashMap<String, bool>,
}

impl SoongModule {
    const INDENT: &'static str = "    ";

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            str_map: HashMap::new(),
            vec_map: HashMap::new(),
            bool_map: HashMap::new(),
        }
    }

    pub fn new_cc_library_headers(name: CcLibraryHeaders, include_dirs: Vec<String>) -> Self {
        let mut module = Self::new("cc_library_headers");
        module.add_str("name", name.str());
        module.add_vec("export_include_dirs", include_dirs);
        module
    }

    pub fn new_copy_genrule(name: String, src: String, out: String) -> Self {
        let mut module = Self::new("genrule");
        module.add_str("name", name);
        module.add_vec("srcs", vec![src]);
        module.add_vec("out", vec![out]);
        module.add_str("cmd", "cp $(in) $(out)".to_string());
        module
    }

    pub fn add_str(&mut self, key: &str, str: String) {
        self.str_map.insert(key.to_string(), str);
    }

    pub fn add_vec(&mut self, key: &str, vec: Vec<String>) {
        self.vec_map.insert(key.to_string(), vec);
    }

    pub fn add_bool(&mut self, key: &str, bool: bool) {
        self.bool_map.insert(key.to_string(), bool);
    }

    pub fn filter_vec<F>(&mut self, key: &str, f: F)
    where
        F: FnMut(&String) -> bool + Clone,
    {
        let Some(vec) = self.vec_map.remove(key) else {
            return;
        };
        self.add_vec(key, vec.into_iter().filter(f.clone()).collect())
    }

    fn print_key_value(key: &str, value: &str) -> String {
        Self::INDENT.to_string() + key + ": " + value + ",\n"
    }

    pub fn print(mut self) -> String {
        let mut module = String::new();
        module += "\n";
        module += &self.name;
        module += " {\n";

        for key in ["name", "stem", "version_script", "cmd"] {
            if let Some(value) = self.str_map.remove(key) {
                module += &Self::print_key_value(&key, &("\"".to_string() + &value + "\""))
            }
        }
        for key in [
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
        ] {
            let Some(mut vec) = self.vec_map.remove(key) else {
                continue;
            };
            if vec.len() == 0 {
                continue;
            }

            module += &Self::print_key_value(
                &key,
                &(if vec.len() == 1 {
                    "[\"".to_string() + &(vec[0]) + "\"]"
                } else {
                    vec.sort();
                    let mut values = String::from("[\n");
                    for value in vec {
                        values = values + Self::INDENT + Self::INDENT + "\"" + &value + "\",\n";
                    }
                    values = values + Self::INDENT + "]";
                    values
                }),
            );
        }
        for key in ["optimize_for_size", "use_clang_lld"] {
            if let Some(value) = self.bool_map.remove(key) {
                module += &Self::print_key_value(&key, if value { "true" } else { "false" });
            }
        }

        module += "}\n";
        module
    }
}
