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
    fn get_name(&self) -> &'static str {
        "SPIRV-Tools"
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

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(ctx.test_path.join(self.get_name()).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.ndk_path),
                    ANDROID_ABI,
                    ANDROID_PLATFORM,
                    &path_to_string(&self.spirv_headers_path),
                ]
            )?;
        }

        let targets = parse_build_ninja::<ninja_target::cmake::CmakeNinjaTarget>(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            "//visibility:public",
            "SPIRV-Tools_license",
            vec!["SPDX-license-identifier-Apache-2.0"],
            vec!["LICENSE"],
        );
        package.generate(
            projects_map.get_deps(ProjectId::Clvk, self.get_id(), GenDeps::TargetsToGen)?,
            targets,
            self,
        )?;
        package.add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvTools,
            vec![String::from("include")],
        ));

        self.gen_deps = package.get_gen_deps();

        Ok(package)
    }

    fn get_project_deps(&self) -> Vec<ProjectId> {
        vec![ProjectId::Clvk]
    }
    fn get_deps_info(&self) -> Vec<(PathBuf, GenDeps)> {
        vec![(self.spirv_headers_path.clone(), GenDeps::SpirvHeaders)]
    }
    fn get_deps_map(&self, _project: ProjectId) -> GenDepsMap {
        let mut deps: GenDepsMap = HashMap::new();
        deps.insert(GenDeps::SpirvHeaders, self.gen_deps.clone());
        deps
    }

    fn get_target_object_module(&self, _target: &str, mut module: SoongModule) -> SoongModule {
        module.add_prop(
            "export_include_dirs",
            SoongProp::VecStr(vec![String::from("include")]),
        );
        module.add_prop(
            "export_header_lib_headers",
            SoongProp::VecStr(vec![CcLibraryHeaders::SpirvHeaders.str()]),
        );
        module
    }
    fn get_target_cflags(&self, _target: &str) -> Vec<String> {
        vec![String::from("-Wno-implicit-fallthrough")]
    }
    fn get_target_header_libs(&self, _target: &str) -> Vec<String> {
        vec![CcLibraryHeaders::SpirvHeaders.str()]
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_include(&self, include: &Path) -> bool {
        !(include.starts_with(&self.build_path) || include.starts_with(&self.spirv_headers_path))
    }
    fn filter_define(&self, _define: &str) -> bool {
        false
    }
}
