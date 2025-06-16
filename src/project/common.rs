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
    write_file(
        &project.get_test_path(ctx).join("generated_deps.txt"),
        &format!("{0:#?}", &gen_deps),
    )?;
    if ctx.copy_to_aosp {
        let dst = project.get_android_path(ctx).join(from);
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

pub fn clean_gen_deps(
    gen_deps: &Vec<PathBuf>,
    build_path: &Path,
    ctx: &Context,
) -> Result<(), String> {
    if !ctx.copy_to_aosp {
        return Ok(());
    }
    for gen_dep in gen_deps {
        let file_path = build_path.join(gen_dep);
        let file_extension = file_path.extension().unwrap().to_str().unwrap();
        if !["c", "cpp", "h"].contains(&file_extension) {
            continue;
        }
        write_file(
            &file_path,
            &read_file(&file_path)?
                .lines()
                .into_iter()
                .filter(|line| !line.starts_with("#line"))
                .collect::<Vec<&str>>()
                .join("\n"),
        )?;
    }
    Ok(())
}

pub fn cmake_build(build_path: &Path, targets: &Vec<PathBuf>) -> Result<(), String> {
    let mut args = vec![String::from("--build"), path_to_string(&build_path)];
    for target in targets {
        args.push(String::from("--target"));
        args.push(path_to_string(target));
    }
    let args: Vec<&str> = args.iter().map(|target| target.as_str()).collect();
    execute_cmd!("cmake", &args)?;
    Ok(())
}
