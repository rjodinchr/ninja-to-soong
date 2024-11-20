use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

mod generator;
mod macros;
mod parser;
mod target;

fn write_file(path: &str, content: &String) -> Result<String, String> {
    match File::create(path) {
        Ok(mut file) => match file.write_fmt(format_args!("{content}")) {
            Ok(_) => {
                return Ok(format!("'{path}' created successfully!"));
            }
            Err(err) => {
                return error!(format!("Could not write into '{path}': '{err:#?}"));
            }
        },
        Err(err) => {
            return error!(format!("Could not create '{path}': '{err}'"));
        }
    }
}

fn copy_files(
    files: &HashSet<String>,
    output_path: String,
    build_root: &str,
) -> Result<String, String> {
    let _ = std::fs::remove_dir_all(&output_path);
    for file in files {
        let from = build_root.to_string() + file;
        let to = output_path.clone() + file;
        println!("Copying {from} to {to}");
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

fn main() {
    let source_root = "/usr/local/google/home/rjodin/aluminium/external/angle/";
    let build_root =
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/build/";
    let native_lib_root =
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/android-ndk-r27c/";
    let cmake_build_files_root = "third_party/clvk/cmake_build_files/";

    let soong_file = match generator::generate(
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
        Ok(result) => result,
        Err(err) => {
            println!("generate for device failed: {err}");
            return;
        }
    };

    match write_file("Android.bp", soong_file.get_content()) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }

    match copy_files(
        soong_file.get_generated_headers(),
        source_root.to_string() + cmake_build_files_root,
        build_root,
    ) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
}
