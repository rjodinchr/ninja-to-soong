use std::collections::HashSet;

use crate::soongmodule::SoongModule;
use crate::soongpackage::SoongPackage;
use crate::target::BuildTarget;
use crate::utils::*;

const LLVM_PREFIX: &str = "third_party/llvm";
const TARGET_PREFIX: &str = "clspv_";

pub struct CLSPV<'a> {
    src_root: &'a str,
    build_root: &'a str,
    ndk_root: &'a str,
    spirv_headers_root: &'a str,
    llvm_project_root: &'a str,
}

impl<'a> CLSPV<'a> {
    pub fn new(
        src_root: &'a str,
        build_root: &'a str,
        ndk_root: &'a str,
        spirv_headers_root: &'a str,
        llvm_project_root: &'a str,
    ) -> Self {
        CLSPV {
            src_root,
            build_root,
            ndk_root,
            spirv_headers_root,
            llvm_project_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for CLSPV<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            self.build_root,
            TARGET_PREFIX,
            "//external/clvk",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        if let Err(err) = package.generate(vec!["libclspv_core.a"], targets, &self) {
            return Err(err);
        }
        package.add_module(SoongModule::new_cc_library_headers(
            CLSPV_HEADERS,
            ["include".to_string()].into(),
        ));
        return package.write();
    }
    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        let mut srcs: HashSet<String> = HashSet::new();
        let mut filtered_inputs: HashSet<String> = HashSet::new();
        let mut generated_deps: HashSet<(String, String)> = HashSet::new();
        let clang_root = &(self.llvm_project_root.to_string() + "/clang");

        for input in inputs {
            if input.contains(self.spirv_headers_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(self.spirv_headers_root, input),
                ));
            } else if input.contains(clang_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &clang_headers_name(clang_root, input),
                ));
            } else if input.contains(LLVM_PREFIX) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &llvm_headers_name(LLVM_PREFIX, input),
                ));
            } else if !input.contains(self.src_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + TARGET_PREFIX + &rework_name(input, self.build_root, ""),
                ));
            } else {
                filtered_inputs.insert(input.clone());
            }
        }
        for input in &filtered_inputs {
            srcs.insert(input.replace(&add_slash_suffix(self.src_root), ""));
        }
        for (_, dep) in &generated_deps {
            srcs.insert(dep.clone());
        }
        return Ok((srcs, filtered_inputs, generated_deps));
    }
    fn get_default_cflags(&self) -> HashSet<String> {
        return ["-Wno-unreachable-code-loop-increment".to_string()].into();
    }
    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
    fn ignore_target(&self, target: &String) -> bool {
        target.starts_with("third_party/")
    }
    fn ignore_include(&self, include: &str) -> bool {
        include.contains(self.build_root)
            || include.contains(self.spirv_headers_root)
            || include.contains(self.llvm_project_root)
    }
    fn get_headers_to_generate(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            if !header.contains(LLVM_PREFIX) {
                set.insert(header.clone());
            }
        }
        return set;
    }
    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [
            SPIRV_HEADERS.to_string(),
            LLVM_HEADERS.to_string(),
            CLANG_HEADERS.to_string(),
        ]
        .into()
    }
    fn rework_command_output(&self, output: &str) -> String {
        if let Some(split) = output.split_once("include/") {
            split.1
        } else if !output.contains("libclc") {
            output.split("/").last().unwrap()
        } else {
            output
        }
        .to_string()
    }
    fn optimize_target_for_size(&self, _target: &String) -> bool {
        true
    }
}
