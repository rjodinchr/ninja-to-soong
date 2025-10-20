#!/usr/bin/env bash

set -xe

[ $# -eq 2 ]

SRC_PATH="$1"
BUILD_PATH="$2"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

cmake -G Ninja \
    -S "${SRC_PATH}" \
    -B "${BUILD_PATH}" \
    -DLIBCLC_CUSTOM_LLVM_TOOLS_BINARY_DIR="${SCRIPT_DIR}" \
    -DLIBCLC_TARGETS_TO_BUILD="clspv--;clspv64--"
