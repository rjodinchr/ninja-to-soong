#[macro_export]
macro_rules! error {
    ($message:expr) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), $message))
    };
}
pub use error;

pub fn add_slash_suffix(str: &str) -> String {
    str.to_string() + "/"
}

pub const SPIRV_TOOLS: &str = "SPIRV-Tools-includes";
pub const SPIRV_HEADERS: &str = "SPIRV-Headers-includes";
pub const LLVM_HEADERS: &str = "llvm-includes";
pub const CLANG_HEADERS: &str = "clang-includes";
pub const CLSPV_HEADERS: &str = "clspv-includes";

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
