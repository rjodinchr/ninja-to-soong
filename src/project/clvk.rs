// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct Clvk {
    gen_libs: HashMap<Dep, Vec<String>>,
}

impl Project for Clvk {
    fn get_name(&self) -> &'static str {
        "clvk"
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
        let build_path = ctx.temp_path.join(self.get_name());
        let ndk_path = get_ndk_path(&ctx.temp_path, ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
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

        const LIBCLVK: &str = "libclvk";
        let mut package = SoongPackage::new(
            &[],
            "clvk_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE"],
        )
        .generate(
            NinjaTargetsToGenMap::from(&[
                target!("libOpenCL.so", LIBCLVK),
                target_typed!("simple_test", "cc_test"),
                target_typed!("api_tests", "cc_test"),
            ]),
            parse_build_ninja::<CmakeNinjaTarget>(&build_path)?,
            &src_path,
            &ndk_path,
            &build_path,
            None,
            self,
            ctx,
        )?
        .add_visibilities(vec![
            ProjectId::OpenclIcdLoader.get_visibility(projects_map)?
        ]);

        let gen_libs = package.get_gen_libs();
        for (dep, prefix) in [
            (Dep::ClspvTargets, "clspv"),
            (Dep::LlvmProjectTargets, "llvm-project"),
            (Dep::SpirvToolsTargets, "SPIRV-Tools"),
        ] {
            self.gen_libs.insert(
                dep,
                gen_libs
                    .iter()
                    .filter_map(|lib| {
                        if let Ok(strip) = self.map_lib(lib).unwrap().strip_prefix(prefix) {
                            return Some(path_to_string(strip));
                        }
                        None
                    })
                    .collect(),
            );
        }

        const CLVK_ICD_GENRULE: &str = "clvk_icd_genrule";
        package
            .add_raw_suffix(&format!(
                r#"
cc_genrule {{
    name: "{CLVK_ICD_GENRULE}",
    cmd: "echo /vendor/$$CC_MULTILIB/{LIBCLVK}.so > $(out)",
    out: ["clvk.icd"],
    soc_specific: true,
}}

prebuilt_etc {{
    name: "clvk_icd_prebuilt",
    src: ":{CLVK_ICD_GENRULE}",
    filename_from_src: true,
    relative_install_path: "Khronos/OpenCL/vendors",
    soc_specific: true,
}}
"#
            ))
            .print(ctx)
    }

    fn get_deps(&self, dep: Dep) -> Vec<NinjaTargetToGen> {
        match self.gen_libs.get(&dep) {
            Some(gen_libs) => gen_libs.iter().map(|lib| target!(lib)).collect(),
            None => Vec::new(),
        }
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> Result<SoongModule, String> {
        let mut header_libs = vec![String::from("OpenCL-Headers")];
        if target.ends_with("api_tests") {
            header_libs.push(CcLibraryHeaders::SpirvHeaders.str());
            header_libs.push(String::from("vulkan_headers"));
            module = module.add_prop(
                "test_config",
                SoongProp::Str(String::from("android/api_tests.xml")),
            );
        } else if target.ends_with("simple_test") {
            module = module.add_prop("gtest", SoongProp::Bool(false)).add_prop(
                "test_config",
                SoongProp::Str(String::from("android/simple_test.xml")),
            );
        }
        let cflags = if target.ends_with("api_tests") {
            vec!["-Wno-missing-braces"]
        } else {
            Vec::new()
        };
        module
            .add_prop("soc_specific", SoongProp::Bool(true))
            .add_prop("header_libs", SoongProp::VecStr(header_libs))
            .extend_prop("cflags", cflags)
    }

    fn map_lib(&self, library: &Path) -> Option<PathBuf> {
        Some(strip_prefix(
            if let Ok(strip) = library.strip_prefix(Path::new("external/clspv/third_party/llvm")) {
                Path::new("llvm-project").join(strip)
            } else {
                PathBuf::from(library)
            },
            "external",
        ))
    }

    fn filter_cflag(&self, _cflag: &str) -> bool {
        false
    }
    fn filter_gen_header(&self, _header: &Path) -> bool {
        false
    }
    fn filter_include(&self, include: &Path) -> bool {
        include.ends_with("src")
    }
    fn filter_lib(&self, lib: &str) -> bool {
        lib != "libatomic" && !lib.contains("gtest")
    }
    fn filter_link_flag(&self, flag: &str) -> bool {
        flag == "-Wl,-Bsymbolic"
    }
    fn filter_target(&self, target: &Path) -> bool {
        !target.starts_with("external")
    }
}
