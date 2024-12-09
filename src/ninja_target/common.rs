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
            shared_libraries.push(PathBuf::from("lib".to_string() + library));
        } else {
            let lib_path = PathBuf::from(lib);
            if lib.ends_with(".a") {
                static_libraries.push(lib_path);
            } else if lib.ends_with(".so") {
                shared_libraries.push(lib_path);
            } else {
                return error!("unsupported library '{lib}'");
            }
        }
    }
    Ok((static_libraries, shared_libraries))
}

pub fn get_defines(defs: &str) -> Vec<String> {
    let mut defines = Vec::new();
    for define in defs.split("-D") {
        if define.is_empty() {
            continue;
        }
        defines.push(define.trim().replace("\\(", "(").replace("\\)", ")"));
    }
    defines
}

pub fn get_includes(incs: &str, build_path: &Path) -> Vec<PathBuf> {
    let mut includes = Vec::new();
    for inc in incs.split(" ") {
        let include = inc.strip_prefix("-I").unwrap_or(inc);
        if include.is_empty() || include == "isystem" {
            continue;
        }
        includes.push(canonicalize_path(include, build_path));
    }
    includes
}

pub fn get_link_flags(flags: &str) -> (Option<PathBuf>, Vec<String>) {
    let mut link_flags = Vec::new();
    let mut version_script = None;
    for flag in flags.split(" ") {
        if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
            version_script = Some(PathBuf::from(vs));
        }
        link_flags.push(flag.to_string());
    }
    (version_script, link_flags)
}

pub fn get_cflags(flags: &str) -> Vec<String> {
    let mut cflags = Vec::new();
    for flag in flags.split(" ") {
        if flag.is_empty() {
            continue;
        }
        cflags.push(flag.to_string());
    }
    cflags
}

pub fn get_sources(ins: &Vec<PathBuf>, build_path: &Path) -> Vec<PathBuf> {
    let mut inputs = Vec::new();
    for input in ins {
        inputs.push(canonicalize_path(input, build_path));
    }
    inputs
}
