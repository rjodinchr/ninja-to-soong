#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/llvm/llvm-project 5fa59edfa73a69ab146d7b9cc115de5770d11dca "${DEST}/external/llvm-project"
