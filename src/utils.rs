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