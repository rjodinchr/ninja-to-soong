use std::collections::HashSet;

use crate::target::BuildTarget;
use crate::utils::error;

pub mod clspv;
pub mod clvk;
pub mod llvm;
pub mod spirvtools;

pub trait Project<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String>;
    fn parse_custom_command_inputs(
        &self,
        _: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String> {
        error!(format!(
            "parse_custom_command_inputs not implemented by this project"
        ))
    }
    fn get_default_defines(&self) -> HashSet<String> {
        HashSet::new()
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
        HashSet::new()
    }
    fn get_headers_to_generate(&self, _: &HashSet<String>) -> HashSet<String> {
        HashSet::new()
    }
    fn get_object_header_libs(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn get_library_name(&self, library: &str) -> String {
        library.to_string()
    }
    fn handle_link_flag(&self, _: &str, _: &mut HashSet<String>) {}
    fn rework_output_path(&self, output: &str) -> String {
        output.to_string()
    }
}
