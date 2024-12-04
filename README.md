# Ninja to Soong [![CI badge](https://github.com/rjodinchr/ninja-to-soong/actions/workflows/presubmit.yml/badge.svg?branch=main)](https://github.com/rjodinchr/ninja-to-soong/actions/workflows/presubmit.yml?query=branch%3Amain++)

`ninja-to-soong` is a project to generate `Soong` files (`Android.bp`) for the Android build system.

# Legal

`ninja-to-soong` is licensed under the terms opf the [Apache 2.0 license](LICENSE)

# How does it work?

1. `ninja-to-soong` generates `Ninja` files using `CMake` and the Android NDK.
2. `ninja-to-soong` generates `Soong` files using `Ninja` files.

# Supported projects

* [clvk](https://github.com/kpet/clvk) and all its submodules (within the of limit of clvk's requirement)
  * [clspv](https://github.com/google/clspv)
    * [llvm-project](https://github.com/llvm/llvm-project)
  * [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools)
  * [SPIRV-Headers](https://github.com/KhronosGroup/SPIRV-Headers)

# Dependencies

`ninja-to-soong` depends on the following:

* [Rust](https://www.rust-lang.org/)
* [Android NDK](https://developer.android.com/ndk)
* [Ninja](https://ninja-build.org/)
* [CMake](https://cmake.org/)

# Using `ninja-to-soong`

```
<ninja-to-soong> $ cargo run --release -- <android_repository_path> <android_ndk_path>
```

# Tests

`ninja-to-soong` uses github actions to check that changes do not bring regression. It checks that the generated files match their reference (located in the `tests` folder).

Each project in the `tests` folder contains the following files:
 * `Android.bp`: the reference file to generate
 * `REPO`: a file containing the git URL of the project
 * `VERSION`: a file containing the `sha1` to use to checkout the project

 To reduce CI time, the environment variable `NINJA_TO_SOONG_SKIP_CMAKE_BUILD` is set to avoid building projects. While it is correct to do for test purpose, it means that things will be missing when trying to update certain project (e.g. `llvm-project`).

If you want more information take a look at the [github action script](.github/workflows/presubmit.yml)

# Developement tips

After a **full** first run of `ninja-to-soong`, it is possible to run with the environment variable `NINJA_TO_SOONG_SKIP_CMAKE_CONFIGURE` set to skip the cmake configure step for every project.

It is possible to run a specific set of projects by adding them after the required arguments:
```
<ninja-to-soong> $ cargo run -- <android_tree_root> <android_ndk_path> <project1> <project2>
```
