#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/llvm/llvm-project e68a20e0b7623738d6af736d3aa02625cba6126a "${DEST}/external/opencl/llvm-project"
