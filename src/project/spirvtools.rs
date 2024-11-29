use std::collections::HashSet;

use crate::soongmodule::SoongModule;
use crate::soongpackage::SoongPackage;
use crate::target::BuildTarget;
use crate::utils::*;

pub struct SpirvTools<'a> {
    src_root: &'a str,
    build_root: &'a str,
    ndk_root: &'a str,
    spirv_headers_root: &'a str,
}

impl<'a> SpirvTools<'a> {
    pub fn new(
        src_root: &'a str,
        build_root: &'a str,
        ndk_root: &'a str,
        spirv_headers_root: &'a str,
    ) -> Self {
        SpirvTools {
            src_root,
            build_root,
            ndk_root,
            spirv_headers_root,
        }
    }
    fn generate_package(&self, targets: Vec<BuildTarget>) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            self.build_root,
            "SPIRV-Tools_",
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        if let Err(err) = package.generate(
            vec![
                "libSPIRV-Tools.a",
                "libSPIRV-Tools-link.a",
                "libSPIRV-Tools-opt.a",
            ],
            targets,
            self,
        ) {
            return Err(err);
        }
        package.add_module(SoongModule::new_cc_library_headers(
            SPIRV_TOOLS,
            ["include".to_string()].into(),
        ));

        return Ok(package);
    }
    pub fn get_generated_deps(self, targets: Vec<BuildTarget>) -> Result<HashSet<String>, String> {
        let package = match self.generate_package(targets) {
            Ok(package) => package,
            Err(err) => return Err(err),
        };
        return Ok(package.get_generated_deps());
    }
}

impl<'a> crate::project::Project<'a> for SpirvTools<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let package = match self.generate_package(targets) {
            Ok(package) => package,
            Err(err) => return Err(err),
        };
        return package.write();
    }
    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        let mut srcs: HashSet<String> = HashSet::new();
        let mut filtered_inputs: HashSet<String> = HashSet::new();
        let mut generated_deps: HashSet<(String, String)> = HashSet::new();

        for input in inputs {
            if input.contains(self.spirv_headers_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(self.spirv_headers_root, input),
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
        ["-Wno-implicit-fallthrough".to_string()].into()
    }
    fn ignore_include(&self, include: &str) -> bool {
        include.contains(self.build_root) || include.contains(self.spirv_headers_root)
    }
    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
    fn get_headers_to_generate(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        return set;
    }
    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [SPIRV_HEADERS.to_string()].into()
    }
}
