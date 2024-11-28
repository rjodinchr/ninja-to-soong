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

pub const SPIRV_HEADERS: &str = "SPIRV-Headers";
pub const LLVM_HEADERS: &str = "LLVM-Headers";
pub const CLANG_HEADERS: &str = "CLANG-Headers";

fn rework_name(origin: &str, from: &str, to: &str) -> String {
    origin.replace(from, to).replace("/", "_").replace(".", "_")
}

pub fn spirv_headers_name(spirv_headers_root: &str, str: &str) -> String {
    rework_name(str, spirv_headers_root, SPIRV_HEADERS)
}

pub fn clang_headers_name(clang_headers_root: &str, str: &str) -> String {
    rework_name(str, clang_headers_root, CLANG_HEADERS)
}

pub fn llvm_headers_name(llvm_headers_root: &str, str: &str) -> String {
    rework_name(str, llvm_headers_root, LLVM_HEADERS)
}
