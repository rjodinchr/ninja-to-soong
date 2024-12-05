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
    android_path: &Path,
) -> Result<(), String> {
    let project_name = project.get_id().str();
    let project_path = project.get_id().android_path(android_path);
    if !is_dependency {
        print_info!("Generating '{project_name}'");
    } else {
        print_info!("Generating dependency '{project_name}'");
    }
    print_debug!("Creating soong package...");
    let package = project.generate_package(projects_generated)?;
    if !is_dependency {
        print_debug!("Writing soong file...");

        const ANDROID_BP: &str = "Android.bp";
        let file_path = project_path.join(ANDROID_BP);
        write_file(file_path.as_path(), &package.print())?;
        print_verbose!("{file_path:#?} created");

        let copy_dst = get_tests_folder()?.join(project_name).join(ANDROID_BP);
        copy_file(&file_path, &copy_dst)?;
        print_verbose!("{file_path:#?} copied to {copy_dst:#?}");
    }
    Ok(())
}

fn generate_projects<'a>(
    all_projects: Vec<&'a mut dyn Project>,
    project_ids: Vec<ProjectId>,
    android_path: &Path,
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
            android_path,
        )?;
        projects_generated.insert(project.get_id(), project);
    }

    Ok(())
}

fn parse_args(
    executable: &str,
    args: Vec<String>,
) -> Result<(PathBuf, PathBuf, Vec<ProjectId>), String> {
    let required_args = 3;
    if args.len() < required_args {
        return error!("USAGE: {executable} <android_dir> <{ANDROID_NDK}_dir> [<projects>]");
    }
    let android_path = PathBuf::from(args[1].clone());
    let ndk_path = PathBuf::from(args[2].clone());

    let ndk_name = file_name(&ndk_path);
    if ndk_name != ANDROID_NDK {
        print_warn!("Expected '{ANDROID_NDK}' got '{ndk_name}'");
    }

    let mut project_ids: Vec<ProjectId> = Vec::new();
    for arg in &args[required_args..] {
        project_ids.push(ProjectId::from(arg)?);
    }
    Ok((android_path, ndk_path, project_ids))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let executable_path = PathBuf::from(&args[0]);
    let executable = file_name(&executable_path);
    let (android_path, ndk_path, project_ids) = parse_args(&executable, args)?;
    let temp_path = std::env::temp_dir().join(&executable);

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
        project.init(&android_path, &ndk_path, &temp_path);
    }

    if let Err(err) = generate_projects(all_projects, project_ids, &android_path) {
        print_error!("{err}");
        Err(format!("{executable} failed"))
    } else {
        Ok(())
    }
}
