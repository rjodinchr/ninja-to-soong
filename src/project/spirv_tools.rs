// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::ninja_target::NinjaTarget;
use crate::project::*;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;

pub struct SpirvTools<'a> {
    src_dir: &'a str,
    build_dir: String,
    ndk_dir: &'a str,
    spirv_headers_dir: &'a str,
    gen_deps: HashSet<String>,
}

impl<'a> SpirvTools<'a> {
    pub fn new(
        temp_dir: &'a str,
        ndk_dir: &'a str,
        spirv_tools_dir: &'a str,
        spirv_headers_dir: &'a str,
    ) -> Self {
        SpirvTools {
            src_dir: spirv_tools_dir,
            build_dir: add_slash_suffix(temp_dir) + ProjectId::SpirvTools.str(),
            ndk_dir,
            spirv_headers_dir,
            gen_deps: HashSet::new(),
        }
    }
}

impl<'a> crate::project::Project<'a> for SpirvTools<'a> {
    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            self.src_dir,
            self.ndk_dir,
            &self.build_dir,
            ProjectId::SpirvTools.str(),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(
            GenDeps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_SPIRV_TOOLS,
            ["include".to_string()].into(),
        ));

        self.gen_deps = package.get_gen_deps();

        Ok(package)
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::SpirvTools
    }

    fn get_build_dir(&mut self, _projects_map: &ProjectsMap) -> Result<Option<String>, String> {
        cmake_configure(
            self.src_dir,
            &self.build_dir,
            self.ndk_dir,
            vec![&("-DSPIRV-Headers_SOURCE_DIR=".to_string() + self.spirv_headers_dir)],
        )?;
        Ok(Some(self.get_generated_build_dir()))
    }

    fn get_cmd_inputs_and_deps(
        &self,
        target_inputs: &Vec<String>,
    ) -> Result<CmdInputAndDeps, String> {
        let mut inputs: HashSet<String> = HashSet::new();
        let mut deps: HashSet<(String, String)> = HashSet::new();

        for input in target_inputs {
            if input.contains(self.spirv_headers_dir) {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(self.spirv_headers_dir, input),
                ));
            } else {
                inputs.insert(input.clone());
            }
        }
        Ok((inputs, deps))
    }

    fn get_default_cflags(&self) -> HashSet<String> {
        ["-Wno-implicit-fallthrough".to_string()].into()
    }

    fn get_generated_build_dir(&self) -> String {
        self.build_dir.clone()
    }

    fn get_gen_deps(&self, _project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        deps.insert(GenDeps::SpirvHeadersFiles, self.gen_deps.clone());
        deps
    }

    fn get_headers_to_generate(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        set
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }

    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        [CC_LIBRARY_HEADERS_SPIRV_HEADERS.to_string()].into()
    }

    fn ignore_include(&self, include: &str) -> bool {
        include.contains(&self.build_dir) || include.contains(self.spirv_headers_dir)
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
}
