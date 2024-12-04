// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{Read, Write};

pub const TAB: &str = "   ";
pub const COLOR_RED: &str = "\x1b[00;31m";
pub const COLOR_GREEN: &str = "\x1b[00;32m";
pub const COLOR_GREEN_BOLD: &str = "\x1b[01;32m";
pub const COLOR_YELLOW_BOLD: &str = "\x1b[01;33m";
pub const COLOR_NONE: &str = "\x1b[0m";

#[macro_export]
macro_rules! print_internal {
    ($print_prefix:expr, $message_prefix:expr, $message:expr, $print_suffix:expr) => {
        println!(
            "{0}[NINJA-TO-SOONG]{1} {2}{3}",
            $print_prefix, $message_prefix, $message, $print_suffix
        );
    };
}
#[macro_export]
macro_rules! print_verbose {
    ($message:expr) => {
        print_internal!(COLOR_GREEN, format!("{COLOR_NONE}{TAB}{TAB}"), $message, "");
    };
}
#[macro_export]
macro_rules! print_debug {
    ($message:expr) => {
        print_internal!(COLOR_GREEN, format!("{COLOR_NONE}{TAB}"), $message, "");
    };
}
#[macro_export]
macro_rules! print_info {
    ($message:expr) => {
        print_internal!(
            format!("\n{COLOR_GREEN}"),
            COLOR_GREEN_BOLD,
            $message,
            COLOR_NONE
        );
    };
}
#[macro_export]
macro_rules! print_warn {
    ($message:expr) => {
        print_internal!(COLOR_YELLOW_BOLD, "", $message, COLOR_NONE);
    };
}
#[macro_export]
macro_rules! print_error {
    ($message:expr) => {
        print_internal!(COLOR_RED, "", $message, COLOR_NONE);
    };
}

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), $message))
    };
}
pub use {error, print_internal, print_verbose};

pub const CC_LIBRARY_HEADERS_SPIRV_TOOLS: &str = "SPIRV-Tools-includes";
pub const CC_LIBRARY_HEADERS_SPIRV_HEADERS: &str = "SPIRV-Headers-includes";
pub const CC_LIBRARY_HEADERS_LLVM: &str = "llvm-includes";
pub const CC_LIBRARY_HEADERS_CLANG: &str = "clang-includes";
pub const CC_LIBRARY_HEADERS_CLSPV: &str = "clspv-includes";

pub const ANDROID_NDK: &str = "android-ndk-r27c";
pub const ANDROID_ISA: &str = "aarch64"; // "x86_64"
pub const ANDROID_ABI: &str = "arm64-v8a"; // "x86_64"
pub const ANDROID_PLATFORM: &str = "35";

pub const LLVM_DISABLE_ZLIB: &str = "-DLLVM_ENABLE_ZLIB=OFF";

pub fn add_slash_suffix(str: &str) -> String {
    str.to_string() + "/"
}

pub fn rework_name(origin: String) -> String {
    origin.replace("/", "_").replace(".", "_")
}

pub fn spirv_headers_name(spirv_headers_dir: &str, str: &str) -> String {
    rework_name(str.replace(spirv_headers_dir, CC_LIBRARY_HEADERS_SPIRV_HEADERS))
}

pub fn clang_headers_name(clang_headers_dir: &str, str: &str) -> String {
    rework_name(str.replace(clang_headers_dir, CC_LIBRARY_HEADERS_CLANG))
}

pub fn llvm_project_headers_name(llvm_project_headers_dir: &str, str: &str) -> String {
    rework_name(str.replace(llvm_project_headers_dir, CC_LIBRARY_HEADERS_LLVM))
}

pub fn cmake_configure(
    src_dir: &str,
    build_dir: &str,
    ndk_dir: &str,
    args: Vec<&str>,
) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_CONFIGURE").is_ok() {
        return Ok(false);
    }
    let mut command = std::process::Command::new("cmake");
    command
        .args([
            "-B",
            build_dir,
            "-S",
            src_dir,
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            &("-DCMAKE_TOOLCHAIN_FILE=".to_string()
                + ndk_dir
                + "/build/cmake/android.toolchain.cmake"),
            &("-DANDROID_ABI=".to_string() + ANDROID_ABI),
            &("-DANDROID_PLATFORM=".to_string() + ANDROID_PLATFORM),
        ])
        .args(args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!(format!("cmake_configure({src_dir}) failed: {err}"));
    }
    Ok(true)
}

pub fn cmake_build(build_dir: &str, targets: &Vec<String>) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_BUILD").is_ok() {
        return Ok(false);
    }
    let targets_args = targets.into_iter().fold(Vec::new(), |mut vec, target| {
        vec.push("--target");
        vec.push(&target);
        vec
    });
    let mut command = std::process::Command::new("cmake");
    command.args(["--build", &build_dir]).args(targets_args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!(format!("cmake_build({build_dir}) failed: {err}"));
    }
    Ok(true)
}

pub fn copy_file(from: &str, to: &str) -> Result<(), String> {
    if let Err(err) = std::fs::copy(from, to) {
        return error!(format!("copy({from}, {to}) failed: {err}"));
    }
    Ok(())
}

pub fn write_file(file_path: &str, content: &str) -> Result<(), String> {
    match File::create(file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_fmt(format_args!("{0}", content)) {
                return error!(format!("Could not write into '{file_path}': '{err:#?}"));
            }
        }
        Err(err) => {
            return error!(format!("Could not create '{file_path}': '{err}'"));
        }
    }
    Ok(())
}

pub fn read_file(file_path: &str) -> Result<String, String> {
    match File::open(&file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            if let Err(err) = file.read_to_string(&mut content) {
                return error!(format!("Could not read '{file_path}': '{err}'"));
            }
            Ok(content.to_owned())
        }
        Err(err) => return error!(format!("Could not open '{file_path}': '{err}'")),
    }
}

pub fn get_tests_folder() -> Result<String, String> {
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
            Ok(tests_path.to_str().unwrap().to_string())
        }
        Err(err) => return error!(format!("Could not get current executable path: {err}")),
    }
}
