#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/SPIRV-Tools fbe4f3ad913c44fe8700545f8ffe35d1382b7093 "${DEST}/external/SPIRV-Tools"
