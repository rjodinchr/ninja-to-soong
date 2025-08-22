// Copyright 2025 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[derive(Default)]
pub struct UnitTest();

fn generate_package<T>(
    targets: Vec<T>,
    targets_to_gen: Vec<NinjaTargetToGen>,
    project: &mut UnitTest,
    test_path: &Path,
    ctx: &Context,
) -> Result<String, String>
where
    T: NinjaTarget,
{
    SoongPackage::new(&[], "unittest_license", &[], &[])
        .generate(
            NinjaTargetsToGenMap::from(&targets_to_gen),
            targets,
            test_path,
            test_path,
            test_path,
            None,
            project,
            ctx,
        )?
        .print(ctx)
}

impl Project for UnitTest {
    fn get_name(&self) -> &'static str {
        "unittests"
    }
    fn get_android_path(&self) -> Result<PathBuf, String> {
        error!("Should not be called")
    }
    fn get_test_path(&self) -> Result<PathBuf, String> {
        Ok(PathBuf::from(""))
    }
    fn generate_package(
        &mut self,
        ctx: &Context,
        _projects_map: &ProjectsMap,
    ) -> Result<String, String> {
        let test_path = ctx.get_test_path(self)?;
        print_verbose!("'{}'", file_name(&test_path));
        let config = read_file(&test_path.join("config"))?;
        let mut lines = config.lines();
        let Some(ninja_generator) = lines.nth(0) else {
            return error!("Could not get ninja_generator from config file");
        };
        let mut targets_to_gen = Vec::new();
        while let Some(target) = lines.nth(0) {
            targets_to_gen.push(target!(target));
        }
        match ninja_generator {
            "cmake" => generate_package(
                parse_build_ninja::<CmakeNinjaTarget>(&test_path)?,
                targets_to_gen,
                self,
                &test_path,
                ctx,
            ),
            "meson" => generate_package(
                parse_build_ninja::<MesonNinjaTarget>(&test_path)?,
                targets_to_gen,
                self,
                &test_path,
                ctx,
            ),
            "gn" => generate_package(
                parse_build_ninja::<GnNinjaTarget>(&test_path)?,
                targets_to_gen,
                self,
                &test_path,
                ctx,
            ),
            _ => return error!("Unknown Ninja Generator"),
        }
    }
}
