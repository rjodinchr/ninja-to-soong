// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

pub fn copy_gen_deps(
    gen_deps: Vec<PathBuf>,
    from: &str,
    build_path: &Path,
    ctx: &Context,
    project: &dyn Project,
) -> Result<(), String> {
    if !ctx.copy_to_aosp {
        write_file(
            &ctx.get_test_path(project).join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps),
        )?;
    } else {
        let dst = ctx.get_android_path(project)?.join(from);
        if remove_dir(&dst)? {
            print_verbose!("{dst:#?} removed");
        }
        for file in gen_deps {
            let from = build_path.join(&file);
            let to = dst.join(&file);
            let to_path = to.parent().unwrap();
            create_dir(to_path)?;
            copy_file(&from, &to)?;
        }
        print_verbose!("Files copied from {build_path:#?} to {dst:#?}");
    }
    Ok(())
}

pub fn ninja_build(build_path: &Path, targets: &Vec<PathBuf>, ctx: &Context) -> Result<(), String> {
    if ctx.skip_build {
        return Ok(());
    }
    let mut args = vec![String::from("-C"), path_to_string(&build_path)];
    for target in targets {
        args.push(path_to_string(target));
    }
    let args: Vec<_> = args.iter().map(|target| target.as_str()).collect();
    execute_cmd!("ninja", &args)?;
    Ok(())
}
