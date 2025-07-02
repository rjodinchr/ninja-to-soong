#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/SPIRV-Tools a871fc43e29038d96109a64a64219eacefdf0634 "${DEST}/external/SPIRV-Tools"
