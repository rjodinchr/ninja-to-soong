use crate::ninja_target::NinjaTarget;
use crate::project::spirv_tools::SpirvTools;
use crate::soong_module::SoongModule;
use crate::soong_package::SoongPackage;
use crate::utils::*;

pub struct SpirvHeaders<'a> {
    src_root: &'a str,
    build_root: &'a str,
    ndk_root: &'a str,
    spirv_tools_root: &'a str,
}

impl<'a> SpirvHeaders<'a> {
    pub fn new(
        src_root: &'a str,
        build_root: &'a str,
        ndk_root: &'a str,
        spirv_tools_root: &'a str,
    ) -> Self {
        SpirvHeaders {
            src_root,
            build_root,
            ndk_root,
            spirv_tools_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for SpirvHeaders<'a> {
    fn generate(self, targets: Vec<NinjaTarget>) -> Result<String, String> {
        let spirvtools = SpirvTools::new(
            self.spirv_tools_root,
            self.build_root,
            self.ndk_root,
            self.src_root,
        );
        let mut files = match spirvtools.get_generated_deps(targets) {
            Ok(files) => files,
            Err(err) => return Err(err),
        };
        let mut package = SoongPackage::new(
            &self.src_root,
            &self.ndk_root,
            &self.build_root,
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
}
