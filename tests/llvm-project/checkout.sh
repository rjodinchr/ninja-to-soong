#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/llvm/llvm-project d43b29fc545d702b35b20802f92357bc4c4177fe "${DEST}/external/opencl/llvm-project"
