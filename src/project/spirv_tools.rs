// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;
use crate::soong_module::SoongModule;

#[derive(Default)]
pub struct SpirvTools {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    spirv_headers_path: PathBuf,
    gen_deps: HashSet<PathBuf>,
}

impl Project for SpirvTools {
    fn init(&mut self, android_path: &Path, ndk_path: &Path, temp_path: &Path) {
        self.src_path = self.get_id().android_path(android_path);
        self.build_path = temp_path.join(self.get_id().str());
        self.ndk_path = ndk_path.to_path_buf();
        self.spirv_headers_path = ProjectId::SpirvHeaders.android_path(android_path);
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
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
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
    ) -> Result<Option<PathBuf>, String> {
        let (ninja_file_path, _) = cmake_configure(
            &self.src_path,
            &self.build_path,
            &self.ndk_path,
            vec![
                &("-DSPIRV-Headers_SOURCE_DIR=".to_string()
                    + &path_to_string(&self.spirv_headers_path)),
            ],
        )?;
        Ok(Some(ninja_file_path))
    }

    fn get_cmd_inputs_and_deps(
        &self,
        target_inputs: &Vec<PathBuf>,
    ) -> Result<CmdInputAndDeps, String> {
        let mut inputs = HashSet::new();
        let mut deps = HashSet::new();

        for input in target_inputs {
            if input.starts_with(&self.spirv_headers_path) {
                deps.insert(cmd_dep(
                    input,
                    &self.spirv_headers_path,
                    CC_LIBRARY_HEADERS_SPIRV_HEADERS,
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

    fn ignore_include(&self, include: &Path) -> bool {
        include.starts_with(&self.build_path) || include.starts_with(&self.spirv_headers_path)
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
}
