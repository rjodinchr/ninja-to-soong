// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::fs::*;
use std::io::{Read, Write};

use super::*;

pub fn remove_dir(dir: &Path) -> Result<bool, String> {
    if !dir.exists() {
        return Ok(false);
    }
    if let Err(err) = remove_dir_all(dir) {
        return error!("remove_dir_all({dir:#?}) failed: {err}");
    }
    Ok(true)
}

pub fn create_dir(dir: &Path) -> Result<bool, String> {
    if dir.exists() {
        return Ok(false);
    }
    if let Err(err) = create_dir_all(dir) {
        return error!("create_dir_all({dir:#?}) failed: '{err}'");
    }
    Ok(true)
}

pub fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Err(err) = copy(from, to) {
        return error!("copy({from:#?}, {to:#?}) failed: '{err}'");
    }
    Ok(())
}

pub fn write_file(file_path: &Path, content: &str) -> Result<(), String> {
    match File::create(file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_fmt(format_args!("{0}", content)) {
                return error!("write_fmt({file_path:#?}) failed: '{err:#?}");
            }
        }
        Err(err) => {
            return error!("File::create({file_path:#?}) failed: '{err}'");
        }
    }
    Ok(())
}

pub fn read_file(file_path: &Path) -> Result<String, String> {
    match File::open(&file_path) {
        Ok(mut file) => {
            let mut content = String::new();
            if let Err(err) = file.read_to_string(&mut content) {
                return error!("read_to_string({file_path:#?}) failed: '{err}'");
            }
            Ok(content)
        }
        Err(err) => return error!("File::open({file_path:#?}) failed: '{err}'"),
    }
}

pub fn ls_regex(regex: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = read_dir(regex.parent().unwrap()) else {
        return files;
    };
    for entry in entries {
        let path = entry.expect("Failed to read entry").path();
        if regex == wildcardize_path(&path) {
            files.push(path);
        }
    }
    files
}

pub fn ls_dir(path: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(entries) = read_dir(path) else {
        return dirs;
    };
    for entry in entries {
        let dir = entry.expect("Failed to read entry");
        if !dir.file_type().unwrap().is_dir() {
            continue;
        }
        dirs.push(dir.path());
    }
    dirs
}
