use std::collections::HashSet;

use crate::soongfile::SoongFile;
use crate::target::BuildTarget;
use crate::utils::add_slash_suffix;

pub struct CLSPV<'a> {
    src_root: &'a str,
    build_root: &'a str,
}

impl<'a> CLSPV<'a> {
    pub fn new(src_root: &'a str, build_root: &'a str) -> Self {
        CLSPV {
            src_root,
            build_root,
        }
    }
}

impl<'a> crate::project::Project<'a> for CLSPV<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String> {
        let mut file = SoongFile::new(self.src_root, "", self.build_root, "clspv_");
        if let Err(err) = file.generate(vec!["libclspv_core.a"], targets, &self) {
            return Err(err);
        }
        return file.write(self.src_root);
    }
    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<
        (
            HashSet<String>,
            HashSet<String>,
            HashSet<(String, String)>,
        ),
        String,
    > {
        let mut srcs: HashSet<String> = HashSet::new();
        let mut filtered_inputs: HashSet<String> = HashSet::new();
        for input in inputs {
            filtered_inputs.insert(input.clone());
        }
        for input in &filtered_inputs {
            srcs.insert(input.replace(&add_slash_suffix(self.src_root), ""));
        }
        return Ok((srcs, filtered_inputs, HashSet::new()));
    }
    fn get_default_defines(&self) -> HashSet<String> {
        return HashSet::new();
    }
    fn ignore_target(&self, _: &String) -> bool {
        false
    }
    fn ignore_include(&self, _: &str) -> bool {
        false
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
        return HashSet::new();
    }
}
