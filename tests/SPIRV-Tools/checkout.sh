#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/SPIRV-Tools 7f2d9ee926f98fc77a3ed1e1e0f113b8c9c49458 "${DEST}/external/SPIRV-Tools"
