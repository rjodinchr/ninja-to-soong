// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;

#[derive(Default)]
pub struct Clspv {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    spirv_headers_path: PathBuf,
    llvm_project_path: PathBuf,
    gen_deps: Vec<PathBuf>,
}

impl Project for Clspv {
    fn get_id(&self) -> ProjectId {
        ProjectId::Clspv
    }

    fn generate_package(
        &mut self,
        android_path: &Path,
        temp_path: &Path,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_id().android_path(android_path);
        self.build_path = temp_path.join(self.get_id().str());
        self.ndk_path = get_ndk_path(temp_path)?;
        self.spirv_headers_path = ProjectId::SpirvHeaders.android_path(android_path);
        self.llvm_project_path = ProjectId::LlvmProject.android_path(android_path);

        let spirv_headers_path =
            "-DSPIRV_HEADERS_SOURCE_DIR=".to_string() + &path_to_string(&self.spirv_headers_path);
        let spirv_tools_path = "-DSPIRV_TOOLS_SOURCE_DIR=".to_string()
            + &path_to_string(&ProjectId::SpirvTools.android_path(android_path));
        let llvm_project_path = "-DCLSPV_LLVM_SOURCE_DIR=".to_string()
            + &path_to_string(self.llvm_project_path.join("llvm"));
        let clang_path = "-DCLSPV_CLANG_SOURCE_DIR=".to_string()
            + &path_to_string(self.llvm_project_path.join("clang"));
        let libclc_path = "-DCLSPV_LIBCLC_SOURCE_DIR=".to_string()
            + &path_to_string(self.llvm_project_path.join("libclc"));

        let (targets, _) = ninja_target::cmake::get_targets(
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
            None,
        )?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_id().str()),
            "//external/clvk",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(
            GenDeps::TargetsToGen.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::Clspv,
            ["include".to_string()].into(),
        ));

        self.gen_deps = Vec::from_iter(package.get_gen_deps());

        Ok(package)
    }

    fn get_cmd_output(&self, output: &Path) -> PathBuf {
        if let Some((_, header)) = split_path(output, "include") {
            header
        } else {
            output.to_path_buf()
        }
    }

    fn get_deps_info(&self) -> Vec<(PathBuf, GenDeps)> {
        vec![
            (self.spirv_headers_path.clone(), GenDeps::SpirvHeaders),
            (self.llvm_project_path.join("clang"), GenDeps::ClangHeaders),
            (PathBuf::from("third_party/llvm"), GenDeps::LibclcBins),
        ]
    }

    fn get_gen_deps(&self, project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        match project {
            ProjectId::SpirvHeaders => {
                let mut files = Vec::new();
                for dep in &self.gen_deps {
                    if dep.starts_with(&self.spirv_headers_path) {
                        files.push(dep.clone());
                    }
                }
                deps.insert(GenDeps::SpirvHeaders, files);
            }
            ProjectId::LlvmProject => {
                let mut clang_headers = Vec::new();
                let mut libclc_binaries = Vec::new();
                for dep in &self.gen_deps {
                    if let Ok(strip) = dep.strip_prefix(&self.llvm_project_path) {
                        clang_headers.push(strip.to_path_buf());
                    } else if file_name(dep) == "clspv--.bc" || file_name(dep) == "clspv64--.bc" {
                        libclc_binaries.push(strip_prefix(dep, "third_party/llvm"));
                    }
                }
                deps.insert(GenDeps::ClangHeaders, clang_headers);
                deps.insert(GenDeps::LibclcBins, libclc_binaries);
            }
            _ => (),
        };
        deps
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }

    fn get_target_header_libs(&self, _target: &str) -> Vec<String> {
        vec![
            CcLibraryHeaders::SpirvHeaders.str(),
            CcLibraryHeaders::Llvm.str(),
            CcLibraryHeaders::Clang.str(),
        ]
    }

    fn ignore_cflag(&self, _cflag: &str) -> bool {
        true
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
