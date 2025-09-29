// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Clspv {
    build_path: PathBuf,
    spirv_headers_path: PathBuf,
    llvm_project_path: PathBuf,
    gen_deps: HashMap<Dep, Vec<String>>,
}

impl Project for Clspv {
    fn get_name(&self) -> &'static str {
        "clspv"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        Ok(Path::new("external").join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let src_path = ctx.get_android_path(self)?;
        self.build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;
        self.spirv_headers_path = ProjectId::SpirvHeaders.get_android_path(projects_map, ctx)?;
        self.llvm_project_path = ProjectId::LlvmProject.get_android_path(projects_map, ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&ndk_path),
                    &path_to_string(&self.spirv_headers_path),
                    &path_to_string(ProjectId::SpirvTools.get_android_path(projects_map, ctx)?),
                    &path_to_string(&self.llvm_project_path),
                ]
            )?;
        }

        let mut package = SoongPackage::new(
            &[],
            "clspv_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&Dep::ClspvTargets.get_ninja_targets(projects_map)?),
            parse_build_ninja::<CmakeNinjaTarget>(&self.build_path)?,
            &src_path,
            &ndk_path,
            &self.build_path,
            None,
            self,
            ctx,
        )?
        .add_visibilities(Dep::ClspvTargets.get_visibilities(projects_map)?);

        let gen_deps = package.get_gen_deps();
        self.gen_deps.insert(
            Dep::ClangHeaders,
            gen_deps
                .iter()
                .filter_map(|dep| {
                    if let Ok(strip) = dep.strip_prefix(&self.llvm_project_path) {
                        return Some(path_to_string(strip));
                    }
                    None
                })
                .collect(),
        );
        self.gen_deps.insert(
            Dep::LibclcBins,
            gen_deps
                .iter()
                .filter_map(|dep| {
                    if file_name(dep) == "clspv--.bc" || file_name(dep) == "clspv64--.bc" {
                        return Some(path_to_string(strip_prefix(dep, "third_party/llvm")));
                    }
                    None
                })
                .collect(),
        );
        self.gen_deps.insert(
            Dep::SpirvHeaders,
            gen_deps
                .iter()
                .filter_map(|dep| {
                    if let Ok(strip) = dep.strip_prefix(&self.spirv_headers_path) {
                        return Some(path_to_string(strip));
                    }
                    None
                })
                .collect(),
        );

        package.print(ctx)
    }

    fn get_deps_prefix(&self) -> Vec<(PathBuf, Dep)> {
        vec![
            (self.spirv_headers_path.clone(), Dep::SpirvHeaders),
            (self.llvm_project_path.join("clang"), Dep::ClangHeaders),
            (PathBuf::from("third_party/llvm"), Dep::LibclcBins),
        ]
    }
    fn get_deps(&self, dep: Dep) -> Vec<NinjaTargetToGen> {
        match self.gen_deps.get(&dep) {
            Some(gen_deps) => gen_deps.iter().map(|lib| target!(lib)).collect(),
            None => Vec::new(),
        }
    }

    fn extend_module(&self, _target: &Path, module: SoongModule) -> Result<SoongModule, String> {
        module
            .add_prop("optimize_for_size", SoongProp::Bool(true))
            .add_prop("vendor_available", SoongProp::Bool(true))
            .add_prop(
                "header_libs",
                SoongProp::VecStr(vec![
                    CcLibraryHeaders::SpirvHeaders.str(),
                    CcLibraryHeaders::Llvm.str(),
                    CcLibraryHeaders::Clang.str(),
                ]),
            )
            .extend_prop("export_include_dirs", vec!["include"])
    }
    fn extend_custom_command(
        &self,
        _target: &Path,
        module: SoongModule,
    ) -> Result<SoongModule, String> {
        Ok(module.add_prop("vendor_available", SoongProp::Bool(true)))
    }

    fn map_cmd_output(&self, output: &Path) -> PathBuf {
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
