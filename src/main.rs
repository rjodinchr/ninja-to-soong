// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, VecDeque};

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
    project: &mut dyn Project,
    is_dependency: bool,
    projects_generated: &ProjectsMap,
    ctx: &Context,
) -> Result<(), String> {
    let project_name = project.get_id().str();
    let android_path = project.get_id().android_path(ctx);
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
    let package = project.generate_package(&project_ctx, projects_generated)?;
    if !is_dependency {
        print_debug!("Writing soong file...");

        const ANDROID_BP: &str = "Android.bp";
        let file_path = ctx.test_path.join(project_name).join(ANDROID_BP);
        write_file(file_path.as_path(), &package.print())?;
        print_verbose!("{file_path:#?} created");

        if ctx.copy_to_aosp {
            let copy_dst = android_path.join(ANDROID_BP);
            copy_file(&file_path, &copy_dst)?;
            print_verbose!("{file_path:#?} copied to {copy_dst:#?}");
        }
    }
    Ok(())
}

fn generate_projects<'a>(
    all_projects: Vec<&'a mut dyn Project>,
    project_ids: Vec<ProjectId>,
    ctx: &Context,
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
            ctx,
        )?;
        projects_generated.insert(project.get_id(), project);
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let mut angle = angle::Angle::default();
    let mut clvk = clvk::Clvk::default();
    let mut clspv = clspv::Clspv::default();
    let mut llvm_project = llvm_project::LlvmProject::default();
    let mut mesa = mesa::Mesa::default();
    let mut spirv_headers = spirv_headers::SpirvHeaders::default();
    let mut spirv_tools = spirv_tools::SpirvTools::default();

    let all_projects: Vec<&mut dyn Project> = vec![
        &mut angle,
        &mut clvk,
        &mut clspv,
        &mut llvm_project,
        &mut mesa,
        &mut spirv_headers,
        &mut spirv_tools,
    ];

    let (ctx, exec, project_ids) =
        match Context::parse_args(std::env::args().collect(), &all_projects) {
            Ok(context) => context,
            Err(err) => {
                print_error!("{err}");
                return Err(format!("Could not create context"));
            }
        };

    if let Err(err) = generate_projects(all_projects, project_ids, &ctx) {
        print_error!("{err}");
        Err(format!("{exec} failed"))
    } else {
        Ok(())
    }
}
