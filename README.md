# Ninja to Soong [![CI badge](https://github.com/rjodinchr/ninja-to-soong/actions/workflows/presubmit.yml/badge.svg?branch=main)](https://github.com/rjodinchr/ninja-to-soong/actions/workflows/presubmit.yml?query=branch%3Amain++)

`ninja-to-soong` is a project to generate `Soong` files (`Android.bp`) for the Android build system.

# Legal

`ninja-to-soong` is licensed under the terms opf the [Apache 2.0 license](LICENSE)

# How does it work?

1. `ninja-to-soong` generates [Ninja](https://ninja-build.org/) files using either:
    - [CMake](https://cmake.org/) and the Android NDK
    - [GN](https://github.com/o-lim/generate-ninja) cross-compiling for [Android](https://gn.googlesource.com/gn/+/HEAD/docs/quick_start.md#cross_compiling-to-a-target-os-or-architecture)
2. `ninja-to-soong` generates `Soong` files using [Ninja](https://ninja-build.org/) files.

# Supported projects

| Project | Ninja Generator | Targets |
|-|-|-|
| [clvk](https://github.com/kpet/clvk) | `CMake` | `libclvk.so` |
| [clspv](https://github.com/google/clspv) | `CMake` | `clvk` dependencies |
| [llvm-project](https://github.com/llvm/llvm-project) | `CMake` | `clvk` & `clspv` dependencies |
| [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools) | `CMake` | `clvk` dependencies |
| [SPIRV-Headers](https://github.com/KhronosGroup/SPIRV-Headers) | `CMake` | `clspv` & `SPIRV-Tools` dependencies |
| [angle](https://github.com/google/angle) (WIP) | `GN` | `libEGL_angle.so`, `libGLESv2_angle.so`, `libGLESv1_CM_angle.so` |

# Dependencies

`ninja-to-soong` depends on the following:

* [Rust](https://www.rust-lang.org/)
* [Ninja](https://ninja-build.org/)
* [CMake](https://cmake.org/)
* [GN](https://github.com/o-lim/generate-ninja)
* [wget](https://www.gnu.org/software/wget/)
* `unzip`

# Using `ninja-to-soong`

```
<ninja-to-soong> $ cargo run --release -- --aosp-path <android_tree_path> <project1> <project2>
```

## Options

* `--angle-path`: Path to angle source repository (required only for `angle` project)
* `--aosp-path`: Path to Android tree (required for most project)
* `--clean-tmp`: Remove the temporary directory before running
* `--copy-to-aosp`: Copy generated Soong files into the Android tree
* `--skip-build`: Skip build step
* `--skip-gen-ninja`: Skip generation of Ninja files
* `-h`, `--help`: Display the help and exit

## Environment variables

* `N2S_NDK`: Android NDK (default: `android-ndk-r27c`)
* `N2S_NDK_PATH`: Path to Android NDK (default: temporary directory)
* `N2S_TMP_PATH`: Path used by `ninja-to-soong` to store its temporary directories (default: `std::env::temp_dir()`)

# Tests

`ninja-to-soong` uses github actions to check that changes do not bring regression. It checks that the generated files match their reference (located in the `tests` folder).

Each project in the `tests` folder contains the following files:
 * `Android.bp`: the reference file to generate
 * `checkout.sh`: a script to checkout the repository in the CI

If you want more information take a look at the [github action script](.github/workflows/presubmit.yml)
