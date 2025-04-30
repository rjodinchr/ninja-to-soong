// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::*;

pub enum CcLibraryHeaders {
    SpirvTools,
    SpirvHeaders,
    Llvm,
    Clang,
}
impl CcLibraryHeaders {
    pub fn str(self) -> String {
        String::from(match self {
            Self::SpirvTools => "SPIRV-Tools-includes",
            Self::SpirvHeaders => "SPIRV-Headers-includes",
            Self::Llvm => "llvm-includes",
            Self::Clang => "clang-includes",
        })
    }
}

#[derive(Debug, Clone)]
pub enum SoongProp {
    Str(String),
    VecStr(Vec<String>),
    Bool(bool),
    Prop(Box<SoongNamedProp>),
}

#[derive(Debug, Clone)]
pub struct SoongNamedProp {
    name: String,
    prop: SoongProp,
}

impl SoongNamedProp {
    pub fn new_prop(name: &str, prop: SoongProp) -> SoongProp {
        SoongProp::Prop(Box::new(Self::new(name, prop)))
    }

    fn new(name: &str, prop: SoongProp) -> Self {
        Self {
            name: String::from(name),
            prop,
        }
    }

    pub fn get_prop(&self) -> SoongProp {
        self.prop.clone()
    }

    fn print(self, indent_level: usize) -> String {
        const INDENT: &str = "    ";
        let indent = INDENT.repeat(indent_level);
        format!(
            "{indent}{0}: {1},\n",
            self.name,
            match self.prop {
                SoongProp::Str(str) => format!("\"{str}\""),
                SoongProp::Bool(bool) => format!("{bool}"),
                SoongProp::Prop(prop) => format!("{{\n{0}{indent}}}", prop.print(indent_level + 1)),
                SoongProp::VecStr(mut vec_str) => {
                    if vec_str.len() == 0 {
                        return String::new();
                    }
                    vec_str.sort_unstable();
                    vec_str.dedup();
                    if vec_str.len() == 1 {
                        format!("[\"{0}\"]", vec_str[0])
                    } else {
                        let indent_next = INDENT.repeat(indent_level + 1);
                        format!(
                            "[\n{0}{indent}]",
                            vec_str
                                .iter()
                                .map(|str| format!("{indent_next}\"{str}\",\n",))
                                .collect::<Vec<String>>()
                                .concat()
                        )
                    }
                }
            }
        )
    }
}

#[derive(Debug)]
pub struct SoongModule {
    name: String,
    props: Vec<SoongNamedProp>,
}

impl SoongModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            props: Vec::new(),
        }
    }

    pub fn new_cc_library_headers(name: CcLibraryHeaders, include_dirs: Vec<String>) -> Self {
        Self::new("cc_library_headers")
            .add_prop("name", SoongProp::Str(name.str()))
            .add_prop("export_include_dirs", SoongProp::VecStr(include_dirs))
    }

    pub fn new_filegroup(name: String, files: Vec<String>) -> Self {
        Self::new("filegroup")
            .add_prop("name", SoongProp::Str(name))
            .add_prop("srcs", SoongProp::VecStr(files))
    }

    pub fn add_prop(mut self, name: &str, prop: SoongProp) -> SoongModule {
        self.props.push(SoongNamedProp::new(name, prop));
        self
    }

    pub fn add_props(mut self, props: Vec<SoongNamedProp>) -> SoongModule {
        self.props.extend(props);
        self
    }

    pub fn update_prop<F>(&mut self, name: &str, f: F) -> Result<(), String>
    where
        F: Fn(SoongProp) -> Result<SoongProp, String>,
    {
        for index in 0..self.props.len() {
            if self.props[index].name == name {
                let prop = self.props.remove(index).prop;
                let updated_prop = f(prop)?;
                self.props
                    .insert(index, SoongNamedProp::new(name, updated_prop));
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn filter_default(&mut self, default: &SoongModule) -> Result<(), String> {
        let my_name = match self.get_prop("name").unwrap().prop {
            SoongProp::Str(name) => name,
            _ => return error!("Unexpected SoongProp 'name' in {default:#?}"),
        };
        for default_prop in &default.props {
            let name = &default_prop.name;
            match &default_prop.prop {
                SoongProp::VecStr(default_vec_str) => {
                    self.update_prop(name, |module_prop| match module_prop {
                        SoongProp::VecStr(module_vec_str) => {
                            if name != "defaults" {
                                for str in default_vec_str {
                                    if !module_vec_str.contains(str) {
                                        return error!(
                                            "Could not filter {name:#?} from {my_name:#?} because it does not contain {str:#?}"
                                        );
                                    }
                                }
                            }
                            Ok(SoongProp::VecStr(
                                module_vec_str
                                    .into_iter()
                                    .filter(|str| !default_vec_str.contains(str))
                                    .collect::<Vec<_>>(),
                            ))
                        }
                        prop => Ok(prop),
                    })?
                }
                _ => (),
            };
        }
        Ok(())
    }

    pub fn get_prop(&self, name: &str) -> Option<SoongNamedProp> {
        for prop in &self.props {
            if prop.name == name {
                return Some(prop.clone());
            }
        }
        None
    }

    pub fn print(self) -> String {
        format!(
            "\n{0} {{\n{1}}}\n",
            self.name,
            self.props
                .into_iter()
                .map(|prop| prop.print(1))
                .collect::<Vec<String>>()
                .concat()
        )
    }
}
