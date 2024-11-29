#[macro_export]
macro_rules! error {
    ($message:expr) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), $message))
    };
}
pub use error;

pub const PRINT_BANNER: &str = "[NINJA-TO-SOONG]";

pub const SPIRV_TOOLS: &str = "SPIRV-Tools-includes";
pub const SPIRV_HEADERS: &str = "SPIRV-Headers-includes";
pub const LLVM_HEADERS: &str = "llvm-includes";
pub const CLANG_HEADERS: &str = "clang-includes";
pub const CLSPV_HEADERS: &str = "clspv-includes";

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
    rework_name(str.replace(spirv_headers_root, SPIRV_HEADERS))
}

pub fn clang_headers_name(clang_headers_root: &str, str: &str) -> String {
    rework_name(str.replace(clang_headers_root, CLANG_HEADERS))
}

pub fn llvm_headers_name(llvm_headers_root: &str, str: &str) -> String {
    rework_name(str.replace(llvm_headers_root, LLVM_HEADERS))
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
