// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct External();

impl Project for External {
    fn get_name(&self) -> &'static str {
        "external"
    }
    fn get_android_path(&self, _ctx: &Context) -> Result<PathBuf, String> {
        error!("Should not be called")
    }
    fn get_test_path(&self, _ctx: &Context) -> Result<PathBuf, String> {
        error!("Should not be called")
    }
    fn generate_package(
        &mut self,
        _ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        error!("Should not be called")
    }
}
