# Ninja to Soong

`ninja-to-soong` is a project to generate `Android.bp` files for the Android build system (`Soong`). Those files are generated from `Ninja` file generated using the Android NDK.

# Legal

`ninja-to-soong` is licensed under the terms opf the [Apache 2.0 license](LICENSE)

# Dependencies

`ninja-to-soong` depends on the following:

* [Rust](https://www.rust-lang.org/) & [cargo](https://doc.rust-lang.org/cargo/)
* [Android NDK](https://developer.android.com/ndk)
* [Ninja](https://ninja-build.org/)

# Supported projects

* [clvk](https://github.com/kpet/clvk) and all its submodules (within the of limit of clvk's requirement)
  * [clspv](https://github.com/google/clspv)
    * [llvm-project](https://github.com/llvm/llvm-project)
  * [SPIRV-Tools](https://github.com/KhronosGroup/SPIRV-Tools)
  * [SPIRV-Headers](https://github.com/KhronosGroup/SPIRV-Headers)

# Using `ninja-to-soong`

```
<ninja-to-soong> $ cargo run -- <android_tree_root> <android_ndk_path> all
```
