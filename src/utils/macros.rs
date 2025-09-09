// Copyright 2024 ninja-to-soong authors
// SPDX-License-Identifier: Apache-2.0

pub const TAB: &str = "   ";
pub const COLOR_RED: &str = "\x1b[00;31m";
pub const COLOR_GREEN: &str = "\x1b[00;32m";
pub const COLOR_GREEN_BOLD: &str = "\x1b[01;32m";
pub const COLOR_NONE: &str = "\x1b[0m";

#[macro_export]
macro_rules! print_internal {
    ($print_prefix:expr, $message_prefix:expr, $print_suffix:expr, $($arg:tt)*) => {
        println!(
            "{0}[N2S]{1} {2}{3}",
            $print_prefix, $message_prefix, format!($($arg)*), $print_suffix
        );
    };
}
#[macro_export]
macro_rules! print_verbose {
    ($($arg:tt)*) => {
        print_internal!(COLOR_GREEN, format!("{COLOR_NONE}{TAB}{TAB}"), "", $($arg)*);
    };
}
#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {
        print_internal!(COLOR_GREEN, format!("{COLOR_NONE}{TAB}"), "", $($arg)*);
    };
}
#[macro_export]
macro_rules! print_info {
    ($($arg:tt)*) => {
        print_internal!(
            format!("\n{COLOR_GREEN}"),
            COLOR_GREEN_BOLD,

            COLOR_NONE,
            $($arg)*,
        );
    };
}
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        print_internal!(COLOR_RED, "", COLOR_NONE, $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), format!($($arg)*)))
    };
}

#[macro_export]
macro_rules! debug_project {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug_project")]
        print_verbose!($($arg)*)
    }
}

pub fn execute_command(program: &str, args: &[&str], description: String) -> Result<(), String> {
    let mut command = std::process::Command::new(program);
    command.args(args);
    println!("{command:#?}");
    match command.status() {
        Ok(status) => {
            if !status.success() {
                return error!("{description} failed");
            }
        }
        Err(err) => return error!("{description} failed: {err}"),
    }
    Ok(())
}

#[macro_export]
macro_rules! execute_cmd {
    ($program:expr, $args:expr) => {
        execute_command(
            $program,
            &$args,
            format!("{0}:{1}: {2}", file!(), line!(), $program),
        )
    };
}

#[macro_export]
macro_rules! define_ProjectId {
    ($(($project:ident, $module:ident)),*) => {
        $(mod $module;)*
        #[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
        pub enum ProjectId {
            External,
            $($project,)*
        }
        fn get_projects() -> HashMap<ProjectId, Box<dyn Project>> {
            let mut projects: HashMap<ProjectId, Box<dyn Project>> = HashMap::new();
            $(projects.insert(ProjectId::$project, Box::new($module::$project::default()));)*
            projects
        }
    };
}

#[macro_export]
macro_rules! define_Dep {
    ($(($deps:ident, $project:ident, ($($projects:ident),*))),*) => {
        #[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
        pub enum Dep {
            $($deps,)*
        }
        impl Dep {
            pub fn projects(&self) -> (ProjectId, Vec<ProjectId>) {
                match self {
                    $(Self::$deps => (ProjectId::$project, vec![$(ProjectId::$projects,)*]),)*
                }
            }
        }
        fn get_deps() -> Vec<Dep> {
            vec![$(Dep::$deps,)*]
        }
    };
}

#[macro_export]
macro_rules! target {
    ($path:expr) => {
        NinjaTargetToGen {
            path: String::from($path),
            entry: NinjaTargetToGenMapEntry {
                name: None,
                stem: None,
                module_type: None,
            },
        }
    };
    ($path:expr, $name:expr) => {
        NinjaTargetToGen {
            path: String::from($path),
            entry: NinjaTargetToGenMapEntry {
                name: Some(PathBuf::from($name)),
                stem: None,
                module_type: None,
            },
        }
    };
    ($path:expr, $name:expr, $stem:expr) => {
        NinjaTargetToGen {
            path: String::from($path),
            entry: NinjaTargetToGenMapEntry {
                name: Some(PathBuf::from($name)),
                stem: Some(String::from($stem)),
                module_type: None,
            },
        }
    };
}
#[macro_export]
macro_rules! target_typed {
    ($path:expr, $module_type:expr) => {
        NinjaTargetToGen {
            path: String::from($path),
            entry: NinjaTargetToGenMapEntry {
                name: None,
                stem: None,
                module_type: Some(String::from($module_type)),
            },
        }
    };
    ($path:expr, $module_type:expr, $name:expr) => {
        NinjaTargetToGen {
            path: String::from($path),
            entry: NinjaTargetToGenMapEntry {
                name: Some(PathBuf::from($name)),
                stem: None,
                module_type: Some(String::from($module_type)),
            },
        }
    };
}

pub use {
    debug_project, define_Dep, define_ProjectId, error, execute_cmd, print_info, print_internal,
    print_verbose, target, target_typed,
};
