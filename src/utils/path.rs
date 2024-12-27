// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::path::*;

pub use std::path::{Path, PathBuf};

use super::*;

const ARM: bool = true;
pub const ANDROID_NDK: &str = "android-ndk-r27c";
pub const ANDROID_PLATFORM: &str = "35";
pub const ANDROID_ISA: &str = if ARM { "aarch64" } else { "x86_64" };
pub const ANDROID_ABI: &str = if ARM { "arm64-v8a" } else { "x86_64" };
pub const ANDROID_CPU: &str = if ARM { "arm64" } else { "x64" };

pub fn get_ndk_path(temp_path: &Path) -> Result<PathBuf, String> {
    let android_ndk = if let Ok(android_ndk) = env::var("N2S_NDK") {
        android_ndk
    } else {
        String::from(ANDROID_NDK)
    };
    let ndk_path = if let Ok(ndk_path) = env::var("N2S_NDK_PATH") {
        PathBuf::from(ndk_path)
    } else {
        PathBuf::from(temp_path)
    };
    let android_ndk_path = ndk_path.join(&android_ndk);
    if exists(&android_ndk_path) {
        return Ok(android_ndk_path);
    }

    let ndk_zip = path_to_string(ndk_path.join("android-ndk.zip"));
    let ndk_url = format!("https://dl.google.com/android/repository/{android_ndk}-linux.zip");
    execute_cmd!("wget", vec![&ndk_url, "-q", "-O", &ndk_zip])?;
    execute_cmd!(
        "unzip",
        vec!["-q", &ndk_zip, "-d", &path_to_string(ndk_path)]
    )?;
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
    path.to_str()
        .unwrap_or_default()
        .replace(MAIN_SEPARATOR_STR, "_")
        .replace(".", "_")
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
