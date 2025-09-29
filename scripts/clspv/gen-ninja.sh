#!/usr/bin/env bash

set -xe

[ $# -eq 6 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"
SPIRV_HEADERS_PATH="$4"
SPIRV_TOOLS_PATH="$5"
LLVM_PROJECT_PATH="$6"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
bash "${SCRIPT_DIR}/../cmake_configure.sh" \
    "${SRC_PATH}" \
    "${BUILD_PATH}" \
    "${NDK_PATH}" \
    -DSPIRV_HEADERS_SOURCE_DIR="${SPIRV_HEADERS_PATH}" \
    -DSPIRV_TOOLS_SOURCE_DIR="${SPIRV_TOOLS_PATH}" \
    -DCLSPV_LLVM_SOURCE_DIR="${LLVM_PROJECT_PATH}/llvm" \
    -DCLSPV_CLANG_SOURCE_DIR="${LLVM_PROJECT_PATH}/clang" \
    -DCLSPV_LIBCLC_SOURCE_DIR="${LLVM_PROJECT_PATH}/libclc"
