#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/SPIRV-Headers 04f10f650d514df88b76d25e83db360142c7b174 "${DEST}/external/SPIRV-Headers"
