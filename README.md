# Ninja to Soong [![CI badge](https://github.com/rjodinchr/ninja-to-soong/actions/workflows/presubmit.yml/badge.svg?branch=main)](https://github.com/rjodinchr/ninja-to-soong/actions/workflows/presubmit.yml?query=branch%3Amain++)

`ninja-to-soong` is a project to generate `Soong` files (`Android.bp`) for the Android build system.

# Legal

`ninja-to-soong` is licensed under the terms of the [Apache 2.0 license](LICENSE)

# How does it work?

1. `ninja-to-soong` generates [Ninja](https://ninja-build.org/) files using either:
    - [CMake](https://cmake.org/) and the Android NDK
    - [GN](https://github.com/o-lim/generate-ninja) cross-compiling for [Android](https://gn.googlesource.com/gn/+/HEAD/docs/quick_start.md#cross_compiling-to-a-target-os-or-architecture)
    - [Meson](https://mesonbuild.com/) and the Android NDK
2. `ninja-to-soong` generates `Soong` files using [Ninja](https://ninja-build.org/) files.

# Dependencies

* [Rust](https://www.rust-lang.org/)
* [Ninja](https://ninja-build.org/)
* Depending on the projects:
  * [CMake](https://cmake.org/)
  * [GN](https://gn.googlesource.com/gn/)
  * [Meson](https://mesonbuild.com/)
  * Linux commands (`wget`, `unzip`, ...)

# Using `ninja-to-soong`

```
<ninja-to-soong> $ cargo run --release -- --aosp-path <path> <project1> <project2>
```

## Options

* `--aosp-path <path>`: Path to Android tree
* `--ext-proj-path <path>`: Path to external project rust file
* `--clean-tmp`: Remove the temporary directory before running
* `--copy-to-aosp`: Copy generated Soong files into the Android tree
* `--skip-build`: Skip build step
* `--skip-gen-ninja`: Skip generation of Ninja files
* `-h`, `--help`: Display the help and exit

## Environment variables

* `N2S_ANGLE_PATH`: Path to angle sources (default: `<aosp-path>/external/angle`)
* `N2S_NDK`: Android NDK (default: `android-ndk-r27c`)
* `N2S_NDK_PATH`: Path to Android NDK (default: temporary directory)
* `N2S_TMP_PATH`: Path used by `ninja-to-soong` to store its temporary directories (default: `std::env::temp_dir()`)

# Supported projects

| Project | Ninja Generator | Targets |
|-|-|-|
| [angle](https://github.com/google/angle) (WIP) | `GN` | `libEGL_angle.so`, `libGLESv2_angle.so`, `libGLESv1_CM_angle.so` |
| [clpeak](https://github.com/krrishnarraj/clpeak) | `CMake` | `clpeak` |
| [clspv](https://github.com/google/clspv) | `CMake` | `clvk` dependencies |
| [clvk](https://github.com/kpet/clvk) | `CMake` | `libclvk.so` |
| [fwupd](https://github.com/fwupd/fwupd.git) (WIP) | `Meson` | `fwupdmgr` & `fwupd-binder` |
| [llvm-project](https://github.com/llvm/llvm-project) | `CMake` | `clvk` & `clspv` dependencies |
| [mesa](https://www.mesa3d.org/) | `meson` | `libgallium_dri.so`, `libglapi.so`, `libEGL_mesa.so`, `libGLESv2_mesa.so`, `libGLESv1_CM_mesa.so`, `libvulkan_${VENDOR}.so` |
| [OpenCL-CTS](https://github.com/KhronosGroup/OpenCL-CTS) | `CMake` | Every binary in `test_conformance/opencl_conformance_tests_full.csv` |
| [OpenCL-ICD-Loader](https://github.com/KhronosGroup/OpenCL-ICD-Loader) | `CMake` | `libOpenCL.so` |
| [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools) | `CMake` | `clvk` dependencies & `spirv-val` (for `OpenCL-CTS`) |
| [SPIRV-Headers](https://github.com/KhronosGroup/SPIRV-Headers) | `CMake` | `clspv` & `SPIRV-Tools` dependencies |

## Adding a project

To add a project, create a `<project>.rs` implementing the `Project` trait under the `project` folder.

Then add the project in `define_ProjectId!` in `project.rs`.

The following feature can be used to output debug information when writting a new project:
```
<ninja-to-soong> $ cargo run --release --features debug_project -- --aosp-path <path> <new_project>
```

Every code leading to a change in the generated `Ninja` files should be stored under `<ninja-to-soong>/scripts/<project>`. For most project, it consists into one single `gen-ninja.sh` file.

## External project

`ninja-to-soong` is able to take a external rust project file, compile it and link with it at runtime.

This is useful for project where the configuration file cannot be shared upstream for example, or when a project prefer to have the configuration file hosted in the project repository.

An example of such a configuration file can be found [here](tests/external-project/project.rs)

The important points are:
- Add `ninja-to-soong` crate: `extern crate ninja_to_soong;`, and use all modules needed for the project.
- Expose a `get_project` function without mangling:
```
#[no_mangle]
pub fn get_project() -> Box<dyn Project>
```

Then the project can be run with the following command:
```
<ninja-to-soong> $ cargo run --release -- --ext-proj-path <path_to_rust_file>
```

# Tests

`ninja-to-soong` uses github actions to check that changes do not bring regression. It checks that the generated files match their reference (located in the `tests` folder).

Each project in the `tests` folder contains the following files:
 * `Android.bp.n2s`: the reference file to generate
 * `checkout.sh`: a script to checkout the repository in the CI

Modification to `checkout.sh` or anything in the `scripts/<project>` directory trigger the generation of `Ninja` files in the CI, otherwise it uses the cached files from a previous CI run.

If you want more information take a look at the [github action script](.github/workflows/presubmit.yml)
