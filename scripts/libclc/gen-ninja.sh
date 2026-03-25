#!/usr/bin/env bash
# Copyright 2026 ninja-to-soong authors
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 2 ]

SRC_PATH="$1"
BUILD_PATH="$2"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

cmake -G Ninja \
    -S "${SRC_PATH}" \
    -B "${BUILD_PATH}" \
    -DLLVM_DIR="${SCRIPT_DIR}" \
    -DCMAKE_CLC_COMPILER="${SCRIPT_DIR}/clang" \
    -DLIBCLC_TARGETS_TO_BUILD="clspv--;clspv64--"
