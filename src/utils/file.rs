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

pub fn ls_regex(regex: &Path) -> Result<Vec<PathBuf>, String> {
    let Some(parent) = regex.parent() else {
        return error!("invalid regex {regex:#?}");
    };
    let Ok(entries) = read_dir(parent) else {
        return Ok(Vec::new());
    };
    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let Ok(entry) = entry else {
                return None;
            };
            let path = entry.path();
            if regex != wildcardize_path(&path) {
                return None;
            }
            Some(path)
        })
        .collect())
}

pub fn ls_dir(path: &Path) -> Result<Vec<PathBuf>, String> {
    let Ok(entries) = read_dir(path) else {
        return Ok(Vec::new());
    };
    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let Ok(dir) = entry else {
                return None;
            };
            let path = dir.path();
            if !path.is_dir() {
                return None;
            }
            Some(path)
        })
        .collect())
}
