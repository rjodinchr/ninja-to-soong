// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

pub struct SpirvTools<'a> {
    src_root: &'a str,
    build_root: String,
    ndk_root: &'a str,
    spirv_headers_root: &'a str,
    generated_deps: HashSet<String>,
}

const SPIRV_TOOLS_ID: ProjectId = ProjectId::SpirvTools;
const SPIRV_TOOLS_NAME: &str = SPIRV_TOOLS_ID.str();

impl<'a> SpirvTools<'a> {
    pub fn new(
        temp_dir: &'a str,
        ndk_root: &'a str,
        spirv_tools_root: &'a str,
        spirv_headers_root: &'a str,
    ) -> Self {
        SpirvTools {
            src_root: spirv_tools_root,
            build_root: temp_dir.to_string() + "/" + SPIRV_TOOLS_NAME,
            ndk_root,
            spirv_headers_root,
            generated_deps: HashSet::new(),
        }
    }
    fn generate_package(&mut self, targets: Vec<NinjaTarget>) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            &self.build_root,
            SPIRV_TOOLS_NAME,
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
            CC_LIB_HEADERS_SPIRV_TOOLS,
            ["include".to_string()].into(),
        ));

        self.generated_deps = package.get_generated_deps();

        return Ok(package);
    }
}

impl<'a> crate::project::Project<'a> for SpirvTools<'a> {
    fn get_id(&self) -> ProjectId {
        SPIRV_TOOLS_ID
    }
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        _dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<SoongPackage, String> {
        Ok(self.generate_package(targets)?)
    }
    fn get_build_directory(
        &mut self,
        _dep_packages: &HashMap<ProjectId, &dyn Project>,
    ) -> Result<String, String> {
        cmake_configure(
            self.src_root,
            &self.build_root,
            self.ndk_root,
            vec![&("-DSPIRV-Headers_SOURCE_DIR=".to_string() + self.spirv_headers_root)],
        )?;
        return Ok(self.get_generated_build_directory());
    }
    fn get_generated_build_directory(&self) -> String {
        self.build_root.clone()
    }
    fn get_generated_deps(&self) -> HashSet<String> {
        self.generated_deps.clone()
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
        include.contains(&self.build_root) || include.contains(self.spirv_headers_root)
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
        [CC_LIB_HEADERS_SPIRV_HEADERS.to_string()].into()
    }
}
