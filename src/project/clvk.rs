// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Clvk {
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
    ) -> Result<String, String> {
        let src_path = self.get_android_path(ctx);
        let build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(self.get_test_path(ctx).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&build_path),
                    &path_to_string(&ndk_path),
                    &path_to_string(ProjectId::SpirvHeaders.get_android_path(projects_map, ctx)?),
                    &path_to_string(ProjectId::SpirvTools.get_android_path(projects_map, ctx)?),
                    &path_to_string(ProjectId::LlvmProject.get_android_path(projects_map, ctx)?),
                    &path_to_string(ProjectId::Clspv.get_android_path(projects_map, ctx)?),
                ]
            )?;
        }

        let mut package = SoongPackage::new(
            "//visibility:public",
            "clvk_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE"],
        )
        .generate(
            vec![PathBuf::from("libOpenCL.so")],
            parse_build_ninja::<CmakeNinjaTarget>(&build_path)?,
            &src_path,
            &ndk_path,
            &build_path,
            self,
        )?;
        self.gen_libs = package.get_gen_libs();

        Ok(package.print())
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
            .filter_map(|lib| {
                if let Ok(strip) = self.map_lib(lib).strip_prefix(prefix) {
                    return Some(PathBuf::from(strip));
                }
                None
            })
            .collect()
    }

    fn get_target_name(&self, target: &Path) -> PathBuf {
        if target.ends_with("libOpenCL.so") {
            PathBuf::from("libclvk")
        } else {
            PathBuf::from(target)
        }
    }
    fn get_target_module(&self, _target: &Path, module: SoongModule) -> SoongModule {
        module.add_prop(
            "header_libs",
            SoongProp::VecStr(vec![String::from("OpenCL-Headers")]),
        )
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
