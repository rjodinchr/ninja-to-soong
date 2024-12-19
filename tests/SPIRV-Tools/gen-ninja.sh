#!/usr/bin/env bash

set -xe

[ $# -eq 6 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"
ANDROID_ABI="$4"
ANDROID_PLATFORM="$5"
SPIRV_HEADERS_PATH="$6"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
bash "${SCRIPT_DIR}/../../utils/cmake_configure.sh" \
    "${SRC_PATH}" \
    "${BUILD_PATH}" \
    "${NDK_PATH}" \
    "${ANDROID_ABI}" \
    "${ANDROID_PLATFORM}" \
    -DSPIRV-Headers_SOURCE_DIR="${SPIRV_HEADERS_PATH}"
