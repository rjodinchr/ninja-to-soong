// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct SpirvTools {
    build_path: PathBuf,
    spirv_headers_path: PathBuf,
    gen_deps: Vec<String>,
}

impl Project for SpirvTools {
    fn get_name(&self) -> &'static str {
        "SPIRV-Tools"
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
        let ndk_path = PathBuf::from("SPIRV-Tools-ndk");
        self.spirv_headers_path = ProjectId::SpirvHeaders.get_android_path(projects_map, ctx)?;

        if !ctx.skip_gen_ninja {
            execute_cmd!(
                "bash",
                [
                    &path_to_string(ctx.get_script_path(self).join("gen-ninja.sh")),
                    &path_to_string(&src_path),
                    &path_to_string(&self.build_path),
                    &path_to_string(&self.spirv_headers_path),
                ]
            )?;
        }

        let generate_vksp_deps = !ctx.copy_to_aosp;
        const GENERATED_TABLES: &str = "SPIRV-Tools_core_tables";
        let mut targets_to_gen =
            NinjaTargetsToGenMap::from(&Dep::SpirvToolsTargets.get_ninja_targets(projects_map)?);
        if generate_vksp_deps {
            targets_to_gen = targets_to_gen
                .push(target!("core_tables_body.inc", GENERATED_TABLES))
                .push(target!("core_tables_header.inc", GENERATED_TABLES))
        }
        let mut package = SoongPackage::new(
            &[],
            "SPIRV-Tools_license",
            &["SPDX-license-identifier-Apache-2.0"],
            &["LICENSE"],
        )
        .generate(
            targets_to_gen,
            parse_build_ninja::<CmakeNinjaTarget>(&self.build_path)?,
            &src_path,
            &ndk_path,
            &self.build_path,
            None,
            self,
            ctx,
        )?
        .add_visibilities(Dep::SpirvToolsTargets.get_visibilities(projects_map)?)
        .add_module(SoongModule::new_cc_library_headers(
            CcLibraryHeaders::SpirvTools,
            vec![String::from("include")],
        ));
        self.gen_deps = package
            .get_gen_deps()
            .into_iter()
            .map(|header| path_to_string(strip_prefix(header, &self.spirv_headers_path)))
            .collect();

        if generate_vksp_deps {
            package = package
                .add_visibilities(vec![String::from("//external/vulkan-shader-profiler")])
                .add_raw_suffix(&format!(
                    r#"
cc_library_headers {{
    name: "SPIRV-Tools-sources",
    header_libs: ["{0}"],
    generated_headers: ["{GENERATED_TABLES}"],
    export_include_dirs: ["."],
    export_header_lib_headers: ["{0}"],
    export_generated_headers: ["{GENERATED_TABLES}"],
    vendor_available: true,
}}
"#,
                    CcLibraryHeaders::SpirvHeadersUnified1.str()
                ))
        }
        package.print(ctx)
    }

    fn get_deps_prefix(&self) -> Vec<(PathBuf, Dep)> {
        vec![(self.spirv_headers_path.clone(), Dep::SpirvHeaders)]
    }
    fn get_deps(&self, dep: Dep) -> Vec<NinjaTargetToGen> {
        match dep {
            Dep::SpirvHeaders => self
                .gen_deps
                .iter()
                .map(|header| target!(header.as_str()))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn extend_module(&self, target: &Path, mut module: SoongModule) -> Result<SoongModule, String> {
        if !target.ends_with("spirv-as") {
            module = module
                .extend_prop("export_include_dirs", vec!["include"])?
                .add_prop(
                    "export_header_lib_headers",
                    SoongProp::VecStr(vec![CcLibraryHeaders::SpirvHeaders.str()]),
                )
                .add_prop("vendor_available", SoongProp::Bool(true));
            if target.ends_with("libSPIRV-Tools.a") {
                module = module.add_prop("host_supported", SoongProp::Bool(true));
            }
        }
        module
            .add_prop(
                "header_libs",
                SoongProp::VecStr(vec![CcLibraryHeaders::SpirvHeaders.str()]),
            )
            .extend_prop("cflags", vec!["-Wno-implicit-fallthrough"])
    }
    fn extend_custom_command(
        &self,
        _target: &Path,
        module: SoongModule,
    ) -> Result<SoongModule, String> {
        Ok(module
            .add_prop("vendor_available", SoongProp::Bool(true))
            .add_prop("host_supported", SoongProp::Bool(true)))
    }
    fn extend_python_binary_host(
        &self,
        _python_binary_path: &Path,
        module: SoongModule,
    ) -> Result<Option<SoongModule>, String> {
        Ok(Some(module.extend_prop("srcs", vec!["utils/Table/*.py"])?))
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
    fn filter_link_flag(&self, _flag: &str) -> bool {
        false
    }
    fn filter_lib(&self, lib: &str) -> bool {
        lib != "librt"
    }
}
