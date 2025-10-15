#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/llvm/llvm-project 891f002026df122b36813b9e1819769c94327503 "${DEST}/external/opencl/llvm-project"
