#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/SPIRV-Headers 3f17b2af6784bfa2c5aa5dbb8e0e74a607dd8b3b "${DEST}/external/SPIRV-Headers"
