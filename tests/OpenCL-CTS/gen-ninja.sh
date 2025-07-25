#!/usr/bin/env bash

set -xe

[ $# -eq 4 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"
SPIRV_TOOLS_PATH="$4"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

PATH="${SPIRV_TOOLS_PATH}:${PATH}" \
bash "${SCRIPT_DIR}/../../utils/cmake_configure.sh" \
    "${SRC_PATH}" \
    "${BUILD_PATH}" \
    "${NDK_PATH}" \
    -DCL_INCLUDE_DIR="${SRC_PATH}/../OpenCL-Headers" \
    -DCL_LIB_DIR="${SRC_PATH}" \
    -DOPENCL_LIBRARIES="-lOpenCL"
