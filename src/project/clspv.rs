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
    fn get_name(&self) -> &'static str {
        "clspv"
    }
    fn get_android_path(&self, ctx: &Context) -> PathBuf {
        ctx.android_path.join("external").join(self.get_name())
    }
    fn get_test_path(&self, ctx: &Context) -> PathBuf {
        ctx.test_path.join(self.get_name())
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_android_path(ctx);
        self.build_path = ctx.temp_path.join(self.get_name());
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;
        self.spirv_headers_path = projects_map.get_android_path(ProjectId::SpirvHeaders, ctx)?;
        self.llvm_project_path = projects_map.get_android_path(ProjectId::LlvmProject, ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.ndk_path),
                    ANDROID_ABI,
                    ANDROID_PLATFORM,
                    &path_to_string(&self.spirv_headers_path),
                    &path_to_string(projects_map.get_android_path(ProjectId::SpirvTools, ctx)?),
                    &path_to_string(&self.llvm_project_path),
                ]
            )?;
        }

        let targets = parse_build_ninja::<ninja_target::cmake::CmakeNinjaTarget>(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            "//external/clvk",
            "clspv_license",
            vec!["SPDX-license-identifier-Apache-2.0"],
            vec!["LICENSE"],
        );
        package.generate(
            projects_map.get_deps(ProjectId::Clvk, self.get_id(), GenDeps::TargetsToGen)?,
            targets,
            self,
        )?;

        self.gen_deps = package.get_gen_deps();

        Ok(package)
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }
    fn get_deps_info(&self) -> Vec<(PathBuf, GenDeps)> {
        vec![
            (self.spirv_headers_path.clone(), GenDeps::SpirvHeaders),
            (self.llvm_project_path.join("clang"), GenDeps::ClangHeaders),
            (PathBuf::from("third_party/llvm"), GenDeps::LibclcBins),
        ]
    }
    fn get_deps_map(&self, project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        match project {
            ProjectId::SpirvHeaders => {
                let files = self
                    .gen_deps
                    .iter()
                    .filter(|dep| dep.starts_with(&self.spirv_headers_path))
                    .map(|dep| dep.clone())
                    .collect();
                deps.insert(GenDeps::SpirvHeaders, files);
            }
            ProjectId::LlvmProject => {
                let mut clang_headers = Vec::new();
                let mut libclc_binaries = Vec::new();
                for dep in &self.gen_deps {
                    if let Ok(strip) = dep.strip_prefix(&self.llvm_project_path) {
                        clang_headers.push(PathBuf::from(strip));
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

    fn get_target_object_module(&self, _target: &str, mut module: SoongModule) -> SoongModule {
        module.add_prop(
            "header_libs",
            SoongProp::VecStr(vec![
                CcLibraryHeaders::SpirvHeaders.str(),
                CcLibraryHeaders::Llvm.str(),
                CcLibraryHeaders::Clang.str(),
            ]),
        );
        module.add_prop(
            "export_include_dirs",
            SoongProp::VecStr(vec![String::from("include")]),
        );
        module.add_prop("optimize_for_size", SoongProp::Bool(true));
        module
    }

    fn get_cmd_output(&self, output: &Path) -> PathBuf {
        let mut prefix = output;
        while let Some(parent) = prefix.parent() {
            prefix = parent;
            if file_name(prefix) == "include" {
                return strip_prefix(output, prefix);
            }
        }
        PathBuf::from(output)
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_define(&self, _define: &str) -> bool {
        false
    }
    fn filter_gen_header(&self, header: &Path) -> bool {
        !header.starts_with("third_party/llvm")
    }
    fn filter_include(&self, include: &Path) -> bool {
        !(include.starts_with(&self.build_path)
            || include.starts_with(&self.spirv_headers_path)
            || include.starts_with(&self.llvm_project_path))
    }
    fn filter_target(&self, target: &Path) -> bool {
        !target.starts_with("third_party/")
    }
}
