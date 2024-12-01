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

use crate::project::*;
use crate::utils::*;

const ALL_TARGETS: &str = "all";

fn generate_projects<'a>(
    all_projects: Vec<&'a mut dyn Project<'a>>,
    projects_string_to_generate: &[String],
) -> Result<(), String> {
    let mut projects_map: HashMap<ProjectId, &mut dyn Project> = HashMap::new();
    for project in all_projects {
        projects_map.insert(project.get_id(), project);
    }

    let mut projects_queue: VecDeque<ProjectId> = VecDeque::new();
    for arg in projects_string_to_generate {
        if arg == ALL_TARGETS {
            for project in projects_map.keys() {
                projects_queue.push_back(project.clone());
            }
            continue;
        }
        match ProjectId::from(arg) {
            Some(id) => projects_queue.push_back(id),
            None => return error!(format!("unknown project '{arg}'")),
        };
    }
    let projects_to_generate = projects_queue.clone();

    let mut projects_generated: ProjectMap = HashMap::new();
    while let Some(project_id) = projects_queue.pop_front() {
        if projects_generated.contains_key(&project_id) {
            continue;
        }

        let project_str = project_id.str();
        let project_to_generate = projects_to_generate.contains(&project_id);
        println!("\n############## {BANNER} ##############");
        if project_to_generate {
            println!("{BANNER} Generating '{0}'", project_str);
        } else {
            println!("{BANNER} Generating dependency '{0}'", project_str);
        }

        let project = projects_map.remove(&project_id).unwrap();
        let deps = project.get_project_dependencies();
        fn missing_deps(deps: &Vec<ProjectId>, projects_generated: &ProjectMap) -> bool {
            for dep in deps {
                if !projects_generated.contains_key(dep) {
                    return true;
                }
            }
            return false;
        }
        if missing_deps(&deps, &projects_generated) {
            projects_map.insert(project.get_id(), project);
            projects_queue.push_front(project_id.clone());
            let mut deps_str: Vec<&str> = Vec::new();
            for dep in deps {
                deps_str.push(dep.str());
                projects_queue.push_front(dep);
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
        if project_to_generate {
            println!("{BANNER} \tgenerating soong package...");
            package.write(project_str)?;
        }

        projects_generated.insert(project_id, project);
    }

    Ok(())
}

fn parse_args<'a>(
    args: &'a Vec<String>,
    all_targets: &'a Vec<String>,
) -> Result<(&'a String, &'a String, &'a [String]), String> {
    let min_args = 3;
    if args.len() < min_args {
        return error!(format!(
            "USAGE: {0} <android_dir> <{ANDROID_NDK}_dir> [<projects>]",
            args[0]
        ));
    }
    let android_dir = &args[1];
    let ndk_dir = &args[2];

    if ndk_dir.rsplit_once("/").unwrap().1 != ANDROID_NDK {
        println!("\x1b[00;31m");
        println!("########");
        println!("WARNING: ninja-to-soong expect to use '{ANDROID_NDK}', which does not seem to be the ndk provided");
        println!("########");
        println!("\x1b[0m");
    }

    if args.len() == min_args {
        return Ok((android_dir, ndk_dir, &all_targets[0..]));
    } else {
        return Ok((android_dir, ndk_dir, &args[min_args..]));
    }
}

fn android_path(android_dir: &String, project: ProjectId) -> String {
    return android_dir.clone() + "/external/" + project.str();
}

fn main() -> Result<(), String> {
    let all_targets = vec![ALL_TARGETS.to_string()];
    let args = std::env::args().collect();
    let (android_dir, ndk_root, projects_to_generate) = parse_args(&args, &all_targets)?;

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

    generate_projects(all_projects, projects_to_generate)
}
