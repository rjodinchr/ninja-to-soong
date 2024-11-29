use std::collections::HashSet;

use crate::soong_package::SoongPackage;
use crate::ninja_target::NinjaTarget;
use crate::utils::*;

pub struct CLVK<'a> {
    src_root: &'a str,
    build_root: &'a str,
    ndk_root: &'a str,
}

impl<'a> CLVK<'a> {
    pub fn new(src_root: &'a str, build_root: &'a str, ndk_root: &'a str) -> Self {
        CLVK {
            src_root,
            build_root,
            ndk_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for CLVK<'a> {
    fn generate(self, targets: Vec<NinjaTarget>) -> Result<String, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            self.build_root,
            "clvk_",
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        if let Err(err) = package.generate(vec!["libOpenCL.so"], targets, &self) {
            return Err(err);
        }
        return package.write();
    }
    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        let mut srcs: HashSet<String> = HashSet::new();
        let mut filtered_inputs: HashSet<String> = HashSet::new();

        for input in inputs {
            filtered_inputs.insert(input.clone());
        }
        for input in &filtered_inputs {
            srcs.insert(input.replace(&add_slash_suffix(self.src_root), ""));
        }
        return Ok((srcs, filtered_inputs, HashSet::new()));
    }
    fn ignore_target(&self, target: &String) -> bool {
        target.starts_with("external/")
    }
    fn ignore_include(&self, _include: &str) -> bool {
        true
    }
    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [
            SPIRV_TOOLS.to_string(),
            SPIRV_HEADERS.to_string(),
            CLSPV_HEADERS.to_string(),
            "OpenCL-Headers".to_string(),
        ]
        .into()
    }
    fn get_library_name(&self, library: &str) -> String {
        library
            .replace("external/clspv/third_party/llvm", "llvm-project")
            .replace("external/", "")
            .replace("/", "_")
            .replace(".", "_")
    }
    fn handle_link_flag(&self, flag: &str, link_flags: &mut HashSet<String>) {
        if flag == "-Wl,-Bsymbolic" {
            link_flags.insert(flag.to_string());
        }
    }
    fn get_target_stem(&self, target: &String) -> String {
        if target == "clvk_libOpenCL_so" {
            "libclvk".to_string()
        } else {
            String::new()
        }
    }
}
