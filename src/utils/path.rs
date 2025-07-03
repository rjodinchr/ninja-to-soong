// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::path::*;

use crate::context::*;

pub use std::path::{Path, PathBuf};

use super::*;

pub fn get_ndk_path(temp_path: &Path, ctx: &Context) -> Result<PathBuf, String> {
    let android_ndk = if let Ok(android_ndk) = env::var("N2S_NDK") {
        android_ndk
    } else {
        String::from("android-ndk-r27c")
    };
    let ndk_path = if let Ok(ndk_path) = env::var("N2S_NDK_PATH") {
        PathBuf::from(ndk_path)
    } else {
        PathBuf::from(temp_path)
    };
    let android_ndk_path = ndk_path.join(&android_ndk);
    if android_ndk_path.exists() || ctx.skip_gen_ninja {
        return Ok(android_ndk_path);
    }

    let ndk_zip = path_to_string(ndk_path.join("android-ndk.zip"));
    let ndk_url = format!("https://dl.google.com/android/repository/{android_ndk}-linux.zip");
    execute_cmd!("wget", [&ndk_url, "-q", "-O", &ndk_zip])?;
    execute_cmd!("unzip", ["-q", &ndk_zip, "-d", &path_to_string(ndk_path)])?;
    Ok(android_ndk_path)
}

pub fn canonicalize_path<P: AsRef<Path>>(path: P, build_path: &Path) -> PathBuf {
    let path_buf = PathBuf::from(path.as_ref());
    if path_buf.has_root() {
        path_buf
    } else {
        build_path
            .join(&path_buf)
            .components()
            .fold(PathBuf::new(), |path, component| {
                if component == Component::ParentDir {
                    PathBuf::from(path.parent().unwrap())
                } else {
                    path.join(component)
                }
            })
    }
}

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    String::from(path.as_ref().to_str().unwrap_or_default())
}

pub fn path_to_string_with_separator<P: AsRef<Path>>(path: P) -> String {
    format!("{0}{1}", path_to_string(path), MAIN_SEPARATOR_STR)
}

pub fn path_to_id(path: PathBuf) -> String {
    let path = path.to_str().unwrap_or_default();
    if path.starts_with("//") {
        return String::from(path);
    }
    path.replace(MAIN_SEPARATOR_STR, "_").replace(".", "_")
}

pub fn file_stem(path: &Path) -> String {
    let file_name = file_name(path);
    String::from(file_name.split_once(".").unwrap_or((&file_name, "")).0)
}

pub fn file_name(path: &Path) -> String {
    String::from(
        path.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
    )
}

pub fn strip_prefix<F: AsRef<Path>, P: AsRef<Path>>(from: F, prefix: P) -> PathBuf {
    PathBuf::from(from.as_ref().strip_prefix(prefix).unwrap_or(from.as_ref()))
}

pub fn wildcardize_path(path: &Path) -> PathBuf {
    path.parent().unwrap().join(
        String::from("*.")
            + path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default(),
    )
}

pub fn wildcardize_paths(paths: Vec<String>, src_path: &Path) -> Vec<String> {
    let mut wildcardized_paths = Vec::new();
    let mut paths = paths
        .into_iter()
        .map(|path| canonicalize_path(path, src_path))
        .collect::<Vec<_>>();
    'outer: while let Some(path) = paths.pop() {
        let wildcard = wildcardize_path(&path);
        let files = ls_regex(&wildcard);
        if files.len() <= 1 {
            wildcardized_paths.push(path_to_string(&path));
            continue;
        }
        for file in &files {
            if !paths.contains(&file) && file != &path {
                wildcardized_paths.push(path_to_string(&path));
                continue 'outer;
            }
        }
        wildcardized_paths.push(path_to_string(wildcard));
        paths = paths
            .into_iter()
            .filter(|path| !files.contains(&path))
            .collect();
    }
    wildcardized_paths
        .into_iter()
        .map(|path| path_to_string(strip_prefix(path, src_path)))
        .collect()
}
