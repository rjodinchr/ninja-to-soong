#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/llvm/llvm-project 2e637dbbb8bc9a41f8eabd1df347ca2559b1abd7 "${DEST}/external/llvm-project"
