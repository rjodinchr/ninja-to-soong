// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;
use crate::soong_module::SoongModule;

#[derive(Default)]
pub struct Clspv {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    spirv_headers_path: PathBuf,
    spirv_tools_path: PathBuf,
    llvm_project_path: PathBuf,
    gen_deps: HashSet<PathBuf>,
}

impl Project for Clspv {
    fn init(&mut self, android_path: &Path, ndk_path: &Path, temp_path: &Path) {
        self.src_path = self.get_id().android_path(android_path);
        self.build_path = temp_path.join(self.get_id().str());
        self.ndk_path = ndk_path.to_path_buf();
        self.spirv_headers_path = ProjectId::SpirvHeaders.android_path(android_path);
        self.spirv_tools_path = ProjectId::SpirvTools.android_path(android_path);
        self.llvm_project_path = ProjectId::LlvmProject.android_path(android_path);
    }

    fn get_id(&self) -> ProjectId {
        ProjectId::Clspv
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
            self.get_id().str(),
            "//external/clvk",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(
            GenDeps::TargetsToGenerate.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;
        package.add_module(SoongModule::new_cc_library_headers(
            CC_LIBRARY_HEADERS_CLSPV,
            ["include".to_string()].into(),
        ));

        self.gen_deps = package.get_gen_deps();

        Ok(package)
    }

    fn get_ninja_file_path(
        &mut self,
        _projects_map: &ProjectsMap,
    ) -> Result<Option<PathBuf>, String> {
        let spirv_headers_path =
            "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + &str(&self.spirv_headers_path);
        let spirv_tools_path =
            "-DSPIRV_TOOLS_SOURCE_DIR=".to_string() + &str(&self.spirv_tools_path);
        let llvm_project_path =
            "-DCLSPV_LLVM_SOURCE_DIR=".to_string() + &str(&self.llvm_project_path.join("llvm"));
        let clang_path =
            "-DCLSPV_CLANG_SOURCE_DIR=".to_string() + &str(&self.llvm_project_path.join("clang"));
        let libclc_path =
            "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string() + &str(&self.llvm_project_path.join("libclc"));
        let (ninja_file_path, _) = cmake_configure(
            &self.src_path,
            &self.build_path,
            &self.ndk_path,
            vec![
                &spirv_headers_path,
                &spirv_tools_path,
                &llvm_project_path,
                &clang_path,
                &libclc_path,
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
        let clang_path = self.llvm_project_path.join("clang");

        for input in target_inputs {
            if input.starts_with(&self.spirv_headers_path) {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(&self.spirv_headers_path, &input),
                ));
            } else if input.starts_with(&clang_path) {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &clang_headers_name(&clang_path, &input),
                ));
            } else if input.starts_with("third_party/llvm") {
                deps.insert((
                    input.clone(),
                    ":".to_string() + &llvm_headers_name(Path::new("third_party/llvm"), &input),
                ));
            } else if !input.starts_with(&self.src_path) {
                deps.insert((
                    input.clone(),
                    ":".to_string()
                        + &path_to_id(
                            Path::new(self.get_id().str())
                                .join(strip_prefix(&input, &self.build_path)),
                        ),
                ));
            } else {
                inputs.insert(input.clone());
            }
        }
        Ok((inputs, deps))
    }

    fn get_cmd_output(&self, output: &Path) -> PathBuf {
        if let Some((_, header)) = split_include(output) {
            header
        } else {
            output.to_path_buf()
        }
    }

    fn get_gen_deps(&self, project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        match project {
            ProjectId::SpirvHeaders => {
                let mut files = HashSet::new();
                for dep in &self.gen_deps {
                    if dep.starts_with(&self.spirv_headers_path) {
                        files.insert(dep.clone());
                    }
                }
                deps.insert(GenDeps::SpirvHeadersFiles, files);
            }
            ProjectId::LlvmProject => {
                let mut clang_headers = HashSet::new();
                let mut libclc_binaries = HashSet::new();
                for dep in &self.gen_deps {
                    if let Ok(strip) = dep.strip_prefix(&self.llvm_project_path) {
                        clang_headers.insert(strip.to_path_buf());
                    } else if file_name(dep) == "clspv--.bc" || file_name(dep) == "clspv64--.bc" {
                        libclc_binaries.insert(strip_prefix(&dep, Path::new("third_party/llvm")));
                    }
                }
                deps.insert(GenDeps::ClangHeaders, clang_headers);
                deps.insert(GenDeps::LibclcBinaries, libclc_binaries);
            }
            _ => (),
        };
        deps
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }

    fn get_target_header_libs(&self, _target: &str) -> HashSet<String> {
        [
            CC_LIBRARY_HEADERS_SPIRV_HEADERS.to_string(),
            CC_LIBRARY_HEADERS_LLVM.to_string(),
            CC_LIBRARY_HEADERS_CLANG.to_string(),
        ]
        .into()
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }

    fn ignore_gen_header(&self, header: &Path) -> bool {
        header.starts_with("third_party/llvm")
    }

    fn ignore_include(&self, include: &Path) -> bool {
        include.starts_with(&self.build_path)
            || include.starts_with(&self.spirv_headers_path)
            || include.starts_with(&self.llvm_project_path)
    }

    fn ignore_target(&self, target: &Path) -> bool {
        target.starts_with("third_party/")
    }

    fn optimize_target_for_size(&self, _target: &str) -> bool {
        true
    }
}
