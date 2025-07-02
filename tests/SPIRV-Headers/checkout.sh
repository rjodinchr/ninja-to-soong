#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/SPIRV-Headers 7c2f5333e9c662620581361dffc327a99800bb52 "${DEST}/external/SPIRV-Headers"
