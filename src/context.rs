// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;
use std::env;

use crate::project::*;
use crate::utils::*;

#[derive(Default, Clone)]
pub struct Context {
    pub projects_to_generate: VecDeque<ProjectId>,
    pub temp_path: PathBuf,
    pub test_path: PathBuf,
    pub android_path: PathBuf,
    pub skip_gen_ninja: bool,
    pub skip_build: bool,
    pub copy_to_aosp: bool,
    pub wildcardize_paths: bool,
}

impl Context {
    fn get_partial_matching_project(projects_map: &ProjectsMap, target: &str) -> Option<ProjectId> {
        for (project_id, project) in projects_map.iter() {
            if project
                .get_name()
                .to_lowercase()
                .contains(&target.to_lowercase())
            {
                print_info!("Could not find perfect match for {target:#?}, but found partial match with {:#?}", project.get_name());
                return Some(*project_id);
            }
        }
        None
    }

    pub fn parse_args(projects_map: &ProjectsMap) -> Result<Self, String> {
        const AOSP_PATH: &str = "--aosp-path";
        const CLEAN_TMP: &str = "--clean-tmp";
        const COPY_TO_AOSP: &str = "--copy-to-aosp";
        const SKIP_BUILD: &str = "--skip-build";
        const SKIP_GEN_NINJA: &str = "--skip-gen-ninja";

        let args = env::args().collect::<Vec<String>>();
        let exec = file_name(&Path::new(&args[0]));
        let mut iter = args[1..].iter();
        let mut clean_tmp = false;
        let project_name_to_id = projects_map.iter().fold(
            std::collections::HashMap::new(),
            |mut map, (project_id, project)| {
                map.insert(project.get_name(), *project_id);
                map
            },
        );
        let mut ctx = Self::default();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                AOSP_PATH => {
                    let Some(path) = iter.next() else {
                        return Err(format!("<path> missing for {AOSP_PATH}"));
                    };
                    ctx.android_path = PathBuf::from(path);
                }
                SKIP_GEN_NINJA => {
                    ctx.skip_gen_ninja = true;
                    ctx.skip_build = true
                }
                SKIP_BUILD => ctx.skip_build = true,
                COPY_TO_AOSP => {
                    ctx.copy_to_aosp = true;
                    ctx.wildcardize_paths = true;
                }
                CLEAN_TMP => clean_tmp = true,
                "-h" | "--help" => {
                    let mut projects_help = projects_map
                        .iter()
                        .map(|(_, project)| format!("  {0}\n", project.get_name()))
                        .collect::<Vec<String>>();
                    projects_help.sort_unstable();
                    return Err(format!(
                        "
USAGE: {exec} [OPTIONS] [PROJECTS]

PROJECTS:
{0}
OPTIONS:
{AOSP_PATH} <path>   Path to Android tree (required for most project)
{CLEAN_TMP}          Remove temporary directory before running
{COPY_TO_AOSP}       Copy generated Soong files into the Android tree
{SKIP_BUILD}         Skip build step
{SKIP_GEN_NINJA}     Skip generation of Ninja files
-h, --help           Display the help and exit
",
                        projects_help.concat()
                    ));
                }
                project => match project_name_to_id.get(project) {
                    Some(project) => ctx.projects_to_generate.push_back(*project),
                    None => match Self::get_partial_matching_project(projects_map, project) {
                        Some(project) => ctx.projects_to_generate.push_back(project),
                        None => return Err(format!("Unknown project '{project}'")),
                    },
                },
            }
        }
        if ctx.copy_to_aosp && !ctx.android_path.exists() {
            return error!("'{COPY_TO_AOSP}' requires a valid '{AOSP_PATH}'");
        }
        // TEMP_PATH
        ctx.temp_path = if let Ok(dir) = env::var("N2S_TMP_PATH") {
            PathBuf::from(dir)
        } else {
            env::temp_dir()
        }
        .join(&exec);
        if clean_tmp && remove_dir(&ctx.temp_path)? {
            print_info!("{0:#?} removed", ctx.temp_path);
        }
        if create_dir(&ctx.temp_path)? {
            print_info!("{0:#?} created", ctx.temp_path);
        }
        // TEST_PATH
        ctx.test_path = match env::current_exe() {
            Ok(exe_path) => {
                PathBuf::from(
                    exe_path // <ninja-to-soong>/target/<build-mode>/ninja-to-soong
                        .parent() // <ninja-to-soong>/target/<build-mode>
                        .unwrap()
                        .parent() // <ninja-to-soong>/target
                        .unwrap()
                        .parent() // <ninja-to-soong>
                        .unwrap()
                        .join("tests"), // <ninja-to-soong>/tests
                )
            }
            Err(err) => return error!("Could not get current executable path: {err}"),
        };
        // PROJECTS_TO_GENERATE
        if ctx.projects_to_generate.len() == 0 {
            ctx.projects_to_generate =
                VecDeque::from_iter(projects_map.iter().map(|(key, _)| *key));
        }

        Ok(ctx)
    }
}
