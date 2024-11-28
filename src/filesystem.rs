use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use crate::macros::error;

pub fn write_file(path: &str, content: String) -> Result<String, String> {
    match File::create(path) {
        Ok(mut file) => {
            if let Err(err) = file.write_fmt(format_args!("{content}")) {
                return error!(format!("Could not write into '{path}': '{err:#?}"));
            }
        }
        Err(err) => {
            return error!(format!("Could not create '{path}': '{err}'"));
        }
    }
    return Ok(format!("'{path}' created successfully!"));
}

fn copy_file(file: &String, src: &str, dst: &str) -> Result<(), String> {
    let from = src.to_string() + file;
    let to = dst.to_string() + file;
    let to_dir = to.rsplit_once("/").unwrap().0;
    if let Err(err) = std::fs::create_dir_all(to_dir) {
        return error!(format!("create_dir_all({to_dir}) failed: {err}"));
    }
    if let Err(err) = std::fs::copy(&from, &to) {
        return error!(format!("copy({from}, {to}) failed: {err}"));
    }
    Ok(())
}

pub fn copy_files(
    files: HashSet<String>,
    src_root: &str,
    dst_root: &str,
) -> Result<String, String> {
    for file in files {
        if let Err(err) = copy_file(&file, src_root, dst_root) {
            return Err(err);
        }
    }
    return Ok(format!(
        "Files created successfully in '{dst_root}' from '{src_root}'"
    ));
}

pub fn touch_directories(directories: &HashSet<String>, dst_root: &str) -> Result<String, String> {
    for include_dir in directories {
        let dir = dst_root.to_string() + include_dir;
        if touch::exists(&dir) {
            continue;
        }
        if let Err(err) = std::fs::create_dir_all(&dir) {
            return error!(format!("create_dir_all({dir}) failed: {err}"));
        }
        if let Err(err) = touch::file::create(&(dir.clone() + "/touch"), false) {
            return error!(format!("touch in '{dir}' failed: {err}"));
        }
    }
    return Ok(format!("Touch include directories successfully!"));
}
