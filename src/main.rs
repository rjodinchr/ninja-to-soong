// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, VecDeque};

mod ninja_target;
mod parser;
mod project;
mod soong_module;
mod soong_package;
mod utils;

use crate::project::*;
use crate::utils::*;

fn generate_project(
    project: &mut dyn Project,
    is_dependency: bool,
    projects_generated: &ProjectsMap,
) -> Result<(), String> {
    let project_name = project.get_id().str();
    if !is_dependency {
        print_info!(format!("Generating '{0}'", project_name));
    } else {
        print_info!(format!("Generating dependency '{0}'", project_name));
    }
    print_debug!("Get build dir...");
    let targets = if let Some(build_dir) = project.get_build_dir(projects_generated)? {
        print_debug!(format!("Parsing '{build_dir}/build.ninja'..."));
        parser::parse_build_ninja(build_dir)?
    } else {
        Vec::new()
    };
    print_debug!("Generating soong package...");
    let package = project.generate_package(targets, projects_generated)?;
    if !is_dependency {
        print_debug!("Writing soong file...");
        package.write(project_name)?;
    }
    Ok(())
}

fn generate_projects<'a>(
    all_projects: Vec<&'a mut dyn Project>,
    mut project_ids_to_write: VecDeque<ProjectId>,
) -> Result<(), String> {
    let write_all = project_ids_to_write.len() == 0;
    let mut projects_not_generated: HashMap<ProjectId, &mut dyn Project> = HashMap::new();
    for project in all_projects {
        if write_all {
            project_ids_to_write.push_back(project.get_id());
        }
        projects_not_generated.insert(project.get_id(), project);
    }
    let mut project_ids_to_generate = project_ids_to_write.clone();

    let mut projects_generated: ProjectsMap = HashMap::new();
    while let Some(project_id) = project_ids_to_generate.pop_front() {
        if projects_generated.contains_key(&project_id) {
            continue;
        }
        let project = projects_not_generated.remove(&project_id).unwrap();
        let mut missing_deps: Vec<ProjectId> = Vec::new();
        for dep in project.get_project_deps() {
            if !projects_generated.contains_key(&dep) {
                missing_deps.push(dep);
            }
        }
        if missing_deps.len() > 0 {
            let mut deps: Vec<&str> = Vec::new();
            project_ids_to_generate.push_front(project_id.clone());
            projects_not_generated.insert(project_id, project);
            for dep in missing_deps {
                deps.push(dep.str());
                project_ids_to_generate.push_front(dep);
            }
            continue;
        }
        generate_project(
            project,
            !project_ids_to_write.contains(&project_id),
            &projects_generated,
        )?;
        projects_generated.insert(project_id, project);
    }

    Ok(())
}

fn parse_args<'a>(
    executable: &str,
    args: &'a Vec<String>,
) -> Result<(&'a str, &'a str, VecDeque<ProjectId>), String> {
    let required_args = 3;
    if args.len() < required_args {
        return error!(format!(
            "USAGE: {executable} <android_dir> <{ANDROID_NDK}_dir> [<projects>]"
        ));
    }
    let android_dir = &args[1];
    let ndk_dir = &args[2];

    let ndk_name = ndk_dir.rsplit_once("/").unwrap().1;
    if ndk_name != ANDROID_NDK {
        print_warn!(format!("Expected '{ANDROID_NDK}' got '{ndk_name}'"));
    }

    let mut project_ids_to_write: VecDeque<ProjectId> = VecDeque::new();
    for arg in &args[required_args..] {
        match ProjectId::from(arg) {
            Some(project_id) => project_ids_to_write.push_back(project_id),
            None => return error!(format!("Unknown project '{arg}'")),
        };
    }
    Ok((android_dir, ndk_dir, project_ids_to_write))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let executable = args[0].rsplit_once("/").unwrap().1;
    let (android_dir, ndk_dir, project_ids_to_write) = parse_args(executable, &args)?;

    let temp_path = std::env::temp_dir().join(executable);
    let temp_dir = temp_path.to_str().unwrap();

    let mut clvk = clvk::Clvk::default();
    let mut clspv = clspv::Clspv::default();
    let mut llvm_project = llvm_project::LlvmProject::default();
    let mut spirv_tools = spirv_tools::SpirvTools::default();
    let mut spirv_headers = spirv_headers::SpirvHeaders::default();

    let mut all_projects: Vec<&mut dyn Project> = vec![
        &mut clvk,
        &mut clspv,
        &mut llvm_project,
        &mut spirv_tools,
        &mut spirv_headers,
    ];
    for project in all_projects.iter_mut() {
        project.init(android_dir, ndk_dir, temp_dir);
    }

    if let Err(err) = generate_projects(all_projects, project_ids_to_write) {
        print_error!(err);
        Err(format!("{executable} failed"))
    } else {
        Ok(())
    }
}
