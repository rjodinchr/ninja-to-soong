use std::collections::HashSet;
use std::ffi::OsString;
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
    return Ok(format!("Files created successfully in '{dst_root}'"));
}

fn header_to_copy(name: OsString) -> bool {
    let name = name.to_str().unwrap();
    return name.ends_with(".h") || name.ends_with(".hpp") || name.ends_with(".def");
}

fn copy_headers_from(
    include_dir: &String,
    src_root: &str,
    dst_root: &str,
) -> Result<(), String> {
    let directory_path = src_root.to_string() + include_dir;
    let Ok(entries) = std::fs::read_dir(&directory_path) else {
        return Ok(());
    };
    for entry in entries {
        let Ok(item) = entry else {
            return error!(format!("Could not read entry in {directory_path}"));
        };
        let Ok(file_type) = item.file_type() else {
            return error!(format!("Could not get file type for {item:#?}"));
        };
        if file_type.is_dir() {
            let dir = include_dir.clone() + "/" + item.file_name().to_str().unwrap();
            if let Err(err) = copy_headers_from(&dir, src_root, dst_root) {
                return Err(err);
            }
        } else if file_type.is_file() && header_to_copy(item.file_name()) {
            if let Err(err) = copy_file(
                &(item.file_name().to_str().unwrap().to_string()),
                &(item
                    .path()
                    .to_str()
                    .unwrap()
                    .rsplit_once("/")
                    .unwrap()
                    .0
                    .to_string()
                    + "/"),
                &(dst_root.to_string() + include_dir + "/"),
            ) {
                return Err(err);
            }
        }
    }
    return Ok(());
}

pub fn copy_include_directories(
    include_directories: &HashSet<String>,
    src_root: &str,
    dst_root: &str,
) -> Result<String, String> {
    for include_dir in include_directories {
        if let Err(err) = copy_headers_from(&include_dir, src_root, dst_root) {
            return Err(err);
        }
    }
    return Ok(format!("Include directories created successfully!"));
}

pub fn touch_include_directories(
    include_directories: &HashSet<String>,
    dst_root: &str,
) -> Result<String, String> {
    for include_dir in include_directories {
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
