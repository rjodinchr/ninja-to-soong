use std::collections::HashSet;

use crate::soongfile::SoongFile;
use crate::soongmodule::SoongModule;
use crate::target::BuildTarget;
use crate::utils::add_slash_suffix;

const SPIRV_HEADERS: &str = "SPIRV-Headers";

pub struct SpirvTools<'a> {
    src_root: &'a str,
    build_root: &'a str,
    spirv_headers_root: &'a str,
}

impl<'a> SpirvTools<'a> {
    fn spirv_headers_name(&self, str: &String) -> String {
        str.replace(self.spirv_headers_root, SPIRV_HEADERS)
            .replace("/", "_")
            .replace(".", "_")
    }
    pub fn new(src_root: &'a str, build_root: &'a str, spirv_headers_root: &'a str) -> Self {
        SpirvTools {
            src_root,
            build_root,
            spirv_headers_root,
        }
    }
    fn generate_spirv_headers(&self, files: HashSet<String>) -> Result<String, String> {
        let mut spirv_headers_package = String::new();

        let mut cc_library_headers = SoongModule::new("cc_library_headers");
        cc_library_headers.add_set("visibility", ["//visibility:public".to_string()].into());
        cc_library_headers.add_set("export_include_dirs", ["include".to_string()].into());
        cc_library_headers.add_str("name", SPIRV_HEADERS.to_string());
        spirv_headers_package += &match cc_library_headers.print() {
            Ok(content) => content,
            Err(err) => return Err(err),
        };

        for file in files {
            let mut genrule = SoongModule::new("genrule");
            genrule.add_str("name", self.spirv_headers_name(&file));
            genrule.add_set("visibility", ["//visibility:public".to_string()].into());
            genrule.add_set(
                "srcs",
                [file.replace(&add_slash_suffix(self.spirv_headers_root), "")].into(),
            );
            genrule.add_set("out", [file.rsplit_once("/").unwrap().1.to_string()].into());
            genrule.add_str("cmd", "cp $(in) $(out)".to_string());
            spirv_headers_package += &match genrule.print() {
                Ok(content) => content,
                Err(err) => return Err(err),
            };
        }

        return crate::filesystem::write_file(
            &(self.spirv_headers_root.to_string() + "/Android.bp"),
            spirv_headers_package,
        );
    }
}

impl<'a> crate::project::Project<'a> for SpirvTools<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let mut file = SoongFile::new(self.src_root, "", self.build_root, "spirvtools_");
        if let Err(err) = file.generate(
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
        match self.generate_spirv_headers(file.get_generated_headers()) {
            Ok(message) => println!("{message}"),
            Err(err) => return Err(err),
        }
        return file.write(self.src_root);
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
                    ":".to_string() + &self.spirv_headers_name(input),
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
    fn ignore_target(&self, _: &String) -> bool {
        false
    }
    fn ignore_include(&self, include: &str) -> bool {
        include.contains(self.build_root) || include.contains(self.spirv_headers_root)
    }
    fn rework_include(&self, include: &str) -> String {
        include.to_string()
    }
    fn get_headers_to_copy(&self, _: &HashSet<String>) -> HashSet<String> {
        return HashSet::new();
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
