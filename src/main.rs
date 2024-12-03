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
    project_ids: Vec<ProjectId>,
) -> Result<(), String> {
    let mut projects_not_generated: HashMap<ProjectId, &mut dyn Project> = HashMap::new();
    let mut project_ids_to_generate: VecDeque<ProjectId> = VecDeque::new();
    for project in all_projects {
        if project_ids.len() == 0 {
            project_ids_to_generate.push_back(project.get_id());
        }
        projects_not_generated.insert(project.get_id(), project);
    }
    project_ids_to_generate.extend(project_ids);
    let project_ids_to_write = project_ids_to_generate.clone();

    let mut projects_generated: ProjectsMap = HashMap::new();
    while let Some(project_id) = project_ids_to_generate.pop_front().as_ref() {
        if projects_generated.contains_key(project_id) {
            continue;
        }
        let project = projects_not_generated.remove(project_id).unwrap();
        let missing_deps: Vec<ProjectId> = project
            .get_project_deps()
            .into_iter()
            .filter(|dep| !projects_generated.contains_key(&dep))
            .collect();
        if missing_deps.len() > 0 {
            project_ids_to_generate.extend(missing_deps);
            project_ids_to_generate.push_back(project.get_id());
            projects_not_generated.insert(project.get_id(), project);
            continue;
        }
        generate_project(
            project,
            !project_ids_to_write.contains(project_id),
            &projects_generated,
        )?;
        projects_generated.insert(project.get_id(), project);
    }

    Ok(())
}

fn parse_args(
    executable: &str,
    args: Vec<String>,
) -> Result<(String, String, Vec<ProjectId>), String> {
    let required_args = 3;
    if args.len() < required_args {
        return error!(format!(
            "USAGE: {executable} <android_dir> <{ANDROID_NDK}_dir> [<projects>]"
        ));
    }
    let android_dir = args[1].clone();
    let ndk_dir = args[2].clone();

    let ndk_name = ndk_dir.rsplit_once("/").unwrap().1;
    if ndk_name != ANDROID_NDK {
        print_warn!(format!("Expected '{ANDROID_NDK}' got '{ndk_name}'"));
    }

    let mut project_ids: Vec<ProjectId> = Vec::new();
    for arg in &args[required_args..] {
        match ProjectId::from(arg) {
            Some(project_id) => project_ids.push(project_id),
            None => return error!(format!("Unknown project '{arg}'")),
        };
    }
    Ok((android_dir, ndk_dir, project_ids))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let executable = args[0].clone().rsplit_once("/").unwrap().1.to_string();
    let (android_dir, ndk_dir, project_ids) = parse_args(&executable, args)?;

    let temp_path = std::env::temp_dir().join(&executable);
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
        project.init(&android_dir, &ndk_dir, temp_dir);
    }

    if let Err(err) = generate_projects(all_projects, project_ids) {
        print_error!(err);
        Err(format!("{executable} failed"))
    } else {
        Ok(())
    }
}
