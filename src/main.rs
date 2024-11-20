extern crate touch;

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

mod generator;
mod macros;
mod parser;
mod target;

fn write_file(path: &str, content: String) -> Result<String, String> {
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

fn copy_files(
    files: HashSet<String>,
    output_path: String,
    build_root: &str,
) -> Result<String, String> {
    for file in files {
        let from = build_root.to_string() + &file;
        let to = output_path.clone() + &file;
        //println!("Copying {from} to {to}");
        let to_dir = to.rsplit_once("/").unwrap().0;
        if let Err(err) = std::fs::create_dir_all(to_dir) {
            return error!(format!("create_dir_all({to_dir}) failed: {err}"));
        }
        if let Err(err) = std::fs::copy(&from, &to) {
            return error!(format!("copy({from}, {to}) failed: {err}"));
        }
    }
    return Ok(format!("Files created successfully in '{output_path}'"));
}

fn create_directories(directories: HashSet<String>, output_path: String) -> Result<String, String> {
    for directory in directories {
        let directory_full_path = output_path.clone() + &directory;
        if let Err(err) = std::fs::create_dir_all(&directory_full_path) {
            return error!(format!("create_dir_all({directory}) failed: {err}"));
        }
        if let Err(err) = touch::file::create(&(directory_full_path + "/dummy"), false) {
            return error!(format!("touch in '{directory}' failed: {err}"));
        }
    }
    return Ok(format!("Directories created successfully!"));
}

fn main() {
    let source_root = "/usr/local/google/home/rjodin/aluminium/external/angle/";
    let build_root =
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/build/";
    let native_lib_root =
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/android-ndk-r27c/";
    let cmake_build_files_root = "third_party/clvk/cmake_build_files/";

    let (content, mut generated_headers, generated_directories) = match generator::generate(
        vec![String::from("libOpenCL.so")],
        &match parser::parse_build_ninja(&(build_root.to_string() + "build.ninja")) {
            Ok(targets) => targets,
            Err(err) => {
                println!("Could not parse build.ninja: '{err}'");
                return;
            }
        },
        source_root,
        native_lib_root,
        build_root,
        cmake_build_files_root,
    ) {
        Ok(return_value) => return_value,
        Err(err) => {
            println!("generate for device failed: {err}");
            return;
        }
    }
    .finish();

    match write_file("Android.bp", content) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }

    let cmake_build_files_full_path = source_root.to_string() + cmake_build_files_root;
    if touch::exists(&cmake_build_files_full_path) {
        if let Err(err) = std::fs::remove_dir_all(&cmake_build_files_full_path) {
            println!("remove_dir_all failed: {err}");
            return;
        }
    }

    match create_directories(generated_directories, source_root.to_string()) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }

    let other_generated_headers = vec![
        "external/clspv/third_party/llvm/include/llvm/Config/llvm-config.h",
        "external/clspv/third_party/llvm/include/llvm/Config/abi-breaking.h",
        "external/clspv/third_party/llvm/include/llvm/Config/config.h",
        "external/clspv/third_party/llvm/include/llvm/Config/Targets.def",
        "external/clspv/third_party/llvm/include/llvm/Config/AsmPrinters.def",
        "external/clspv/third_party/llvm/include/llvm/Config/AsmParsers.def",
        "external/clspv/third_party/llvm/include/llvm/Support/Extension.def",
        "external/clspv/third_party/llvm/tools/clang/include/clang/Basic/Version.inc",
        "external/clspv/third_party/llvm/tools/clang/include/clang/Config/config.h",
    ];
    for header in other_generated_headers {
        generated_headers.insert(header.to_string());
    }
    match copy_files(generated_headers, cmake_build_files_full_path, build_root) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
}
