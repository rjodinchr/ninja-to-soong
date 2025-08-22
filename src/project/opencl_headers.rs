// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct OpenclHeaders();

impl Project for OpenclHeaders {
    fn get_name(&self) -> &'static str {
        "OpenCL-Headers"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        error!("Should not be called")
    }
    fn get_test_path(&self, ctx: &Context) -> Result<PathBuf, String> {
        Ok(ctx.test_path.join(self.get_name()))
    }
    fn generate_package(
        &mut self,
        _ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        Ok(String::from(""))
    }
}
