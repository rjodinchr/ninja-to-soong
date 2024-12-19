#!/usr/bin/env bash

set -xe

[ $# -ge 5 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"
ANDROID_ABI="$4"
ANDROID_PLATFORM="$5"
shift 5
CMAKE_ARGS=$@

cmake -B "${BUILD_PATH}" -S "${SRC_PATH}" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_TOOLCHAIN_FILE="${NDK_PATH}/build/cmake/android.toolchain.cmake" \
    -DANDROID_ABI="${ANDROID_ABI}" \
    -DANDROID_PLATFORM="${ANDROID_PLATFORM}" \
    ${CMAKE_ARGS}
