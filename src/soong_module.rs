// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::error;

pub enum CcLibraryHeaders {
    SpirvTools,
    SpirvHeaders,
    Llvm,
    Clang,
    Clspv,
}
impl CcLibraryHeaders {
    pub fn str(self) -> String {
        String::from(match self {
            Self::SpirvTools => "SPIRV-Tools-includes",
            Self::SpirvHeaders => "SPIRV-Headers-includes",
            Self::Llvm => "llvm-includes",
            Self::Clang => "clang-includes",
            Self::Clspv => "clspv-includes",
        })
    }
}

fn print_indent(level: u8) -> String {
    let mut indent = String::new();
    for _ in 0..level {
        indent += "    ";
    }
    indent
}

#[derive(Debug)]
pub enum SoongProp {
    Str(String),
    VecStr(Vec<String>),
    Bool(bool),
    Prop(Box<SoongNamedProp>),
}

#[derive(Debug)]
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
            name: name.to_string(),
            prop,
        }
    }

    fn print(self, indent_level: u8) -> String {
        let mut output = String::new();
        output += &format!("{0}{1}: ", print_indent(indent_level), self.name);
        output += &(match self.prop {
            SoongProp::Str(str) => format!("\"{str}\""),
            SoongProp::VecStr(mut vec_str) => {
                vec_str.sort();
                let mut output = String::new();
                if vec_str.len() == 0 {
                    return output;
                }
                if vec_str.len() == 1 {
                    output = format!("[\"{0}\"]", vec_str[0]);
                } else {
                    output += "[\n";
                    for str in vec_str {
                        output +=
                            format!("{0}\"{str}\",\n", print_indent(indent_level + 1)).as_str();
                    }
                    output += &format!("{0}]", print_indent(indent_level));
                }
                output
            }
            SoongProp::Bool(bool) => {
                if bool {
                    String::from("true")
                } else {
                    String::from("false")
                }
            }
            SoongProp::Prop(prop) => format!(
                "{{\n{0}{1}}}",
                prop.print(indent_level + 1),
                print_indent(indent_level)
            ),
        });
        output += ",\n";
        output
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
            name: name.to_string(),
            props: Vec::new(),
        }
    }

    pub fn new_cc_library_headers(name: CcLibraryHeaders, include_dirs: Vec<String>) -> Self {
        let mut module = Self::new("cc_library_headers");
        module.add_prop("name", SoongProp::Str(name.str()));
        module.add_prop("export_include_dirs", SoongProp::VecStr(include_dirs));
        module
    }

    pub fn new_copy_genrule(name: String, src: String, out: String) -> Self {
        let mut module = Self::new("genrule");
        module.add_prop("name", SoongProp::Str(name));
        module.add_prop("cmd", SoongProp::Str("cp $(in) $(out)".to_string()));
        module.add_prop("srcs", SoongProp::VecStr(vec![src]));
        module.add_prop("out", SoongProp::VecStr(vec![out]));
        module
    }

    pub fn add_prop(&mut self, name: &str, prop: SoongProp) {
        self.props.push(SoongNamedProp::new(name, prop));
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
        error!("'{name}' property not found in {self:#?}")
    }

    pub fn print(self) -> String {
        let mut output = format!("\n{0} {{\n", self.name);
        for prop in self.props {
            output += &prop.print(1);
        }
        output += "}\n";
        output
    }
}
