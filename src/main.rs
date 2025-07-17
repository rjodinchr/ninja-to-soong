// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use libloading::Library;

mod context;
mod ninja_parser;
mod ninja_target;
mod project;
mod soong_module;
mod soong_module_generator;
mod soong_package;
mod soong_package_merger;
mod utils;

use crate::context::*;
use crate::project::*;
use crate::utils::*;

fn generate_project(
    project: &mut Box<dyn Project>,
    project_to_write: bool,
    projects_map: &ProjectsMap,
    ctx: &Context,
) -> Result<(), String> {
    let project_name = project.get_name();
    if project_to_write {
        print_info!("Generating '{project_name}'");

        print_debug!("Creating soong package...");
        let package = project.generate_package(ctx, projects_map)?;

        print_debug!("Writing soong file...");
        let file_path = if !ctx.copy_to_aosp {
            project.get_test_path(ctx)?.join("Android.bp.n2s")
        } else {
            project.get_android_path(ctx)?.join("Android.bp")
        };
        let Ok(current_package) = read_file(&file_path) else {
            write_file(&file_path, &package)?;
            print_verbose!("{file_path:#?} created");
            return Ok(());
        };
        if current_package != package {
            write_file(&file_path, &package)?;
            print_verbose!("{file_path:#?} updated");
        } else {
            print_verbose!("{file_path:#?} unmodified");
        }
    } else {
        print_info!("Generating dependency '{project_name}'");
        let mut dep_ctx = ctx.clone();
        dep_ctx.copy_to_aosp = false;
        dep_ctx.skip_build = true;
        project.generate_package(&dep_ctx, projects_map)?;
    }
    Ok(())
}

fn get_library(ctx: &Context) -> Result<Library, String> {
    let path = ctx.get_external_project_path()?;
    if !path.exists() {
        return error!("external project path ({path:#?} does not exist");
    }
    let library_path = path_to_string(ctx.temp_path.join("external_project.so"));
    execute_cmd!(
        "rustc",
        [
            "--crate-type=cdylib",
            "-L",
            &path_to_string(&ctx.exe_path),
            "-lninja_to_soong",
            "-o",
            &library_path,
            &path_to_string(&path),
        ]
    )?;
    match unsafe { Library::new(&library_path) } {
        Ok(lib) => Ok(lib),
        Err(_) => error!("Could not create load {library_path:#?}"),
    }
}

fn generate_projects(mut projects_map: ProjectsMap, ctx: &Context) -> Result<(), String> {
    let projects_to_write: HashSet<&ProjectId> = HashSet::from_iter(&ctx.projects_to_generate);
    let mut projects_to_generate = ctx.projects_to_generate.clone();
    let mut projects_generated = HashSet::new();
    while let Some(project_id) = projects_to_generate.pop_front() {
        if projects_generated.contains(&project_id) {
            continue;
        }
        let missing_deps = project_id
            .get_deps()
            .into_iter()
            .filter(|dep| !projects_generated.contains(dep));
        if missing_deps.clone().count() > 0 {
            projects_to_generate.extend(missing_deps);
            projects_to_generate.push_back(project_id);
            continue;
        }
        match project_id {
            ProjectId::External => {
                let mut project_ctx = ctx.clone();
                project_ctx.wildcardize_paths = true;

                let library = get_library(&project_ctx)?;
                const GET_PROJECT_SYMBOL: &str = "get_project";
                let mut project = match unsafe {
                    library.get::<fn() -> Box<dyn Project>>(GET_PROJECT_SYMBOL.as_bytes())
                } {
                    Ok(get_project) => get_project(),
                    Err(_) => return error!("Could not get symbol '{GET_PROJECT_SYMBOL}'"),
                };
                generate_project(&mut project, true, &projects_map, &project_ctx)?;
            }
            ProjectId::UnitTest => {
                let mut project = projects_map.remove(&project_id)?;
                for dir in ls_dir(&ctx.test_path.join("unittests")) {
                    let mut test_ctx = ctx.clone();
                    test_ctx.test_path = dir;
                    test_ctx.wildcardize_paths = true;
                    generate_project(&mut project, true, &projects_map, &test_ctx)?;
                }
                projects_map.insert(project_id, project);
            }
            _ => {
                let mut project = projects_map.remove(&project_id)?;
                generate_project(
                    &mut project,
                    projects_to_write.contains(&project_id),
                    &projects_map,
                    ctx,
                )?;
                projects_map.insert(project_id, project);
            }
        }
        projects_generated.insert(project_id);
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let projects_map = ProjectsMap::new();
    match Context::parse_args(&projects_map) {
        Ok(ctx) => {
            if let Err(err) = generate_projects(projects_map, &ctx) {
                print_error!("{err}");
                return error!("generate_projects failed");
            }
        }
        Err(err) => {
            print_error!("{err}");
            return error!("parse_args failed");
        }
    }
    Ok(())
}
