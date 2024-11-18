#[macro_export]
macro_rules! error {
    ($message:expr) => {
        Err(format!("{0}:{1}: {2}", file!(), line!(), $message))
    };
}
pub use error;

pub fn rework_target_name(target_name: &str, prefix: &str) -> String {
    let mut name = prefix.to_string() + target_name;
    name = name.strip_suffix(".so").unwrap_or(&name).to_string();
    name = name.strip_suffix(".a").unwrap_or(&name).to_string();
    return name.replace("/", "__").replace(".", "__");
}

pub fn rework_source_path(source: &str, source_root: &str) -> String {
    let source = source.strip_prefix(source_root).unwrap_or(&source);
    return String::from(source);
}
