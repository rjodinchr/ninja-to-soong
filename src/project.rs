use std::collections::HashSet;

use crate::target::BuildTarget;
use crate::utils::error;

pub mod clspv;
pub mod clvk;
pub mod llvm;
pub mod spirvheaders;
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
    fn get_default_cflags(&self) -> HashSet<String> {
        HashSet::new()
    }
    fn get_headers_to_copy(&self, _headers: &HashSet<String>) -> HashSet<String> {
        HashSet::new()
    }
    fn get_headers_to_generate(&self, _headers: &HashSet<String>) -> HashSet<String> {
        HashSet::new()
    }
    fn get_target_header_libs(&self, _target: &String) -> HashSet<String> {
        HashSet::new()
    }
    fn get_target_stem(&self, _target: &String) -> String {
        String::new()
    }
    fn get_library_name(&self, library: &str) -> String {
        library.to_string()
    }
    fn optimize_target_for_size(&self, _target: &String) -> bool {
        false
    }
    fn ignore_target(&self, _target: &String) -> bool {
        false
    }
    fn ignore_include(&self, _include: &str) -> bool {
        false
    }
    fn ignore_define(&self, _define: &str) -> bool {
        false
    }
    fn rework_include(&self, include: &str) -> String {
        include.to_string()
    }
    fn rework_command_output(&self, output: &str) -> String {
        output.to_string()
    }
    fn handle_link_flag(&self, _flag: &str, _link_flags: &mut HashSet<String>) {}
}
