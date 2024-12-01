// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet, VecDeque};

mod ninja_target;
mod parser;
mod project;
mod soong_module;
mod soong_package;
mod utils;

use crate::project::*;
use crate::utils::*;

fn get_project_ids_to_write<'a>(
    project_ids_to_generate: HashSet<ProjectId>,
    all_projects: &Vec<&'a mut dyn Project<'a>>,
) -> VecDeque<ProjectId> {
    let mut project_ids_queue: VecDeque<ProjectId> = VecDeque::new();
    if project_ids_to_generate.contains(&ProjectId::All) {
        for project in all_projects {
            project_ids_queue.push_back(project.get_id());
        }
    } else {
        for project in project_ids_to_generate {
            project_ids_queue.push_back(project);
        }
    }
    project_ids_queue
}

fn missing_deps(deps: &Vec<ProjectId>, projects_generated: &ProjectsMap) -> bool {
    for dep in deps {
        if !projects_generated.contains_key(dep) {
            return true;
        }
    }
    false
}

fn generate_projects<'a>(
    all_projects: Vec<&'a mut dyn Project<'a>>,
    project_ids_to_generate: HashSet<ProjectId>,
) -> Result<(), String> {
    let project_ids_to_write = get_project_ids_to_write(project_ids_to_generate, &all_projects);
    let mut project_ids_to_generate = project_ids_to_write.clone();
    let mut projects_not_generated: HashMap<ProjectId, &mut dyn Project> = HashMap::new();
    for project in all_projects {
        projects_not_generated.insert(project.get_id(), project);
    }

    let mut projects_generated: ProjectsMap = HashMap::new();
    while let Some(project_id) = project_ids_to_generate.pop_front() {
        if projects_generated.contains_key(&project_id) {
            continue;
        }

        let project_name = project_id.str();
        let is_dependency = !project_ids_to_write.contains(&project_id);
        if !is_dependency {
            print_info!(format!("Generating '{0}'", project_name));
        } else {
            print_info!(format!("Generating dependency '{0}'", project_name));
        }

        let project = projects_not_generated.remove(&project_id).unwrap();
        let project_deps = project.get_project_deps();
        if missing_deps(&project_deps, &projects_generated) {
            projects_not_generated.insert(project.get_id(), project);
            project_ids_to_generate.push_front(project_id.clone());
            let mut deps: Vec<&str> = Vec::new();
            for dep in project_deps {
                deps.push(dep.str());
                project_ids_to_generate.push_front(dep);
            }
            print_debug!(format!("Missing dependencies: {0}", deps.join(", ")));
            continue;
        }

        print_debug!("Get build dir...");
        let targets = if let Some(build_dir) = project.get_build_dir(&projects_generated)? {
            print_debug!(format!("Parsing '{build_dir}/build.ninja'..."));
            crate::parser::parse_build_ninja(build_dir)?
        } else {
            Vec::new()
        };
        print_debug!("Generating soong package...");
        let package = project.generate_package(targets, &projects_generated)?;
        if !is_dependency {
            print_debug!("Writing soong package...");
            package.write(project_name)?;
        }

        projects_generated.insert(project_id, project);
    }

    Ok(())
}

fn parse_args<'a>(
    args: &'a Vec<String>,
) -> Result<(&'a String, &'a String, HashSet<ProjectId>), String> {
    let required_args = 3;
    if args.len() < required_args {
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

    let mut project_ids_to_generate: HashSet<ProjectId> = HashSet::new();
    for arg in &args[required_args..] {
        match ProjectId::from(arg) {
            Some(project_id) => project_ids_to_generate.insert(project_id),
            None => return error!(format!("Unknown project '{arg}'")),
        };
    }
    if project_ids_to_generate.len() == 0 {
        project_ids_to_generate.insert(ProjectId::All);
    }
    Ok((android_dir, ndk_dir, project_ids_to_generate))
}

fn android_path(android_dir: &String, project: ProjectId) -> String {
    android_dir.clone() + "/external/" + project.str()
}

fn execute(executable: &str, args: &Vec<String>) -> Result<(), String> {
    let (android_dir, ndk_dir, project_id_to_generate) = parse_args(args)?;

    let temp_path = std::env::temp_dir().join(executable);
    let temp_dir = temp_path.to_str().unwrap();

    let clvk_dir = android_path(android_dir, ProjectId::Clvk);
    let clspv_dir = android_path(android_dir, ProjectId::Clspv);
    let llvm_project_dir = android_path(android_dir, ProjectId::LlvmProject);
    let spirv_tools_dir = android_path(android_dir, ProjectId::SpirvTools);
    let spirv_headers_dir = android_path(android_dir, ProjectId::SpirvHeaders);

    let mut spirv_tools = project::spirv_tools::SpirvTools::new(
        temp_dir,
        &ndk_dir,
        &spirv_tools_dir,
        &spirv_headers_dir,
    );
    let mut spirv_headers = project::spirv_headers::SpirvHeaders::new(&ndk_dir, &spirv_headers_dir);
    let mut llvm_project =
        project::llvm_project::LlvmProject::new(temp_dir, &ndk_dir, &llvm_project_dir);
    let mut clspv = project::clspv::Clspv::new(
        temp_dir,
        &ndk_dir,
        &clspv_dir,
        &llvm_project_dir,
        &spirv_tools_dir,
        &spirv_headers_dir,
    );
    let mut clvk = project::clvk::Clvk::new(
        temp_dir,
        &ndk_dir,
        &clvk_dir,
        &clspv_dir,
        &llvm_project_dir,
        &spirv_tools_dir,
        &spirv_headers_dir,
    );

    let mut all_projects: Vec<&mut dyn Project> = Vec::new();
    all_projects.push(&mut spirv_tools);
    all_projects.push(&mut spirv_headers);
    all_projects.push(&mut llvm_project);
    all_projects.push(&mut clspv);
    all_projects.push(&mut clvk);

    generate_projects(all_projects, project_id_to_generate)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let executable = args[0].rsplit_once("/").unwrap().1;
    if let Err(err) = execute(executable, &args) {
        print_error!(err);
        Err(format!("{0} failed", args[0]))
    } else {
        Ok(())
    }
}
