use std::collections::HashSet;

use crate::soongmodule::SoongModule;
use crate::soongpackage::SoongPackage;
use crate::target::BuildTarget;
use crate::utils::*;

pub struct SpirvTools<'a> {
    src_root: &'a str,
    build_root: &'a str,
    ndk_root: &'a str,
    spirv_headers_root: &'a str,
}

impl<'a> SpirvTools<'a> {
    pub fn new(
        src_root: &'a str,
        build_root: &'a str,
        ndk_root: &'a str,
        spirv_headers_root: &'a str,
    ) -> Self {
        SpirvTools {
            src_root,
            build_root,
            ndk_root,
            spirv_headers_root,
        }
    }
    fn generate_spirv_headers(&self, mut files: HashSet<String>) -> Result<String, String> {
        let mut package = SoongPackage::new("", "", "", "");

        if let Err(err) = package.add_module(SoongModule::new_cc_library_headers(
            SPIRV_HEADERS,
            ["include".to_string()].into(),
        )) {
            return Err(err);
        }

        files.insert(self.spirv_headers_root.to_string() + "/include/spirv/unified1/spirv.hpp"); // for clspv
        let mut sorted = Vec::from_iter(files);
        sorted.sort();
        for file in sorted {
            if let Err(err) = package.add_module(SoongModule::new_copy_genrule(
                spirv_headers_name(self.spirv_headers_root, &file),
                file.replace(&add_slash_suffix(self.spirv_headers_root), ""),
                file.rsplit_once("/").unwrap().1.to_string(),
            )) {
                return Err(err);
            }
        }

        return package.write(self.spirv_headers_root);
    }
}

impl<'a> crate::project::Project<'a> for SpirvTools<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let mut package = SoongPackage::new(
            self.src_root,
            self.ndk_root,
            self.build_root,
            "SPIRV-Tools_",
        );
        if let Err(err) = package.generate(
            vec![
                "libSPIRV-Tools.a",
                "libSPIRV-Tools-link.a",
                "libSPIRV-Tools-opt.a",
            ],
            targets,
            &self,
        ) {
            return Err(err);
        }
        match self.generate_spirv_headers(package.get_generated_headers()) {
            Ok(message) => println!("{message}"),
            Err(err) => return Err(err),
        }
        if let Err(err) = package.add_module(SoongModule::new_cc_library_headers(
            SPIRV_TOOLS_HEADERS,
            ["include".to_string()].into(),
        )) {
            return Err(err);
        }
        return package.write(self.src_root);
    }
    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        let mut srcs: HashSet<String> = HashSet::new();
        let mut filtered_inputs: HashSet<String> = HashSet::new();
        let mut generated_deps: HashSet<(String, String)> = HashSet::new();

        for input in inputs {
            if input.contains(self.spirv_headers_root) {
                generated_deps.insert((
                    input.clone(),
                    ":".to_string() + &spirv_headers_name(self.spirv_headers_root, input),
                ));
            } else {
                filtered_inputs.insert(input.clone());
            }
        }
        for input in &filtered_inputs {
            srcs.insert(input.replace(&add_slash_suffix(self.src_root), ""));
        }
        for (_, dep) in &generated_deps {
            srcs.insert(dep.clone());
        }
        return Ok((srcs, filtered_inputs, generated_deps));
    }
    fn get_default_defines(&self) -> HashSet<String> {
        ["-Wno-implicit-fallthrough".to_string()].into()
    }
    fn ignore_include(&self, include: &str) -> bool {
        include.contains(self.build_root) || include.contains(self.spirv_headers_root)
    }
    fn get_headers_to_generate(&self, headers: &HashSet<String>) -> HashSet<String> {
        let mut set = HashSet::new();
        for header in headers {
            set.insert(header.clone());
        }
        return set;
    }
    fn get_object_header_libs(&self) -> HashSet<String> {
        [SPIRV_HEADERS.to_string()].into()
    }
}
