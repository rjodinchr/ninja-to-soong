use std::fs::File;
use std::io::Write;

mod generators;
mod macros;
mod parser;
mod target;

fn write_file(path: &str, content: String) {
    match File::create(path) {
        Ok(mut file) => match file.write_fmt(format_args!("{content}")) {
            Ok(_) => {
                println!("'{path}' created successfully!");
                return;
            }
            Err(err) => {
                println!("Could not write into '{path}': '{err:#?}");
                return;
            }
        },
        Err(err) => {
            println!("Could not create '{path}': '{err}'");
            return;
        }
    }
}

fn main() {
    let source_root = "/usr/local/google/home/rjodin/aluminium/external/angle/";
    let build_root =
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/build/";
    let device_native_lib_root =
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/android-ndk-r27c/";
    let host_native_lib_root = "/usr/lib/x86_64-linux-gnu/";

    let generator = generators::soong_generator::SoongGenerator();
    let input_ref_for_genrule = String::from("README.md");
    const HOST_PREFIX: &str = "external/clspv/third_party/llvm/NATIVE/";

    let android_host_bp = match generators::generate(
        &generator,
        vec![
            "bin/clang".to_string(),
            "bin/llvm-link".to_string(),
            "bin/llvm-as".to_string(),
            "bin/opt".to_string(),
            "bin/prepare_builtins".to_string(),
            "bin/llvm-min-tblgen".to_string(),
            "bin/llvm-tblgen".to_string(),
            "bin/clang-tblgen".to_string(),
        ],
        &match parser::parse_build_ninja(&(build_root.to_string() + HOST_PREFIX + "build.ninja")) {
            Ok(targets) => targets,
            Err(err) => {
                println!("Could not parse host build.ninja: '{err}'");
                return;
            }
        },
        source_root,
        host_native_lib_root,
        build_root,
        HOST_PREFIX,
        true,
        &input_ref_for_genrule,
    ) {
        Ok(result) => result,
        Err(err) => {
            println!("generate for host failed: {err}");
            return;
        }
    };

    let android_bp = match generators::generate(
        &generator,
        vec![String::from("libOpenCL.so")],
        &match parser::parse_build_ninja(&(build_root.to_string() + "build.ninja")) {
            Ok(targets) => targets,
            Err(err) => {
                println!("Could not parse build.ninja: '{err}'");
                return;
            }
        },
        source_root,
        device_native_lib_root,
        build_root,
        "",
        false,
        &input_ref_for_genrule,
    ) {
        Ok(result) => result + &android_host_bp,
        Err(err) => {
            println!("generate for device failed: {err}");
            return;
        }
    };

    write_file("Android.bp", android_bp);
}
