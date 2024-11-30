// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::VecDeque;

mod ninja_target;
mod parser;
mod project;
mod soong_module;
mod soong_package;
mod utils;

use crate::project::Project;
use crate::utils::*;

fn generate_projects(
    all_projects: Vec<&mut dyn Project>,
    projects: &[String],
) -> Result<(), String> {
    let mut projects_map: HashMap<ProjectId, &mut dyn Project> = HashMap::new();
    for project in all_projects {
        projects_map.insert(project.get_id(), project);
    }

    let mut projects_to_generate: VecDeque<ProjectId> = VecDeque::new();
    for arg in projects {
        if arg == "all" {
            for project in projects_map.keys() {
                projects_to_generate.push_back(project.clone());
            }
            continue;
        }
        match ProjectId::from(arg) {
            Some(id) => projects_to_generate.push_back(id),
            None => return Err(format!("unknown project '{arg}'")),
        };
    }
    let projects_to_write = projects_to_generate.clone();

    let mut projects_generated: HashMap<ProjectId, &dyn Project> = HashMap::new();
    while let Some(project_id) = projects_to_generate.pop_front() {
        if projects_generated.contains_key(&project_id) {
            continue;
        }

        let project_str = project_id.str();
        let generating = projects_to_write.contains(&project_id);
        println!("\n############## {BANNER} ##############");
        if generating {
            println!("{BANNER} Generating '{0}'", project_str);
        } else {
            println!("{BANNER} Generating dependency '{0}'", project_str);
        }

        let project = projects_map.remove(&project_id).unwrap();
        let deps = project.get_deps();
        fn missing_deps(
            deps: &Vec<ProjectId>,
            projects_generated: &HashMap<ProjectId, &dyn Project>,
        ) -> bool {
            for dep in deps {
                if !projects_generated.contains_key(dep) {
                    return true;
                }
            }
            return false;
        }
        if missing_deps(&deps, &projects_generated) {
            projects_map.insert(project.get_id(), project);
            projects_to_generate.push_front(project_id.clone());
            let mut deps_str: Vec<&str> = Vec::new();
            for dep in deps {
                deps_str.push(dep.str());
                projects_to_generate.push_front(dep);
            }
            println!("{BANNER} missing dependencies: {0}", deps_str.join(", "));
            continue;
        }

        println!("{BANNER} \tget build directory...");
        let build_directory = project.get_build_directory(&projects_generated)?;
        println!("{BANNER} \tparsing build.ninja...");
        let targets = crate::parser::parse_build_ninja(build_directory)?;
        println!("{BANNER} \tprocessing...");
        let package = project.generate_package(targets, &projects_generated)?;
        if projects_to_write.contains(&project_id) {
            println!("{BANNER} \tgenerating soong package...");
            package.write(project_str)?;
        }

        projects_generated.insert(project_id, project);
    }

    Ok(())
}

fn parse_args(args: &Vec<String>) -> Result<(&String, &String, &[String]), String> {
    let min_args = 4;
    if args.len() < min_args {
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

    return Ok((android_dir, ndk_dir, &args[min_args - 1..]));
}

fn android_path(android_dir: &String, project: ProjectId) -> String {
    return android_dir.clone() + "/external/" + project.str();
}

fn main() -> Result<(), String> {
    let args = std::env::args().collect();
    let (android_dir, ndk_root, projects) = parse_args(&args)?;

    let temp_path = std::env::temp_dir().join("ninja-to-soong");
    let temp_dir = temp_path.to_str().unwrap();

    let clvk_root = android_path(android_dir, ProjectId::CLVK);
    let clspv_root = android_path(android_dir, ProjectId::CLSPV);
    let llvm_project_root = android_path(android_dir, ProjectId::LLVM);
    let spirv_tools_root = android_path(android_dir, ProjectId::SpirvTools);
    let spirv_headers_root = android_path(android_dir, ProjectId::SpirvHeaders);

    
    let mut spirv_tools = project::spirv_tools::SpirvTools::new(
        temp_dir,
        &ndk_root,
        &spirv_tools_root,
        &spirv_headers_root,
    );
    let mut spirv_headers =
        project::spirv_headers::SpirvHeaders::new(&ndk_root, &spirv_headers_root);
    let mut llvm = project::llvm::LLVM::new(temp_dir, &ndk_root, &llvm_project_root);
    let mut clspv = project::clspv::CLSPV::new(
        temp_dir,
        &ndk_root,
        &clspv_root,
        &llvm_project_root,
        &spirv_tools_root,
        &spirv_headers_root,
    );
    let mut clvk = project::clvk::CLVK::new(
        temp_dir,
        &ndk_root,
        &clvk_root,
        &clspv_root,
        &llvm_project_root,
        &spirv_tools_root,
        &spirv_headers_root,
    );

    let mut all_projects: Vec<&mut dyn Project> = Vec::new();
    all_projects.push(&mut spirv_tools);
    all_projects.push(&mut spirv_headers);
    all_projects.push(&mut llvm);
    all_projects.push(&mut clspv);
    all_projects.push(&mut clvk);

    generate_projects(all_projects, projects)
}
