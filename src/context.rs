use crate::print_info;
use crate::project::*;
use crate::utils::*;

pub const ANGLE_PATH: &str = "--angle-path";
const CLEAN_TMP: &str = "--clean-tmp";
const COPY_TO_AOSP: &str = "--copy-to-aosp";
const SKIP_BUILD: &str = "--skip-build";
const SKIP_GEN_NINJA: &str = "--skip-gen-ninja";

#[derive(Default)]
pub struct Context {
    pub executable: String,
    pub android_path: PathBuf,
    pub temp_path: PathBuf,
    pub n2s_path: PathBuf,
    pub test_path: PathBuf,
    pub angle_path: Option<PathBuf>,
    pub project_ids: Vec<ProjectId>,
    pub skip_gen_ninja: bool,
    pub skip_build: bool,
    pub copy_to_aosp: bool,
}

impl Context {
    fn help(&self, projects: &Vec<&mut dyn Project>) -> String {
        let mut projects_help = String::new();
        for project in projects {
            projects_help += "  ";
            projects_help += project.get_id().str();
            projects_help += "\n";
        }
        format!(
            "
USAGE: {0} [OPTIONS] <android_path> [PROJECTS]

PROJECTS:
{projects_help}
OPTIONS:
  {ANGLE_PATH}       Path to angle source repository
  {CLEAN_TMP}        Remove temporary directory before running
  {COPY_TO_AOSP}     Copy generated Soong file into <android_path> tree
  {SKIP_BUILD}       Skip build step
  {SKIP_GEN_NINJA}   Skip generation of Ninja files
  -h, --help         Display the help and exit
 ",
            self.executable
        )
    }
    pub fn new(args: Vec<String>, projects: &Vec<&mut dyn Project>) -> Result<Self, String> {
        let mut ctx = Self::default();
        ctx.executable = file_name(&Path::new(&args[0]));
        ctx.temp_path = std::env::temp_dir().join(&ctx.executable);

        let mut android_path = None;
        let mut clean_tmp = false;
        let mut iter = args[1..].iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                ANGLE_PATH => {
                    let Some(angle_path) = iter.next() else {
                        return Err(ctx.help(projects));
                    };
                    ctx.angle_path = Some(PathBuf::from(angle_path));
                }
                SKIP_GEN_NINJA => ctx.skip_gen_ninja = true,
                SKIP_BUILD => ctx.skip_build = true,
                COPY_TO_AOSP => ctx.copy_to_aosp = true,
                CLEAN_TMP => clean_tmp = true,
                "-h" | "--help" => return Err(ctx.help(projects)),
                _ => {
                    if arg.starts_with("-") {
                        return Err(ctx.help(projects));
                    }
                    android_path = Some(PathBuf::from(arg.clone()));
                    while let Some(project) = iter.next() {
                        match ProjectId::from(project) {
                            Ok(project) => ctx.project_ids.push(project),
                            Err(err) => return Err(format!("{0}\n{1}", err, ctx.help(projects))),
                        }
                    }
                }
            }
        }
        match android_path {
            Some(path) => ctx.android_path = path,
            None => {
                return Err(format!(
                    "ERROR: Missing 'android_path'\n{0}",
                    ctx.help(projects)
                ))
            }
        }

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
        ctx.n2s_path = match std::env::current_exe() {
            Ok(exe_path) => {
                exe_path // <ninja-to-soong>/target/<build-mode>/ninja-to-soong
                    .parent() // <ninja-to-soong>/target/<build-mode>
                    .unwrap()
                    .parent() // <ninja-to-soong>/target
                    .unwrap()
                    .parent() // <ninja-to-soong>
                    .unwrap()
                    .to_path_buf()
            }
            Err(err) => return error!("Could not get current executable path: {err}"),
        };
        ctx.test_path = ctx.n2s_path.join("tests");

        Ok(ctx)
    }
}
