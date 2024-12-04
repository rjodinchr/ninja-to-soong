// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;
use crate::soong_module::SoongModule;

#[derive(Default)]
pub struct SpirvTools {
    src_dir: String,
    build_dir: String,
    ndk_dir: String,
    spirv_headers_dir: String,
    gen_deps: HashSet<String>,
}

impl Project for SpirvTools {
    fn init(&mut self, android_dir: &str, ndk_dir: &str, temp_dir: &str) {
        self.src_dir = self.get_id().android_path(android_dir);
        self.build_dir = add_slash_suffix(temp_dir) + self.get_id().str();
        self.ndk_dir = ndk_dir.to_string();
        self.spirv_headers_dir = ProjectId::SpirvHeaders.android_path(android_dir);
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::SpirvTools
    }

    fn generate_package(
        &mut self,
        targets: Vec<NinjaTarget>,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        let mut package = SoongPackage::new(
            &self.src_dir,
            &self.ndk_dir,
            &self.build_dir,
            self.get_id().str(),
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

    fn get_ninja_file_path(
        &mut self,
        _projects_map: &ProjectsMap,
    ) -> Result<Option<String>, String> {
        let (ninja_file_path, _) = cmake_configure(
            &self.src_dir,
            &self.build_dir,
            &self.ndk_dir,
            vec![&("-DSPIRV-Headers_SOURCE_DIR=".to_string() + &self.spirv_headers_dir)],
        )?;
        Ok(Some(ninja_file_path))
    }

    fn get_cmd_inputs_and_deps(
        &self,
        target_inputs: &Vec<String>,
    ) -> Result<CmdInputAndDeps, String> {
        let mut inputs: HashSet<String> = HashSet::new();
        let mut deps: HashSet<(String, String)> = HashSet::new();

        for input in target_inputs {
            if input.contains(&self.spirv_headers_dir) {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(&self.spirv_headers_dir, input),
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

    fn get_gen_deps(&self, _project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        deps.insert(GenDeps::SpirvHeadersFiles, self.gen_deps.clone());
        deps
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }

    fn get_target_header_libs(&self, _target: &str) -> HashSet<String> {
        [CC_LIBRARY_HEADERS_SPIRV_HEADERS.to_string()].into()
    }

    fn ignore_include(&self, include: &str) -> bool {
        include.contains(&self.build_dir) || include.contains(&self.spirv_headers_dir)
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
}
