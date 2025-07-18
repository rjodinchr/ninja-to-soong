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
            &project.get_test_path(ctx)?.join("generated_deps.txt"),
            &format!("{0:#?}", &gen_deps),
        )?;
    } else {
        let dst = project.get_android_path(ctx)?.join(from);
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
                .chain(std::iter::once(""))
                .collect::<Vec<&str>>()
                .join("\n"),
        )?;
    }
    Ok(())
}

pub fn ninja_build(build_path: &Path, targets: &Vec<PathBuf>) -> Result<(), String> {
    let mut args = vec![String::from("-C"), path_to_string(&build_path)];
    for target in targets {
        args.push(path_to_string(target));
    }
    let args: Vec<_> = args.iter().map(|target| target.as_str()).collect();
    execute_cmd!("ninja", &args)?;
    Ok(())
}

#[allow(dead_code)]
pub fn cmake_configure(
    src_path: &Path,
    build_path: &Path,
    ndk_path: &Path,
    android_abi: Option<&str>,
    android_platform: Option<&str>,
    build_type: Option<&str>,
    extra_args: &[&str],
) -> Result<(), String> {
    let android_abi = format!(
        "-DANDROID_ABI={}",
        match android_abi {
            Some(android_abi) => android_abi,
            None => "arm64-v8a",
        }
    );
    let android_platform = format!(
        "-DANDROID_PLATFORM={}",
        match android_platform {
            Some(android_platform) => android_platform,
            None => "35",
        }
    );
    let build_type = format!(
        "-DCMAKE_BUILD_TYPE={}",
        match build_type {
            Some(build_type) => build_type,
            None => "Release",
        }
    );
    let build_path = path_to_string(build_path);
    let src_path = path_to_string(src_path);
    let cmake_toolchain_file = format!(
        "-DCMAKE_TOOLCHAIN_FILE={}/build/cmake/android.toolchain.cmake",
        path_to_string(&ndk_path)
    );
    let mut args = vec![
        "-B",
        &build_path,
        "-S",
        &src_path,
        "-G",
        "Ninja",
        &build_type,
        &cmake_toolchain_file,
        &android_abi,
        &android_platform,
    ];
    args.extend(extra_args);
    execute_cmd!("cmake", &args)?;
    Ok(())
}
