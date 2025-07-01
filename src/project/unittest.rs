// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct UnitTest();

fn generate_package<T>(
    targets: Vec<T>,
    target: &str,
    project: &mut UnitTest,
) -> Result<String, String>
where
    T: NinjaTarget,
{
    SoongPackage::new(&[], "unittest_license", &[], &[])
        .generate(
            NinjaTargetsToGenMap::from(&[NinjaTargetToGen(&target, None, None)]),
            targets,
            Path::new(""),
            Path::new(""),
            Path::new("/ninja-to-soong/unit/test/build/folder"),
            None,
            project,
        )?
        .print()
}

impl Project for UnitTest {
    fn get_name(&self) -> &'static str {
        "unittests"
    }
    fn get_android_path(&self, _ctx: &Context) -> PathBuf {
        PathBuf::from("/dev/null")
    }
    fn get_test_path(&self, ctx: &Context) -> PathBuf {
        ctx.test_path.clone()
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let config = read_file(&ctx.test_path.join("config"))?;
        let mut lines = config.lines();
        let Some(target) = lines.nth(0) else {
            return error!("Could not get target from config file");
        };
        let Some(ninja_generator) = lines.nth(0) else {
            return error!("Could not get ninja_generator from config file");
        };
        match ninja_generator {
            "cmake" => generate_package(
                parse_build_ninja::<CmakeNinjaTarget>(&ctx.test_path)?,
                &target,
                self,
            ),
            "meson" => generate_package(
                parse_build_ninja::<MesonNinjaTarget>(&ctx.test_path)?,
                &target,
                self,
            ),
            "gn" => generate_package(
                parse_build_ninja::<GnNinjaTarget>(&ctx.test_path)?,
                &target,
                self,
            ),
            _ => return error!("Unknown Ninja Generator"),
        }
    }
}
