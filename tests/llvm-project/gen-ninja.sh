#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
bash "${SCRIPT_DIR}/../../utils/cmake_configure.sh" \
    "${SRC_PATH}" \
    "${BUILD_PATH}" \
    "${NDK_PATH}" \
    -DLLVM_ENABLE_PROJECTS="clang;libclc" \
    -DLIBCLC_TARGETS_TO_BUILD="clspv--;clspv64--" \
    -DLLVM_TARGETS_TO_BUILD=
