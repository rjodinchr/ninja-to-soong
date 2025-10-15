#!/usr/bin/env bash

set -xe

[ $# -eq 2 ]

SRC_PATH="$1"
BUILD_PATH="$2"

cmake -G Ninja \
    -S "${SRC_PATH}" \
    -B "${BUILD_PATH}" \
    -DLIBCLC_TARGETS_TO_BUILD="clspv--;clspv64--"
