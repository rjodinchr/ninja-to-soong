// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;

#[derive(Default)]
pub struct SpirvTools {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    spirv_headers_path: PathBuf,
    gen_deps: Vec<PathBuf>,
}

impl Project for SpirvTools {
    fn get_id(&self) -> ProjectId {
        ProjectId::SpirvTools
    }

    fn generate_package(
        &mut self,
        ctx: &Context,
        projects_map: &ProjectsMap,
    ) -> Result<SoongPackage, String> {
        self.src_path = self.get_id().android_path(&ctx.android_path);
        self.build_path = ctx.temp_path.join(self.get_id().str());
        self.ndk_path = get_ndk_path(&ctx.temp_path)?;
        self.spirv_headers_path = ProjectId::SpirvHeaders.android_path(&ctx.android_path);

        let (targets, _) = ninja_target::cmake::get_targets(
            &self.src_path,
            &self.build_path,
            &self.ndk_path,
            vec![
                &("-DSPIRV-Headers_SOURCE_DIR=".to_string()
                    + &path_to_string(&self.spirv_headers_path)),
            ],
            None,
            ctx,
        )?;

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
            GenDeps::TargetsToGen.get(self, ProjectId::Clvk, projects_map),
            targets,
            self,
        )?;
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvTools,
            ["include".to_string()].into(),
        ));

        self.gen_deps = Vec::from_iter(package.get_gen_deps());

        Ok(package)
    }

    fn get_default_cflags(&self, _target: &str) -> Vec<String> {
        vec!["-Wno-implicit-fallthrough".to_string()]
    }

    fn get_deps_info(&self) -> Vec<(PathBuf, GenDeps)> {
        vec![(self.spirv_headers_path.clone(), GenDeps::SpirvHeaders)]
    }

    fn get_gen_deps(&self, _project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        deps.insert(GenDeps::SpirvHeaders, self.gen_deps.clone());
        deps
    }

    fn get_library_module(&self, _module: &mut SoongModule) {
        _module.add_prop(
            "export_include_dirs",
            SoongProp::VecStr(vec!["include".to_string()]),
        );
        _module.add_prop(
            "export_header_lib_headers",
            SoongProp::VecStr(vec![CcLibraryHeaders::SpirvHeaders.str()]),
        )
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }

    fn get_target_header_libs(&self, _target: &str) -> Vec<String> {
        vec![CcLibraryHeaders::SpirvHeaders.str()]
    }

    fn ignore_cflag(&self, _cflag: &str) -> bool {
        true
    }

    fn ignore_include(&self, include: &Path) -> bool {
        include.starts_with(&self.build_path) || include.starts_with(&self.spirv_headers_path)
    }

    fn ignore_define(&self, _define: &str) -> bool {
        true
    }
}
