use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), $message))
    };
}
pub use error;

pub const BANNER: &str = "\x1b[01;32m[NINJA-TO-SOONG]\x1b[0m";

pub const CC_LIB_HEADERS_SPIRV_TOOLS: &str = "SPIRV-Tools-includes";
pub const CC_LIB_HEADERS_SPIRV_HEADERS: &str = "SPIRV-Headers-includes";
pub const CC_LIB_HEADERS_LLVM: &str = "llvm-includes";
pub const CC_LIB_HEADERS_CLANG: &str = "clang-includes";
pub const CC_LIB_HEADERS_CLSPV: &str = "clspv-includes";

pub const ANDROID_ISA: &str = "aarch64";
pub const ANDROID_ABI: &str = "arm64-v8a";
pub const ANDROID_PLATFORM: &str = "35";

pub const LLVM_DISABLE_ZLIB: &str = "-DLLVM_ENABLE_ZLIB=OFF";

pub fn add_slash_suffix(str: &str) -> String {
    str.to_string() + "/"
}

pub fn rework_name(origin: String) -> String {
    origin.replace("/", "_").replace(".", "_")
}

pub fn spirv_headers_name(spirv_headers_root: &str, str: &str) -> String {
    rework_name(str.replace(spirv_headers_root, CC_LIB_HEADERS_SPIRV_HEADERS))
}

pub fn clang_headers_name(clang_headers_root: &str, str: &str) -> String {
    rework_name(str.replace(clang_headers_root, CC_LIB_HEADERS_CLANG))
}

pub fn llvm_headers_name(llvm_headers_root: &str, str: &str) -> String {
    rework_name(str.replace(llvm_headers_root, CC_LIB_HEADERS_LLVM))
}

pub fn cmake_configure(
    source: &str,
    build: &str,
    ndk_root: &str,
    args: Vec<&str>,
) -> Result<bool, String> {
    if std::env::var("NINJA_TO_SOONG_SKIP_CMAKE_CONFIGURE").is_ok() {
        return Ok(false);
    }
    let mut command = std::process::Command::new("cmake");
    command
        .args([
            "-B",
            build,
            "-S",
            source,
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
            &("-DCMAKE_TOOLCHAIN_FILE=".to_string()
                + ndk_root
                + "/build/cmake/android.toolchain.cmake"),
            &("-DANDROID_ABI=".to_string() + ANDROID_ABI),
            &("-DANDROID_PLATFORM=".to_string() + ANDROID_PLATFORM),
        ])
        .args(args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!(format!("cmake from '{source}' to '{build}' failed: {err}"));
    }
    return Ok(true);
}

pub fn cmake_build(build: &str, targets: Vec<&str>) -> Result<(), String> {
    let target_args = targets.into_iter().fold(Vec::new(), |mut vec, target| {
        vec.push("--target");
        vec.push(target);
        vec
    });
    let mut command = std::process::Command::new("cmake");
    command.args(["--build", &build]).args(target_args);
    println!("{command:#?}");
    if let Err(err) = command.status() {
        return error!(format!("cmake build '{0}' failed: {err}", &build));
    }
    return Ok(());
}

pub fn copy_file(from: &str, to: &str) -> Result<(), String> {
    if let Err(err) = std::fs::copy(from, to) {
        return error!(format!("copy({from}, {to}) failed: {err}"));
    }
    return Ok(());
}

pub fn copy_files(files: HashSet<String>, src_root: &str, dst_root: &str) -> Result<(), String> {
    for file in files {
        let from = add_slash_suffix(src_root) + &file;
        let to = add_slash_suffix(dst_root) + &file;
        let to_dir = to.rsplit_once("/").unwrap().0;
        if let Err(err) = std::fs::create_dir_all(to_dir) {
            return error!(format!("create_dir_all({to_dir}) failed: {err}"));
        }
        copy_file(&from, &to)?;
    }
    println!("{BANNER} \t\tFiles copied successfully from '{src_root}' to '{src_root}'!");
    return Ok(());
}

pub fn touch_directories(directories: &HashSet<String>, dst_root: &str) -> Result<(), String> {
    for include_dir in directories {
        let dir = dst_root.to_string() + include_dir;
        if touch::exists(&dir) {
            continue;
        }
        if let Err(err) = std::fs::create_dir_all(&dir) {
            return error!(format!("create_dir_all({dir}) failed: {err}"));
        }
        if let Err(err) = touch::file::create(&(dir.clone() + "/touch"), false) {
            return error!(format!("touch in '{dir}' failed: {err}"));
        }
    }
    println!("{BANNER} \t\tInclude directories created successfully!");
    return Ok(());
}

pub fn remove_directory(directory: String) -> Result<(), String> {
    if touch::exists(&directory) {
        if let Err(err) = std::fs::remove_dir_all(&directory) {
            return error!(format!("remove_dir_all failed: {err}"));
        }
    }
    println!("{BANNER} \t\t'{directory}' removed successfully!");
    return Ok(());
}

pub fn write_file(file_path: &str, content: &str) -> Result<(), String> {
    match File::create(file_path) {
        Ok(mut file) => {
            if let Err(err) = file.write_fmt(format_args!("{0}", content)) {
                return error!(format!("Could not write into '{file_path}': '{err:#?}"));
            }
        }
        Err(err) => {
            return error!(format!("Could not create '{file_path}': '{err}'"));
        }
    }
    println!("{BANNER} \t\t'{file_path}' created successfully!");
    return Ok(());
}
