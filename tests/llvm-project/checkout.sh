#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/llvm/llvm-project e0a6905287050d57ea0413cba7f011803b1f65ef "${DEST}/external/opencl/llvm-project"
