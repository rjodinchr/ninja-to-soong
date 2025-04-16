#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/SPIRV-Tools 4d2f0b40bfe290dea6c6904dafdf7fd8328ba346 "${DEST}/external/SPIRV-Tools"
