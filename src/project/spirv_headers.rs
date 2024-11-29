use crate::ninja_target::NinjaTarget;
use crate::project::spirv_tools::SpirvTools;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;
use crate::utils::*;

pub struct SpirvHeaders<'a> {
    src_root: &'a str,
    ndk_root: &'a str,
    spirv_tools: &'a SpirvTools<'a>,
}

const SPIRV_HEADERS_PROJECT_NAME: &str = "spirv-headers";

impl<'a> SpirvHeaders<'a> {
    pub fn new(
        ndk_root: &'a str,
        spirv_headers_root: &'a str,
        spirv_tools: &'a SpirvTools,
    ) -> Self {
        SpirvHeaders {
            src_root: spirv_headers_root,
            ndk_root,
            spirv_tools,
        }
    }
}

impl<'a> crate::project::Project<'a> for SpirvHeaders<'a> {
    fn get_name(&self) -> String {
        SPIRV_HEADERS_PROJECT_NAME.to_string()
    }
    fn generate(&self, targets: Vec<NinjaTarget>) -> Result<String, String> {
        let mut files = match self.spirv_tools.get_generated_deps(targets) {
            Ok(files) => files,
            Err(err) => return Err(err),
        };
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            "",
            "SPIRV-Headers_",
            "//visibility:public",
            "SPDX-license-identifier-MIT",
            "LICENSE",
        );

        package.add_module(SoongModule::new_cc_library_headers(
            SPIRV_HEADERS,
            ["include".to_string()].into(),
        ));

        files.insert(self.src_root.to_string() + "/include/spirv/unified1/spirv.hpp"); // for clspv
        let mut sorted = Vec::from_iter(files);
        sorted.sort();
        for file in sorted {
            package.add_module(SoongModule::new_copy_genrule(
                spirv_headers_name(self.src_root, &file),
                file.replace(&add_slash_suffix(self.src_root), ""),
                file.rsplit_once("/").unwrap().1.to_string(),
            ));
        }

        return package.write();
    }
    fn get_build_directory(&self) -> Result<String, String> {
        return self.spirv_tools.get_build_directory();
    }
}
