// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::utils::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Library {
    Shared,
    Static,
    StaticWhole,
}

fn get_libs(libs: &str, target: Library) -> Vec<PathBuf> {
    let mut prev_state: Option<Library> = None;
    let mut state: Option<Library> = None;
    libs.split(" ")
        .filter_map(|lib| {
            if lib.is_empty() || lib == "-pthread" {
                return None;
            } else if let Some(library) = lib.strip_prefix("-l") {
                if library == "dl"
                    || library == "m"
                    || library == "c"
                    || library == "pthread"
                    || library == "atomic"
                {
                    return None;
                }
                if state == Some(target.clone()) || (target == Library::Shared && state.is_none()) {
                    return Some(PathBuf::from(format!("lib{library}")));
                }
            } else if let Some(arg) = lib.strip_prefix("-Wl,") {
                if arg == "-Bstatic" {
                    state = Some(Library::Static)
                } else if arg == "-Bdynamic" {
                    state = Some(Library::Shared)
                } else if arg == "--whole-archive" {
                    prev_state = state.clone();
                    state = Some(Library::StaticWhole)
                } else if arg == "--no-whole-archive" {
                    state = prev_state.clone()
                }
            } else if lib.ends_with(".a") {
                if (target == Library::Static && state != Some(Library::StaticWhole))
                    || (target == Library::StaticWhole && state == Some(Library::StaticWhole))
                {
                    return Some(PathBuf::from(lib));
                }
            } else if lib.ends_with(".so") {
                if target == Library::Shared {
                    return Some(PathBuf::from(lib));
                }
            }
            return None;
        })
        .collect()
}

pub fn get_libs_static(libs: &str) -> Vec<PathBuf> {
    get_libs(libs, Library::Static)
}

pub fn get_libs_shared(libs: &str) -> Vec<PathBuf> {
    get_libs(libs, Library::Shared)
}

pub fn get_libs_static_whole(libs: &str) -> Vec<PathBuf> {
    get_libs(libs, Library::StaticWhole)
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
            if include.is_empty() || include == "-isystem" {
                return None;
            }
            Some(canonicalize_path(include, build_path))
        })
        .collect()
}

pub fn get_link_flags(flags: &str) -> (Option<PathBuf>, Vec<String>) {
    let mut link_flags = Vec::new();
    let mut version_script = None;
    let mut next_is_version_script = false;
    for flag in flags.split(" ") {
        if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
            version_script = Some(PathBuf::from(vs));
        } else if flag == "-Wl,--version-script" {
            next_is_version_script = true;
        } else if next_is_version_script {
            next_is_version_script = false;
            if let Some(vs) = flag.strip_prefix("-Wl,") {
                version_script = Some(PathBuf::from(vs));
            }
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

pub fn get_cmd(cmd: &str) -> String {
    cmd.replace("$ ", " ")
}
