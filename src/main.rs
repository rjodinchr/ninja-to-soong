// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

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
        const ANDROID_BP: &str = "Android.bp";
        let file_path = project.get_test_path(ctx).join(ANDROID_BP);
        write_file(file_path.as_path(), &package)?;
        print_verbose!("{file_path:#?} created");

        if ctx.copy_to_aosp {
            let copy_dst = project.get_android_path(ctx).join(ANDROID_BP);
            copy_file(&file_path, &copy_dst)?;
            print_verbose!("{file_path:#?} copied to {copy_dst:#?}");
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

fn generate_projects(mut projects_map: ProjectsMap, ctx: &Context) -> Result<(), String> {
    let projects_to_write: HashSet<&ProjectId> = HashSet::from_iter(&ctx.projects_to_generate);
    let mut projects_to_generate = ctx.projects_to_generate.clone();
    let mut projects_generated = HashSet::new();
    while let Some(project_id) = projects_to_generate.pop_front() {
        if projects_generated.contains(&project_id) {
            continue;
        }
        let mut project = projects_map.remove(&project_id)?;
        let missing_deps = project_id
            .get_deps()
            .into_iter()
            .filter(|dep| !projects_generated.contains(dep));
        if missing_deps.clone().count() > 0 {
            projects_to_generate.extend(missing_deps);
            projects_to_generate.push_back(project_id);
        } else {
            generate_project(
                &mut project,
                projects_to_write.contains(&project_id),
                &projects_map,
                ctx,
            )?;
            projects_generated.insert(project_id);
        }
        projects_map.insert(project_id, project);
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let projects_map = ProjectsMap::new();
    match Context::parse_args(&projects_map) {
        Ok(ctx) => {
            if let Err(err) = generate_projects(projects_map, &ctx) {
                print_error!("{err}");
                return Err(String::from("generate_projects failed"));
            }
        }
        Err(err) => {
            print_error!("{err}");
            return Err(String::from("parse_args failed"));
        }
    }
    Ok(())
}
