// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{Read, Write};

pub use std::path::{Path, PathBuf};

pub const TAB: &str = "   ";
pub const COLOR_RED: &str = "\x1b[00;31m";
pub const COLOR_GREEN: &str = "\x1b[00;32m";
pub const COLOR_GREEN_BOLD: &str = "\x1b[01;32m";
pub const COLOR_YELLOW_BOLD: &str = "\x1b[01;33m";
pub const COLOR_NONE: &str = "\x1b[0m";

#[macro_export]
macro_rules! print_internal {
    ($print_prefix:expr, $message_prefix:expr, $print_suffix:expr, $($arg:tt)*) => {
        println!(
            "{0}[NINJA-TO-SOONG]{1} {2}{3}",
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
macro_rules! print_warn {
    ($($arg:tt)*) => {
        print_internal!(COLOR_YELLOW_BOLD, "", COLOR_NONE, $($arg)*);
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

pub const ANDROID_NDK: &str = "android-ndk-r27c";
pub const ANDROID_ISA: &str = "aarch64"; // "x86_64"
pub const ANDROID_ABI: &str = "arm64-v8a"; // "x86_64"
pub const ANDROID_PLATFORM: &str = "35";

pub const LLVM_DISABLE_ZLIB: &str = "-DLLVM_ENABLE_ZLIB=OFF";

pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_str().unwrap_or_default().to_string()
}

pub fn path_to_id(path: PathBuf) -> String {
    path.to_str()
        .unwrap_or_default()
        .replace(std::path::MAIN_SEPARATOR, "_")
        .replace(".", "_")
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

pub fn dep_name<P: AsRef<Path>>(from: &Path, prefix: P, path: &str) -> String {
    path_to_id(Path::new(path).join(strip_prefix(from, prefix)))
}

pub fn cmake_configure(
    src_path: &Path,
    build_path: &Path,
    ndk_path: &Path,
    args: Vec<&str>,
) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_CONFIGURE").is_ok() {
        return Ok(false);
    }
    let mut command = std::process::Command::new("cmake");
    command
        .args([
            "-B",
            &path_to_string(build_path),
            "-S",
            &path_to_string(src_path),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            &("-DCMAKE_TOOLCHAIN_FILE=".to_string()
                + &path_to_string(ndk_path.join("build/cmake/android.toolchain.cmake"))),
            &("-DANDROID_ABI=".to_string() + ANDROID_ABI),
            &("-DANDROID_PLATFORM=".to_string() + ANDROID_PLATFORM),
        ])
        .args(args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!("cmake_configure({src_path:#?}) failed: {err}");
    }
    Ok(true)
}

pub fn cmake_build(build_path: &Path, targets: &Vec<PathBuf>) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_BUILD").is_ok() {
        return Ok(false);
    }
    let targets_args = targets.into_iter().fold(Vec::new(), |mut vec, target| {
        vec.push("--target");
        vec.push(target.to_str().unwrap_or_default());
        vec
    });
    let mut command = std::process::Command::new("cmake");
    command
        .args(["--build", &path_to_string(build_path)])
        .args(targets_args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!("cmake_build({build_path:#?}) failed: {err}");
    }
    Ok(true)
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

pub fn get_tests_folder() -> Result<PathBuf, String> {
    match std::env::current_exe() {
        Ok(exe_path) => {
            let tests_path = exe_path // <ninja-to-soong>/target/debug/ninja-to-soong
                .parent() // <ninja-to-soong>/target/debug
                .unwrap()
                .parent() // <ninja-to-soong>/target
                .unwrap()
                .parent() // <ninja-to-soong>
                .unwrap()
                .join("tests"); // <ninja-to-soong>/tests
            Ok(tests_path)
        }
        Err(err) => return error!("Could not get current executable path: {err}"),
    }
}
