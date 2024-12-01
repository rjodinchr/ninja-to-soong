// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

mod ninja_target;
mod parser;
mod project;
mod soong_module;
mod soong_package;
mod utils;

use crate::project::*;
use crate::utils::*;

fn get_project_id_to_write<'a>(
    project_id_to_generate: HashSet<ProjectId>,
    all_projects: &Vec<&'a mut dyn Project<'a>>,
) -> VecDeque<ProjectId> {
    let mut project_id_queue: VecDeque<ProjectId> = VecDeque::new();
    if project_id_to_generate.contains(&ProjectId::All) {
        for project in all_projects {
            project_id_queue.push_back(project.get_id());
        }
    } else {
        for project in project_id_to_generate {
            project_id_queue.push_back(project);
        }
    }
    project_id_queue
}

fn missing_deps(deps: &Vec<ProjectId>, projects_generated: &ProjectMap) -> bool {
    for dep in deps {
        if !projects_generated.contains_key(dep) {
            return true;
        }
    }
    false
}

fn generate_projects<'a>(
    all_projects: Vec<&'a mut dyn Project<'a>>,
    project_id_to_generate: HashSet<ProjectId>,
) -> Result<(), String> {
    let project_id_to_write = get_project_id_to_write(project_id_to_generate, &all_projects);
    let mut project_id_to_generate = project_id_to_write.clone();
    let mut project_not_generated: HashMap<ProjectId, &mut dyn Project> = HashMap::new();
    for project in all_projects {
        project_not_generated.insert(project.get_id(), project);
    }

    let mut projects_generated: ProjectMap = HashMap::new();
    while let Some(project_id) = project_id_to_generate.pop_front() {
        if projects_generated.contains_key(&project_id) {
            continue;
        }

        let project_str = project_id.str();
        let project_to_generate = project_id_to_write.contains(&project_id);
        if project_to_generate {
            print_info!(format!("Generating '{0}'", project_str));
        } else {
            print_info!(format!("Generating dependency '{0}'", project_str));
        }

        let project = project_not_generated.remove(&project_id).unwrap();
        let deps = project.get_project_dependencies();
        if missing_deps(&deps, &projects_generated) {
            project_not_generated.insert(project.get_id(), project);
            project_id_to_generate.push_front(project_id.clone());
            let mut deps_str: Vec<&str> = Vec::new();
            for dep in deps {
                deps_str.push(dep.str());
                project_id_to_generate.push_front(dep);
            }
            print_debug!(format!("Missing dependencies: {0}", deps_str.join(", ")));
            continue;
        }

        print_debug!("Get build directory...");
        let targets =
            if let Some(build_directory) = project.get_build_directory(&projects_generated)? {
                print_debug!(format!("Parsing '{build_directory}/build.ninja'..."));
                crate::parser::parse_build_ninja(build_directory)?
            } else {
                Vec::new()
            };
        print_debug!("Generating soong package...");
        let package = project.generate_package(targets, &projects_generated)?;
        if project_to_generate {
            print_debug!("Writing soong package...");
            package.write(project_str)?;
        }

        projects_generated.insert(project_id, project);
    }

    Ok(())
}

fn parse_args<'a>(
    args: &'a Vec<String>,
) -> Result<(&'a String, &'a String, HashSet<ProjectId>), String> {
    let min_args = 3;
    if args.len() < min_args {
        return error!(format!(
            "USAGE: {0} <android_dir> <{ANDROID_NDK}_dir> [<projects>]",
            args[0]
        ));
    }
    let android_dir = &args[1];
    let ndk_dir = &args[2];

    let ndk_name = ndk_dir.rsplit_once("/").unwrap().1;
    if ndk_name != ANDROID_NDK {
        print_warn!(format!("Expected '{ANDROID_NDK}' got '{ndk_name}'"));
    }

    let mut project_id_to_generate: HashSet<ProjectId> = HashSet::new();
    for arg in &args[min_args..] {
        match ProjectId::from(arg) {
            Some(project_id) => project_id_to_generate.insert(project_id),
            None => return error!(format!("Unknown project '{arg}'")),
        };
    }
    if project_id_to_generate.len() == 0 {
        project_id_to_generate.insert(ProjectId::All);
    }
    Ok((android_dir, ndk_dir, project_id_to_generate))
}

fn android_path(android_dir: &String, project: ProjectId) -> String {
    android_dir.clone() + "/external/" + project.str()
}

fn ninja_to_soong(args: &Vec<String>) -> Result<(), String> {
    let (android_dir, ndk_root, project_id_to_generate) = parse_args(args)?;

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

    generate_projects(all_projects, project_id_to_generate)
}

fn main() -> Result<(), String> {
    let args = std::env::args().collect();
    if let Err(err) = ninja_to_soong(&args) {
        print_error!(err);
        Err(format!(" '{0}' failed", args[0]))
    } else {
        Ok(())
    }
}
