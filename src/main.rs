extern crate touch;

mod filesystem;
mod parser;
mod project;
mod soongfile;
mod soongmodule;
mod target;
mod utils;

use crate::project::Project;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let number_common_arg = 4;
    if args.len() < number_common_arg {
        println!(
            "USAGE: {0} <project> <project_source_directory> <build_ninja_source_directory> [<project_arguments>...]",
            args[0]
        );
        return;
    }
    let project = &args[1];
    let project_source_directory = &args[2];
    let build_source_directory = &args[3];

    let targets = match parser::parse_build_ninja(&(build_source_directory)) {
        Ok(targets) => targets,
        Err(err) => {
            println!("Could not parse build.ninja: '{err}'");
            return;
        }
    };
    match if project == "spirvtools" {
        if args.len() < number_common_arg + 1 {
            println!("USAGE: {0} spirvtools <project_source_directory> <build_ninja_source_directory> <spirv_headers_directory>", args[0]);
            return;
        }
        let spirv_headers_directory = &args[number_common_arg];
        project::spirvtools::SpirvTools::new(
            &project_source_directory,
            &build_source_directory,
            spirv_headers_directory,
        )
        .generate(targets)
    } else if project == "llvm" {
        project::llvm::LLVM::new(&project_source_directory, &build_source_directory)
            .generate(targets)
    } else if project == "clspv" {
        project::clspv::CLSPV::new(&project_source_directory, &build_source_directory)
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
