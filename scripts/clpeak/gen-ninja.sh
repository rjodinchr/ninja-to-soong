#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
bash "${SCRIPT_DIR}/../cmake_configure.sh" \
    "${SRC_PATH}" \
    "${BUILD_PATH}" \
    "${NDK_PATH}" \
    -DOpenCL_LIBRARY="${BUILD_PATH}/sdk_install/lib/libOpenCL.so" \
    -DOpenCL_INCLUDE_DIR="${BUILD_PATH}/sdk_install/include/" \
    -DHPP_FOUND="${BUILD_PATH}/sdk_install/include"
