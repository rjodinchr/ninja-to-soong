// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Clvk {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    gen_libs: Vec<PathBuf>,
}

impl Project for Clvk {
    fn get_name(&self) -> &'static str {
        "clvk"
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

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                vec![
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.ndk_path),
                    ANDROID_ABI,
                    ANDROID_ISA,
                    ANDROID_PLATFORM,
                    &path_to_string(ProjectId::SpirvHeaders.get_android_path(projects_map, ctx)?),
                    &path_to_string(ProjectId::SpirvTools.get_android_path(projects_map, ctx)?),
                    &path_to_string(ProjectId::LlvmProject.get_android_path(projects_map, ctx)?),
                    &path_to_string(ProjectId::Clspv.get_android_path(projects_map, ctx)?),
                ]
            )?;
        }

        let targets = parse_build_ninja::<CmakeNinjaTarget>(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            "//visibility:public",
            "clvk_license",
            vec!["SPDX-license-identifier-Apache-2.0"],
            vec!["LICENSE"],
        );
        package.generate(vec![PathBuf::from("libOpenCL.so")], targets, self)?;

        self.gen_libs = Vec::from_iter(package.get_gen_libs());
        Ok(package)
    }

    fn get_deps(&self, dep: Dep) -> Vec<PathBuf> {
        let prefix = match dep {
            Dep::ClspvTargets => "clspv",
            Dep::LlvmProjectTargets => "llvm-project",
            Dep::SpirvToolsTargets => "SPIRV-Tools",
            _ => return Vec::new(),
        };
        self.gen_libs
            .iter()
            .filter(|lib| self.map_lib(lib).starts_with(prefix))
            .map(|lib| strip_prefix(self.map_lib(lib), prefix))
            .collect()
    }

    fn get_target_name(&self, target: &Path) -> PathBuf {
        if target.ends_with("libOpenCL.so") {
            PathBuf::from("libclvk")
        } else {
            PathBuf::from(target)
        }
    }
    fn get_target_module(&self, _target: &Path, mut module: SoongModule) -> SoongModule {
        module.add_prop(
            "header_libs",
            SoongProp::VecStr(vec![String::from("OpenCL-Headers")]),
        );
        module
    }

    fn map_lib(&self, library: &Path) -> PathBuf {
        strip_prefix(
            if let Ok(strip) = library.strip_prefix(Path::new("external/clspv/third_party/llvm")) {
                Path::new("llvm-project").join(strip)
            } else {
                PathBuf::from(library)
            },
            "external",
        )
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_include(&self, _include: &Path) -> bool {
        false
    }
    fn filter_lib(&self, lib: &str) -> bool {
        lib != "libatomic"
    }
    fn filter_link_flag(&self, flag: &str) -> bool {
        flag == "-Wl,-Bsymbolic"
    }
    fn filter_target(&self, target: &Path) -> bool {
        !target.starts_with("external")
    }
}
