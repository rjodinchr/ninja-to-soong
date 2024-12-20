// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

use crate::project::*;
use crate::utils::*;

const AOSP_PATH: &str = "--aosp-path";
const CLEAN_TMP: &str = "--clean-tmp";
const COPY_TO_AOSP: &str = "--copy-to-aosp";
const SKIP_BUILD: &str = "--skip-build";
const SKIP_GEN_NINJA: &str = "--skip-gen-ninja";

fn help(exec: &str, projects: &Vec<&mut dyn Project>) -> String {
    let projects_help = projects
        .iter()
        .map(|project| project.get_id().str())
        .collect::<Vec<&str>>()
        .join("\n  ");
    format!(
        "
USAGE: {exec} [OPTIONS] [PROJECTS]

PROJECTS:
{projects_help}

OPTIONS:
{AOSP_PATH} <path>   Path to Android tree (required for most project)
{CLEAN_TMP}          Remove temporary directory before running
{COPY_TO_AOSP}       Copy generated Soong files into the Android tree
{SKIP_BUILD}         Skip build step
{SKIP_GEN_NINJA}     Skip generation of Ninja files
-h, --help           Display the help and exit
"
    )
}

#[derive(Default)]
pub struct Context {
    pub temp_path: PathBuf,
    pub test_path: PathBuf,
    pub android_path: PathBuf,
    pub skip_gen_ninja: bool,
    pub skip_build: bool,
    pub copy_to_aosp: bool,
}

impl Context {
    pub fn parse_args(
        args: Vec<String>,
        projects: &Vec<&mut dyn Project>,
    ) -> Result<(Self, String, Vec<ProjectId>), String> {
        let exec = file_name(&Path::new(&args[0]));
        let mut iter = args[1..].iter();
        let mut clean_tmp = false;
        let mut project_ids = Vec::new();
        let mut ctx = Self::default();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                AOSP_PATH => {
                    let Some(path) = iter.next() else {
                        return Err(help(&exec, projects));
                    };
                    ctx.android_path = PathBuf::from(path);
                }
                SKIP_GEN_NINJA => {
                    ctx.skip_gen_ninja = true;
                    ctx.skip_build = true
                }
                SKIP_BUILD => ctx.skip_build = true,
                COPY_TO_AOSP => ctx.copy_to_aosp = true,
                CLEAN_TMP => clean_tmp = true,
                "-h" | "--help" => return Err(help(&exec, projects)),
                project => match ProjectId::from(project) {
                    Ok(project) => project_ids.push(project),
                    Err(err) => return Err(format!("{0}\n{1}", err, help(&exec, projects))),
                },
            }
        }
        if ctx.copy_to_aosp && std::fs::File::open(&ctx.android_path).is_err() {
            return error!("'{COPY_TO_AOSP}' requires a valid '{AOSP_PATH}'");
        }

        ctx.temp_path = if let Ok(dir) = std::env::var("N2S_TMP_PATH") {
            PathBuf::from(dir)
        } else {
            std::env::temp_dir()
        }
        .join(&exec);
        if clean_tmp && std::fs::read_dir(&ctx.temp_path).is_ok() {
            print_info!("Removing temporary directory {0:#?}", ctx.temp_path);
            if let Err(err) = std::fs::remove_dir_all(&ctx.temp_path) {
                return error!(
                    "Could not remove temporary directory {0:#?}: {err}",
                    ctx.temp_path
                );
            }
        }
        if std::fs::read_dir(&ctx.temp_path).is_err() {
            print_info!("Creating temporary directory in {0:#?}", ctx.temp_path);
            if let Err(err) = std::fs::create_dir_all(&ctx.temp_path) {
                return error!(
                    "Could not create temporary directory {0:#?}: {err}",
                    ctx.temp_path
                );
            }
        }
        ctx.test_path = match std::env::current_exe() {
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

        Ok((ctx, exec, project_ids))
    }
}
