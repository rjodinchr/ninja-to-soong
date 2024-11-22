extern crate touch;

mod filesystem;
mod generator;
mod macros;
mod parser;
mod target;

fn main() {
    let src_root = "/usr/local/google/home/rjodin/work/clvk/";
    let build_root = "/usr/local/google/home/rjodin/work/clvk/build_android/";
    let ndk_root = "/usr/local/google/home/rjodin/work/android-ndk-r27c/";
    let dst_root = "/usr/local/google/home/rjodin/android-internal/external/clvk/";

    let (content, sources, mut generated_headers, mut include_directories) =
        match generator::generate(
            vec![String::from("libOpenCL.so")],
            &match parser::parse_build_ninja(&(build_root.to_string() + "build.ninja")) {
                Ok(targets) => targets,
                Err(err) => {
                    println!("Could not parse build.ninja: '{err}'");
                    return;
                }
            },
            src_root,
            ndk_root,
            build_root,
        ) {
            Ok(return_values) => return_values,
            Err(err) => {
                println!("generate for device failed: {err}");
                return;
            }
        };

    match filesystem::write_file(&(dst_root.to_string() + "Android.bp"), content) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }

    let dirs_to_remove = vec!["src", "external"];
    for dir_to_remove in dirs_to_remove {
        let dir = dst_root.to_string() + dir_to_remove;
        if touch::exists(&dir) {
            if let Err(err) = std::fs::remove_dir_all(&dir) {
                println!("remove_dir_all failed: {err}");
                return;
            }
        }
    }

    for source in &sources {
        include_directories.insert(source.rsplit_once("/").unwrap().0.to_string());
    }

    let missing_generated_headers = vec![
        "external/clspv/third_party/llvm/include/llvm/Config/llvm-config.h",
        "external/clspv/third_party/llvm/include/llvm/Config/abi-breaking.h",
        "external/clspv/third_party/llvm/include/llvm/Config/config.h",
        "external/clspv/third_party/llvm/include/llvm/Config/Targets.def",
        "external/clspv/third_party/llvm/include/llvm/Config/AsmPrinters.def",
        "external/clspv/third_party/llvm/include/llvm/Config/AsmParsers.def",
        "external/clspv/third_party/llvm/include/llvm/Config/Disassemblers.def",
        "external/clspv/third_party/llvm/include/llvm/Config/TargetMCAs.def",
        "external/clspv/third_party/llvm/include/llvm/Support/Extension.def",
        "external/clspv/third_party/llvm/tools/clang/include/clang/Basic/Version.inc",
        "external/clspv/third_party/llvm/tools/clang/include/clang/Config/config.h",
    ];
    for header in missing_generated_headers {
        generated_headers.insert(header.to_string());
    }
    match filesystem::copy_files(generated_headers, build_root, dst_root) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
    match filesystem::copy_files(sources, src_root, dst_root) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
    match filesystem::copy_include_directories(&include_directories, src_root, dst_root) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
    match filesystem::touch_include_directories(&include_directories, dst_root) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
}
