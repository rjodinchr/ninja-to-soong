#!/usr/bin/env bash

set -xe

[ $# -ge 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"
shift 3
CMAKE_ARGS=$@

cmake -B "${BUILD_PATH}" -S "${SRC_PATH}" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_TOOLCHAIN_FILE="${NDK_PATH}/build/cmake/android.toolchain.cmake" \
    -DANDROID_ABI="arm64-v8a" \
    -DANDROID_PLATFORM="35" \
    ${CMAKE_ARGS}
