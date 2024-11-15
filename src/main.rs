use std::fs::File;
use std::io::Write;

mod parser;
mod target;
mod generator;
mod utils;

fn main() {
    let device_targets = match parser::parse_build_ninja(
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/build/build.ninja",
    ) {
        Ok(targets) => targets,
        Err(err) => {
            println!("Could not parse build.ninja: '{err}'");
            return;
        }
    };
    let device_targets_map = target::create_map(&device_targets);
    let android_bp = match generator::generate_android_bp(
        vec![String::from("libOpenCL.so")],
        device_targets_map,
        "/usr/local/google/home/rjodin/aluminium/external/angle/",
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/android-ndk-r27c/",
        "/usr/local/google/home/rjodin/aluminium/external/angle/third_party/clvk/build/",
    ) {
        Ok(result) => result,
        Err(err) => {
            println!("generate_android_bp failed: {err}");
            return;
        }
    };

    let mut file = match File::create("Android.bp") {
        Ok(file) => file,
        Err(err) => {
            println!("Could not create 'Android.bp': '{err}'");
            return;
        }
    };
    let _ = file.write_fmt(format_args!("{android_bp}"));
}
