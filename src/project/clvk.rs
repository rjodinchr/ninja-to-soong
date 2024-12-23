// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;

#[derive(Default)]
pub struct Clvk {
    src_path: PathBuf,
    build_path: PathBuf,
    ndk_path: PathBuf,
    generated_libraries: Vec<PathBuf>,
}

impl Project for Clvk {
    fn get_id(&self) -> ProjectId {
        ProjectId::Clvk
    }
    fn get_name(&self) -> &'static str {
        "clvk"
    }
    fn get_android_path(&self, ctx: &Context) -> PathBuf {
        ctx.android_path.join("external").join(self.get_name())
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
                    &path_to_string(ctx.test_path.join(self.get_name()).join("gen-ninja.sh")),
                    &path_to_string(&self.src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.ndk_path),
                    ANDROID_ABI,
                    ANDROID_ISA,
                    ANDROID_PLATFORM,
                    &path_to_string(projects_map.get_android_path(ProjectId::SpirvHeaders, ctx)?),
                    &path_to_string(projects_map.get_android_path(ProjectId::SpirvTools, ctx)?),
                    &path_to_string(projects_map.get_android_path(ProjectId::LlvmProject, ctx)?),
                    &path_to_string(projects_map.get_android_path(ProjectId::Clspv, ctx)?),
                ]
            )?;
        }

        let targets = parse_build_ninja::<ninja_target::cmake::CmakeNinjaTarget>(&self.build_path)?;

        let mut package = SoongPackage::new(
            &self.src_path,
            &self.ndk_path,
            &self.build_path,
            Path::new(self.get_name()),
            "//visibility:public",
            "SPDX-license-identifier-Apache-2.0",
            "LICENSE",
        );
        package.generate(vec![PathBuf::from("libOpenCL.so")], targets, self)?;

        self.generated_libraries = Vec::from_iter(package.get_generated_libraries());
        Ok(package)
    }

    fn get_deps_map(&self, project: ProjectId) -> GenDepsMap {
        let mut deps = HashMap::new();
        let mut libs = Vec::new();
        let prefix = match project {
            ProjectId::Clspv => "clspv",
            ProjectId::LlvmProject => "llvm-project",
            ProjectId::SpirvTools => "SPIRV-Tools",
            _ => "",
        };
        for library in &self.generated_libraries {
            if let Ok(lib) = self.get_lib(library).strip_prefix(prefix) {
                libs.push(PathBuf::from(lib));
            }
        }
        deps.insert(GenDeps::TargetsToGen, libs);
        deps
    }

    fn get_target_alias(&self, target: &str) -> Option<String> {
        if target.ends_with("libOpenCL_so") {
            Some(String::from("libclvk"))
        } else {
            None
        }
    }
    fn get_target_header_libs(&self, _target: &str) -> Vec<String> {
        vec![String::from("OpenCL-Headers")]
    }

    fn get_lib(&self, library: &Path) -> PathBuf {
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
