use crate::print_info;
use crate::project::*;
use crate::utils::*;

pub const AOSP_PATH: &str = "--aosp-path";
pub const ANGLE_PATH: &str = "--angle-path";
pub const COPY_TO_AOSP: &str = "--copy-to-aosp";

const CLEAN_TMP: &str = "--clean-tmp";
const SKIP_BUILD: &str = "--skip-build";
const SKIP_GEN_NINJA: &str = "--skip-gen-ninja";

#[derive(Default)]
pub struct Context {
    pub executable: String,
    pub temp_path: PathBuf,
    pub n2s_path: PathBuf,
    pub test_path: PathBuf,
    android_path: Option<PathBuf>,
    angle_path: Option<PathBuf>,
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
USAGE: {0} [OPTIONS] [PROJECTS]

PROJECTS:
{projects_help}
OPTIONS:
  {ANGLE_PATH} <path>  Path to angle source repository
  {AOSP_PATH} <path>   Path to android tree
  {CLEAN_TMP}          Remove temporary directory before running
  {COPY_TO_AOSP}       Copy generated Soong file into <android_path> tree
  {SKIP_BUILD}         Skip build step
  {SKIP_GEN_NINJA}     Skip generation of Ninja files
  -h, --help           Display the help and exit
 ",
            self.executable
        )
    }
    fn next(
        &self,
        iter: &mut std::slice::Iter<'_, String>,
        projects: &Vec<&mut dyn Project>,
    ) -> Result<Option<PathBuf>, String> {
        let Some(path) = iter.next() else {
            return Err(self.help(projects));
        };
        Ok(Some(PathBuf::from(path)))
    }
    pub fn new(args: Vec<String>, projects: &Vec<&mut dyn Project>) -> Result<Self, String> {
        let mut ctx = Self::default();
        ctx.executable = file_name(&Path::new(&args[0]));
        ctx.temp_path = std::env::temp_dir().join(&ctx.executable);

        let mut clean_tmp = false;
        let mut iter = args[1..].iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                ANGLE_PATH => {
                    ctx.angle_path = ctx.next(&mut iter, projects)?;
                }
                AOSP_PATH => {
                    ctx.android_path = ctx.next(&mut iter, projects)?;
                }
                SKIP_GEN_NINJA => ctx.skip_gen_ninja = true,
                SKIP_BUILD => ctx.skip_build = true,
                COPY_TO_AOSP => ctx.copy_to_aosp = true,
                CLEAN_TMP => clean_tmp = true,
                "-h" | "--help" => return Err(ctx.help(projects)),
                project => match ProjectId::from(project) {
                    Ok(project) => ctx.project_ids.push(project),
                    Err(err) => return Err(format!("{0}\n{1}", err, ctx.help(projects))),
                },
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
    pub fn get_path(&self, arg: &str, dep: &str) -> Result<PathBuf, String> {
        let optional_path = match arg {
            AOSP_PATH => self.android_path.as_ref(),
            ANGLE_PATH => self.angle_path.as_ref(),
            _ => return error!("'{arg}' not supported for get_path"),
        };
        let Some(path) = optional_path else {
            return error!("missing '{arg}' required for '{dep}'");
        };
        Ok(path.to_path_buf())
    }
    pub fn get_android_path(&self, dep: &str) -> Result<PathBuf, String> {
        self.get_path(AOSP_PATH, dep)
    }
}
