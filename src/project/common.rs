// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

pub fn copy_gen_deps(
    mut gen_deps: Vec<PathBuf>,
    from: &str,
    build_path: &Path,
    ctx: &Context,
    project: &dyn Project,
) -> Result<(), String> {
    gen_deps.sort();
    write_file(
        &project.get_test_path(ctx).join("generated_deps.txt"),
        &format!("{0:#?}", &gen_deps),
    )?;
    if ctx.copy_to_aosp {
        copy_files(
            build_path,
            &project.get_android_path(ctx).join(from),
            gen_deps,
        )?;
    }
    Ok(())
}
