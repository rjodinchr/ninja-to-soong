// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{Read, Write};

pub use std::path::{Path, PathBuf};

pub const LLVM_DISABLE_ZLIB: &str = "-DLLVM_ENABLE_ZLIB=OFF";

pub const TAB: &str = "   ";
pub const COLOR_RED: &str = "\x1b[00;31m";
pub const COLOR_GREEN: &str = "\x1b[00;32m";
pub const COLOR_GREEN_BOLD: &str = "\x1b[01;32m";
pub const COLOR_NONE: &str = "\x1b[0m";

#[macro_export]
macro_rules! print_internal {
    ($print_prefix:expr, $message_prefix:expr, $print_suffix:expr, $($arg:tt)*) => {
        println!(
            "{0}[N2S]{1} {2}{3}",
            $print_prefix, $message_prefix, format!($($arg)*), $print_suffix
        );
    };
}
#[macro_export]
macro_rules! print_verbose {
    ($($arg:tt)*) => {
        print_internal!(COLOR_GREEN, format!("{COLOR_NONE}{TAB}{TAB}"), "", $($arg)*);
    };
}
#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {
        print_internal!(COLOR_GREEN, format!("{COLOR_NONE}{TAB}"), "", $($arg)*);
    };
}
#[macro_export]
macro_rules! print_info {
    ($($arg:tt)*) => {
        print_internal!(
            format!("\n{COLOR_GREEN}"),
            COLOR_GREEN_BOLD,

            COLOR_NONE,
            $($arg)*,
        );
    };
}
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        print_internal!(COLOR_RED, "", COLOR_NONE, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), format!($($arg)*)))
    };
}
pub use {error, print_internal, print_verbose};

const ARM: bool = true;
pub const ANDROID_NDK: &str = "android-ndk-r27c";
pub const ANDROID_PLATFORM: &str = "35";
pub const ANDROID_ISA: &str = if ARM { "aarch64" } else { "x86_64" };
pub const ANDROID_ABI: &str = if ARM { "arm64-v8a" } else { "x86_64" };
pub const ANDROID_CPU: &str = if ARM { "arm64" } else { "x64" };

pub const NDK_CMAKE_TOOLCHAIN_PATH: &str = "build/cmake/android.toolchain.cmake";

pub fn get_ndk_path(temp_path: &Path) -> Result<PathBuf, String> {
    let android_ndk = if let Ok(android_ndk) = std::env::var("N2S_NDK") {
        android_ndk
    } else {
        ANDROID_NDK.to_string()
    };
    let ndk_path = if let Ok(ndk_path) = std::env::var("N2S_NDK_PATH") {
        PathBuf::from(ndk_path)
    } else {
        temp_path.to_path_buf()
    };
    let android_ndk_path = ndk_path.join(&android_ndk);
    if File::open(android_ndk_path.join(NDK_CMAKE_TOOLCHAIN_PATH)).is_ok() {
        return Ok(android_ndk_path);
    }

    let ndk_zip = path_to_string(ndk_path.join("android-ndk.zip"));
    let ndk_url =
        "https://dl.google.com/android/repository/".to_string() + &android_ndk + "-linux.zip";
    execute_cmd!("wget", vec![&ndk_url, "-q", "-O", &ndk_zip], None)?;
    execute_cmd!(
        "unzip",
        vec!["-q", &ndk_zip, "-d", &path_to_string(ndk_path)],
        None
    )?;
    Ok(android_ndk_path)
}

pub fn canonicalize_path<P: AsRef<Path>>(path: P, build_path: &Path) -> PathBuf {
    let path_buf = path.as_ref().to_path_buf();
    if path_buf.has_root() {
        path_buf
    } else {
        build_path
            .join(&path_buf)
            .components()
            .fold(PathBuf::new(), |path, component| {
                if component == std::path::Component::ParentDir {
                    path.parent().unwrap().to_path_buf()
                } else {
                    path.join(component)
                }
            })
    }
}

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_str().unwrap_or_default().to_string()
}

pub fn path_to_id(path: PathBuf) -> String {
    path.to_str()
        .unwrap_or_default()
        .replace(std::path::MAIN_SEPARATOR, "_")
        .replace(".", "_")
}

pub fn file_stem(path: &Path) -> String {
    let file_name = file_name(path);
    file_name
        .split_once(".")
        .unwrap_or((&file_name, ""))
        .0
        .to_string()
}

pub fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string()
}

pub fn strip_prefix<F: AsRef<Path>, P: AsRef<Path>>(from: F, prefix: P) -> PathBuf {
    PathBuf::from(from.as_ref().strip_prefix(prefix).unwrap_or(from.as_ref()))
}

pub fn split_path(path: &Path, delimiter: &str) -> Option<(PathBuf, PathBuf)> {
    let mut sub_path = path;
    while sub_path.parent().is_some() {
        sub_path = sub_path.parent().unwrap();
        if file_name(sub_path) == delimiter {
            return Some((sub_path.to_path_buf(), strip_prefix(path, sub_path)));
        }
    }
    None
}

pub fn dep_name<P: AsRef<Path>>(from: &Path, prefix: P, path: &str, build_path: &Path) -> String {
    path_to_id(Path::new(path).join(strip_prefix(
        canonicalize_path(from, build_path),
        canonicalize_path(prefix, build_path),
    )))
}

pub fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Err(err) = std::fs::copy(from, to) {
        return error!("copy({from:#?}, {to:#?}) failed: {err}");
    }
    Ok(())
}

pub fn write_file(file_path: &Path, content: &str) -> Result<(), String> {
    match File::create(file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_fmt(format_args!("{0}", content)) {
                return error!("Could not write into {file_path:#?}: '{err:#?}");
            }
        }
        Err(err) => {
            return error!("Could not create {file_path:#?}: '{err}'");
        }
    }
    Ok(())
}

pub fn read_file(file_path: &Path) -> Result<String, String> {
    match File::open(&file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            if let Err(err) = file.read_to_string(&mut content) {
                return error!("Could not read {file_path:#?}: '{err}'");
            }
            Ok(content)
        }
        Err(err) => return error!("Could not open {file_path:#?}: '{err}'"),
    }
}

pub fn execute_command(
    program: &str,
    args: Vec<&str>,
    env_vars: Option<Vec<(&str, &str)>>,
    description: String,
) -> Result<(), String> {
    let mut command = std::process::Command::new(program);
    command.args(args);
    if let Some(vec_env_vars) = env_vars {
        for (key, val) in vec_env_vars {
            command.env(key, val);
        }
    }
    println!("{command:#?}");
    match command.status() {
        Ok(status) => {
            if !status.success() {
                return error!("{description} failed");
            }
        }
        Err(err) => return error!("{description} failed: {err}"),
    }
    Ok(())
}

#[macro_export]
macro_rules! execute_cmd {
    ($program:expr, $args:expr, $env_vars:expr) => {
        execute_command(
            $program,
            $args,
            $env_vars,
            format!("{0}:{1}: {2}", file!(), line!(), $program),
        )
    };
}
pub use execute_cmd;
