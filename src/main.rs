extern crate touch;

mod filesystem;
mod generator;
mod macros;
mod parser;
mod soongmodule;
mod target;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        println!(
            "USAGE: {0} <src_root> <build_root> <ndk_root> <dst_root>",
            args[0]
        );
        return;
    }

    let src_root = args[1].as_str();
    let build_root = args[2].as_str();
    let ndk_root = args[3].as_str();
    let dst_root = args[4].as_str();
    let dst_build_prefix = "cmake_generated/";

    let input_ref_for_genrule = "README.md";

    let (content, mut generated_headers, include_directories) = match generator::generate(
        vec!["libOpenCL.so"],
        &match parser::parse_build_ninja(&(build_root.to_string())) {
            Ok(targets) => targets,
            Err(err) => {
                println!("Could not parse build.ninja: '{err}'");
                return;
            }
        },
        src_root,
        ndk_root,
        build_root,
        "clvk_",
        input_ref_for_genrule,
        dst_build_prefix,
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

    let dirs_to_remove = vec![dst_build_prefix];
    for dir_to_remove in dirs_to_remove {
        let dir = src_root.to_string() + dir_to_remove;
        if touch::exists(&dir) {
            if let Err(err) = std::fs::remove_dir_all(&dir) {
                println!("remove_dir_all failed: {err}");
                return;
            }
        }
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
        "external/clspv/third_party/llvm/include/llvm/Support/VCSRevision.h",
        "external/clspv/third_party/llvm/tools/clang/lib/Basic/VCSVersion.inc",
        "external/clspv/third_party/llvm/tools/clang/include/clang/Basic/Version.inc",
        "external/clspv/third_party/llvm/tools/clang/include/clang/Config/config.h",
    ];
    for header in missing_generated_headers {
        generated_headers.insert(header.to_string());
    }
    match filesystem::copy_files(
        generated_headers,
        build_root,
        &(src_root.to_string() + dst_build_prefix),
    ) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
    match filesystem::touch_directories(&include_directories, src_root) {
        Ok(msg) => println!("{msg}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
}
