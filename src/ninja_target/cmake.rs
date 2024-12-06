use std::collections::{HashMap, HashSet};

use super::*;

#[derive(Debug)]
pub struct CmakeNinjaTarget {
    rule: String,
    outputs: Vec<PathBuf>,
    implicit_outputs: Vec<PathBuf>,
    inputs: Vec<PathBuf>,
    implicit_deps: Vec<PathBuf>,
    order_only_deps: Vec<PathBuf>,
    variables: HashMap<String, String>,
}

impl GeneratorTarget for CmakeNinjaTarget {
    fn new(
        rule: String,
        outputs: Vec<PathBuf>,
        implicit_outputs: Vec<PathBuf>,
        inputs: Vec<PathBuf>,
        implicit_deps: Vec<PathBuf>,
        order_only_deps: Vec<PathBuf>,
        variables: HashMap<String, String>,
    ) -> Self {
        Self {
            rule,
            outputs,
            implicit_outputs,
            inputs,
            implicit_deps,
            order_only_deps,
            variables,
        }
    }

    fn set_globals(&mut self, _globals: HashMap<String, String>) {}

    fn get_rule(&self) -> Option<NinjaRule> {
        Some(if self.rule.starts_with("CXX_SHARED_LIBRARY") {
            NinjaRule::SharedLibrary
        } else if self.rule.starts_with("CXX_STATIC_LIBRARY") {
            NinjaRule::StaticLibrary
        } else if self.rule.starts_with("CUSTOM_COMMAND") {
            NinjaRule::CustomCommand
        } else {
            return None;
        })
    }

    fn get_all_inputs(&self) -> Vec<PathBuf> {
        let mut inputs = Vec::new();
        for input in &self.inputs {
            inputs.push(input.clone());
        }
        for input in &self.implicit_deps {
            inputs.push(input.clone());
        }
        for input in &self.order_only_deps {
            inputs.push(input.clone());
        }
        inputs
    }

    fn get_inputs(&self) -> &Vec<PathBuf> {
        &self.inputs
    }

    fn get_all_outputs(&self) -> Vec<PathBuf> {
        let mut outputs = Vec::new();
        for output in &self.outputs {
            outputs.push(output.clone());
        }
        for output in &self.implicit_outputs {
            outputs.push(output.clone());
        }
        outputs
    }

    fn get_outputs(&self) -> &Vec<PathBuf> {
        &self.outputs
    }

    fn get_name(&self, prefix: &Path) -> String {
        path_to_id(prefix.join(&self.outputs[0]))
    }

    fn get_link_flags(&self) -> (Option<PathBuf>, HashSet<String>) {
        let mut link_flags = HashSet::new();
        let mut version_script = None;
        if let Some(flags) = self.variables.get("LINK_FLAGS") {
            for flag in flags.split(" ") {
                if let Some(vs) = flag.strip_prefix("-Wl,--version-script=") {
                    version_script = Some(PathBuf::from(vs));
                }
                link_flags.insert(flag.to_string());
            }
        }
        (version_script, link_flags)
    }

    fn get_link_libraries(&self) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String> {
        let mut static_libraries = HashSet::new();
        let mut shared_libraries = HashSet::new();
        let Some(libs) = self.variables.get("LINK_LIBRARIES") else {
            return Ok((static_libraries, shared_libraries));
        };
        for lib in libs.split(" ") {
            if lib.starts_with("-") || lib.is_empty() {
                continue;
            } else {
                let lib_path = PathBuf::from(lib);
                if lib.ends_with(".a") {
                    static_libraries.insert(lib_path);
                } else if lib.ends_with(".so") {
                    shared_libraries.insert(lib_path);
                } else {
                    return error!("unsupported library '{lib}' from target: {self:#?}");
                }
            }
        }
        Ok((static_libraries, shared_libraries))
    }

    fn get_defines(&self) -> HashSet<String> {
        let mut defines = HashSet::new();
        if let Some(defs) = self.variables.get("DEFINES") {
            for define in defs.split("-D") {
                if define.is_empty() {
                    continue;
                }
                defines.insert(define.trim().to_string());
            }
        };
        defines
    }

    fn get_includes(&self) -> HashSet<PathBuf> {
        let mut includes = HashSet::new();
        let Some(incs) = self.variables.get("INCLUDES") else {
            return includes;
        };
        for inc in incs.split(" ") {
            let include = inc.strip_prefix("-I").unwrap_or(inc);
            if include.is_empty() || include == "isystem" {
                continue;
            }
            includes.insert(PathBuf::from(include));
        }
        includes
    }

    fn get_cmd(&self) -> Result<Option<String>, String> {
        let Some(command) = self.variables.get("COMMAND") else {
            return error!("No command in: {self:#?}");
        };
        let mut split = command.split(" && ");
        let split_count = split.clone().count();
        if split_count < 2 {
            return error!(
                "Could not find enough split in command (expected at least 2, got {split_count}"
            );
        }
        let command = split.nth(1).unwrap();
        Ok(if command.contains("bin/cmake ") {
            None
        } else {
            Some(command.to_string())
        })
    }
}

pub fn cmake_configure(
    src_path: &Path,
    build_path: &Path,
    ndk_path: &Path,
    args: Vec<&str>,
) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_CONFIGURE").is_ok() {
        return Ok(false);
    }
    let mut command = std::process::Command::new("cmake");
    command
        .args([
            "-B",
            &path_to_string(build_path),
            "-S",
            &path_to_string(src_path),
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            &("-DCMAKE_TOOLCHAIN_FILE=".to_string()
                + &path_to_string(ndk_path.join("build/cmake/android.toolchain.cmake"))),
            &("-DANDROID_ABI=".to_string() + ANDROID_ABI),
            &("-DANDROID_PLATFORM=".to_string() + ANDROID_PLATFORM),
        ])
        .args(args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!("cmake_configure({src_path:#?}) failed: {err}");
    }
    Ok(true)
}

pub fn cmake_build(build_path: &Path, targets: &Vec<PathBuf>) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_BUILD").is_ok() {
        return Ok(false);
    }
    let targets_args = targets.into_iter().fold(Vec::new(), |mut vec, target| {
        vec.push("--target");
        vec.push(target.to_str().unwrap_or_default());
        vec
    });
    let mut command = std::process::Command::new("cmake");
    command
        .args(["--build", &path_to_string(build_path)])
        .args(targets_args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!("cmake_build({build_path:#?}) failed: {err}");
    }
    Ok(true)
}
