// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashSet, VecDeque};

mod context;
mod ninja_target;
mod parser;
mod project;
mod soong_module;
mod soong_package;
mod utils;

use crate::context::*;
use crate::project::*;
use crate::utils::*;

fn generate_project(
    project: &mut Box<dyn Project>,
    is_dependency: bool,
    projects_map: &ProjectsMap,
    ctx: &Context,
) -> Result<(), String> {
    let project_name = project.get_name();
    let project_ctx = if !is_dependency {
        print_info!("Generating '{project_name}'");
        ctx.clone()
    } else {
        print_info!("Generating dependency '{project_name}'");
        let mut dep_ctx = ctx.clone();
        dep_ctx.copy_to_aosp = false;
        dep_ctx.skip_build = true;
        dep_ctx
    };
    print_debug!("Creating soong package...");
    let package = project.generate_package(&project_ctx, projects_map)?;
    if !is_dependency {
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
    }
    Ok(())
}

fn generate_projects(
    mut projects_map: ProjectsMap,
    project_ids: Vec<ProjectId>,
    ctx: &Context,
) -> Result<(), String> {
    let mut projects_to_generate = if project_ids.len() == 0 {
        VecDeque::from_iter(projects_map.iter().map(|(key, _)| key.clone()))
    } else {
        VecDeque::from_iter(project_ids)
    };
    let project_to_write: HashSet<ProjectId> = HashSet::from_iter(projects_to_generate.clone());

    let mut projects_generated = HashSet::new();
    while let Some(project_id) = projects_to_generate.pop_front().as_ref() {
        if projects_generated.contains(project_id) {
            continue;
        }
        let mut project = projects_map.remove(project_id)?;
        let missing_deps = project_id
            .get_deps()
            .into_iter()
            .filter(|dep| !projects_generated.contains(dep));
        if missing_deps.clone().count() > 0 {
            projects_to_generate.extend(missing_deps);
            projects_to_generate.push_back(project_id.clone());
        } else {
            generate_project(
                &mut project,
                !project_to_write.contains(project_id),
                &projects_map,
                ctx,
            )?;
            projects_generated.insert(project_id.clone());
        }
        projects_map.insert(project_id.clone(), project);
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let projects_map = ProjectsMap::new();
    let (ctx, project_ids) = match Context::parse_args(&projects_map) {
        Ok(context) => context,
        Err(err) => {
            print_error!("{err}");
            return Err(format!("parse_args failed"));
        }
    };

    if let Err(err) = generate_projects(projects_map, project_ids, &ctx) {
        print_error!("{err}");
        Err(format!("generate_projects failed"))
    } else {
        Ok(())
    }
}
