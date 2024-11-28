use std::collections::HashSet;

use crate::target::BuildTarget;

pub mod clspv;
pub mod llvm;
pub mod spirvtools;

pub trait Project<'a> {
    fn generate(self, targets: Vec<BuildTarget>) -> Result<String, String>;
    fn parse_custom_command_inputs(
        &self,
        inputs: &Vec<String>,
    ) -> Result<(HashSet<String>, HashSet<String>, HashSet<(String, String)>), String>;
    fn get_default_defines(&self) -> HashSet<String>;
    fn ignore_target(&self, input: &String) -> bool;
    fn ignore_include(&self, include: &str) -> bool;
    fn rework_include(&self, include: &str) -> String;
    fn get_headers_to_copy(&self, headers: &HashSet<String>) -> HashSet<String>;
    fn get_headers_to_generate(&self, headers: &HashSet<String>) -> HashSet<String>;
    fn get_object_header_libs(&self) -> HashSet<String>;
}
