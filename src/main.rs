extern crate touch;

mod filesystem;
mod parser;
mod project;
mod soongmodule;
mod soongpackage;
mod target;
mod utils;

use crate::project::Project;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let number_common_arg = 5;
    if args.len() < number_common_arg {
        println!(
            "USAGE: {0} <project> <ndk_directory> <project_source_directory> <build_ninja_source_directory> [<project_arguments>...]",
            args[0]
        );
        return;
    }
    let project = &args[1];
    let ndk_directory = &args[2];
    let project_source_directory = &args[3];
    let build_source_directory = &args[4];

    let targets = match parser::parse_build_ninja(&(build_source_directory)) {
        Ok(targets) => targets,
        Err(err) => {
            println!("Could not parse build.ninja: '{err}'");
            return;
        }
    };
    match if project == "spirvtools" {
        if args.len() < number_common_arg + 1 {
            println!("USAGE: {0} spirvtools <ndk_directory> <project_source_directory> <build_ninja_source_directory> <spirv_headers_directory>", args[0]);
            return;
        }
        let spirv_headers_directory = &args[number_common_arg];
        project::spirvtools::SpirvTools::new(
            &project_source_directory,
            &build_source_directory,
            &ndk_directory,
            spirv_headers_directory,
        )
        .generate(targets)
    } else if project == "spirvheaders" {
        if args.len() < number_common_arg + 1 {
            println!("USAGE: {0} spirvtools <ndk_directory> <project_source_directory> <build_ninja_source_directory> <spirv_tools_directory>", args[0]);
            return;
        }
        let spirv_tools_directory = &args[number_common_arg];
        project::spirvheaders::SpirvHeaders::new(
            &project_source_directory,
            &build_source_directory,
            &ndk_directory,
            spirv_tools_directory,
        )
        .generate(targets)
    } else if project == "llvm" {
        project::llvm::LLVM::new(
            &project_source_directory,
            &build_source_directory,
            &ndk_directory,
        )
        .generate(targets)
    } else if project == "clspv" {
        if args.len() < number_common_arg + 2 {
            println!("USAGE: {0} clspv <ndk_directory> <project_source_directory> <build_ninja_source_directory> <spirv_headers_directory> <llvm_project_directory>", args[0]);
            return;
        }
        let spirv_headers_directory = &args[number_common_arg];
        let llvm_project_directory = &args[number_common_arg + 1];
        project::clspv::CLSPV::new(
            &project_source_directory,
            &build_source_directory,
            &ndk_directory,
            spirv_headers_directory,
            llvm_project_directory,
        )
        .generate(targets)
    } else if project == "clvk" {
        project::clvk::CLVK::new(
            &project_source_directory,
            &build_source_directory,
            &ndk_directory,
        )
        .generate(targets)
    } else {
        println!("unknown project '{project}'");
        return;
    } {
        Ok(message) => println!("{message}"),
        Err(err) => {
            println!("{err}");
            return;
        }
    }
}
