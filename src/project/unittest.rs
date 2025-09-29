// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct UnitTest {
    targets_to_gen: Vec<NinjaTargetToGen>,
    test_path: PathBuf,
    ctx: Context,
}

fn generate_package<T>(targets: Vec<T>, project: &mut UnitTest) -> Result<String, String>
where
    T: NinjaTarget,
{
    SoongPackage::new(&[], "unittest_license", &[], &[])
        .generate(
            NinjaTargetsToGenMap::from(&project.targets_to_gen),
            targets,
            &project.test_path,
            &project.test_path,
            &project.test_path,
            None,
            project,
            &project.ctx,
        )?
        .print(&project.ctx)
}

impl Project for UnitTest {
    fn get_name(&self) -> &'static str {
        "unittests"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        error!("Should not be called")
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let Some(test_path) = ctx.unittest_path.clone() else {
            return error!("unittest_path not defined");
        };
        self.test_path = test_path.clone();
        self.ctx = ctx.clone();
        print_verbose!("'{}'", file_name(&test_path));
        let config = read_file(&test_path.join("config"))?;
        let mut lines = config.lines();
        let Some(ninja_generator) = lines.nth(0) else {
            return error!("Could not get ninja_generator from config file");
        };
        while let Some(target) = lines.nth(0) {
            self.targets_to_gen.push(target!(target));
        }
        match ninja_generator {
            "cmake" => generate_package(parse_build_ninja::<CmakeNinjaTarget>(&test_path)?, self),
            "meson" => generate_package(parse_build_ninja::<MesonNinjaTarget>(&test_path)?, self),
            "gn" => generate_package(parse_build_ninja::<GnNinjaTarget>(&test_path)?, self),
            _ => return error!("Unknown Ninja Generator"),
        }
    }
}
