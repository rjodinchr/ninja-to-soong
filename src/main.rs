mod ninja_target;
mod parser;
mod project;
mod soong_module;
mod soong_package;
mod utils;

use crate::project::Project;
use crate::utils::*;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let number_common_arg = 4;
    if args.len() < number_common_arg {
        return Err(format!(
            "USAGE: {0} <android_dir> <ndk_r27c_dir> [<projects>]",
            args[0]
        ));
    }
    let android_dir = &args[1];
    let ndk_dir = &args[2];

    if ndk_dir.rsplit_once("/").unwrap().1 != "android-ndk-r27c" {
        println!("WARN: ninja-to-soong expect to use 'android-ndk-r27c', which does not seem to be the ndk provided");
    }

    let temp_path = std::env::temp_dir().join("ninja-to-soong");
    let temp_dir = temp_path.to_str().unwrap();

    let clvk_root = android_dir.clone() + "/external/clvk";
    let clspv_root = android_dir.clone() + "/external/clspv";
    let llvm_root = android_dir.clone() + "/external/llvm-project";
    let spirv_tools_root = android_dir.clone() + "/external/SPIRV-Tools";
    let spirv_headers_root = android_dir.clone() + "/external/SPIRV-Headers";

    let spirv_tools = project::spirv_tools::SpirvTools::new(
        temp_dir,
        &ndk_dir,
        &spirv_tools_root,
        &spirv_headers_root,
    );
    let spirv_headers =
        project::spirv_headers::SpirvHeaders::new(&ndk_dir, &spirv_headers_root, &spirv_tools);
    let llvm = project::llvm::LLVM::new(temp_dir, &ndk_dir, &llvm_root);
    let clspv = project::clspv::CLSPV::new(
        temp_dir,
        &ndk_dir,
        &clspv_root,
        &llvm_root,
        &spirv_tools_root,
        &spirv_headers_root,
    );
    let clvk = project::clvk::CLVK::new(
        temp_dir,
        &ndk_dir,
        &clvk_root,
        &clspv_root,
        &llvm_root,
        &spirv_tools_root,
        &spirv_headers_root,
    );

    let all_projects: Vec<&dyn Project> = vec![&spirv_tools, &spirv_headers, &llvm, &clspv, &clvk];

    let first_project_index = number_common_arg - 1;
    for project in all_projects {
        if args[first_project_index] == "all"
            || args[first_project_index..].contains(&project.get_name().to_string())
        {
            println!("\n############## {PRINT_BANNER} ##############");
            println!("{PRINT_BANNER} Generating '{0}'", project.get_name());
            println!("{PRINT_BANNER} \tget build directory...");
            let build_directory = project.get_build_directory()?;
            println!("{PRINT_BANNER} \tparsing build.ninja...");
            let targets = crate::parser::parse_build_ninja(build_directory)?;
            println!("{PRINT_BANNER} \tgenerating soong package...");
            println!("{PRINT_BANNER} \t\t{0}", project.generate(targets)?);
        }
    }
    Ok(())
}
