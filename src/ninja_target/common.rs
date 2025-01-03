// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::*;

pub fn get_link_libraries(libs: &str) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut static_libraries = Vec::new();
    let mut shared_libraries = Vec::new();
    for lib in libs.split(" ") {
        if lib.is_empty() || lib == "-pthread" {
            continue;
        } else if let Some(library) = lib.strip_prefix("-l") {
            if library == "dl" || library == "m" || library == "c" {
                continue;
            }
            shared_libraries.push(PathBuf::from(format!("lib{library}")));
        } else {
            let lib_path = PathBuf::from(lib);
            if lib.contains(".a") {
                static_libraries.push(lib_path);
            } else if lib.contains(".so") {
                shared_libraries.push(lib_path);
            } else {
                return error!("unsupported library '{lib}'");
            }
        }
    }
    Ok((static_libraries, shared_libraries))
}

pub fn get_defines(defines: &str) -> Vec<String> {
    defines
        .split("-D")
        .filter_map(|define| {
            if define.is_empty() {
                return None;
            }
            Some(define.trim().replace("\\(", "(").replace("\\)", ")"))
        })
        .collect()
}

pub fn get_includes(includes: &str, build_path: &Path) -> Vec<PathBuf> {
    includes
        .split(" ")
        .map(|include| include.strip_prefix("-I").unwrap_or(include))
        .filter_map(|include| {
            if include.is_empty() || include == "isystem" {
                return None;
            }
            Some(canonicalize_path(include, build_path))
        })
        .collect()
}

pub fn get_link_flags(flags: &str) -> (Option<PathBuf>, Vec<String>) {
    let mut link_flags = Vec::new();
    let mut version_script = None;
    for flag in flags.split(" ") {
        if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
            version_script = Some(PathBuf::from(vs));
        }
        link_flags.push(String::from(flag));
    }
    (version_script, link_flags)
}

pub fn get_cflags(flags: &str) -> Vec<String> {
    flags
        .split(" ")
        .filter_map(|flag| {
            if flag.is_empty() {
                return None;
            }
            Some(String::from(flag))
        })
        .collect()
}

pub fn get_sources(inputs: &Vec<PathBuf>, build_path: &Path) -> Vec<PathBuf> {
    inputs
        .into_iter()
        .map(|input| canonicalize_path(input, build_path))
        .collect()
}
