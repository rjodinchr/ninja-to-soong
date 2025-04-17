#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/llvm/llvm-project 7d172f96ff2c4c7cf5c428b79a3c18e067ce0079 "${DEST}/external/opencl/llvm-project"
