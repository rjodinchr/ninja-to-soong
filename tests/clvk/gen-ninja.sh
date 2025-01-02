#!/usr/bin/env bash

set -xe

[ $# -eq 7 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"
SPIRV_HEADERS_PATH="$4"
SPIRV_TOOLS_PATH="$5"
LLVM_PROJECT_PATH="$6"
CLSPV_PATH="$7"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
bash "${SCRIPT_DIR}/../../utils/cmake_configure.sh" \
    "${SRC_PATH}" \
    "${BUILD_PATH}" \
    "${NDK_PATH}" \
    -DSPIRV_HEADERS_SOURCE_DIR="${SPIRV_HEADERS_PATH}" \
    -DSPIRV_TOOLS_SOURCE_DIR="${SPIRV_TOOLS_PATH}" \
    -DCLSPV_LLVM_SOURCE_DIR="${LLVM_PROJECT_PATH}/llvm" \
    -DCLSPV_CLANG_SOURCE_DIR="${LLVM_PROJECT_PATH}/clang" \
    -DCLSPV_LIBCLC_SOURCE_DIR="${LLVM_PROJECT_PATH}/libclc" \
    -DCLSPV_SOURCE_DIR="${CLSPV_PATH}" \
    -DVulkan_LIBRARY="${NDK_PATH}/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/35/libvulkan.so" \
    -DCLVK_CLSPV_ONLINE_COMPILER=1 \
    -DCLVK_ENABLE_SPIRV_IL=OFF \
    -DCLVK_BUILD_TESTS=OFF
