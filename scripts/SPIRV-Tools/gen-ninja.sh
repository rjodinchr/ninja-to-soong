#!/usr/bin/env bash
# Copyright 2026 ninja-to-soong authors
# SPDX-License-Identifier: Apache-2.0

set -xe

[ $# -eq 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
SPIRV_HEADERS_PATH="$3"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
cmake -S "${SRC_PATH}" -B "${BUILD_PATH}" -G Ninja \
    -DSPIRV-Headers_SOURCE_DIR="${SPIRV_HEADERS_PATH}"
