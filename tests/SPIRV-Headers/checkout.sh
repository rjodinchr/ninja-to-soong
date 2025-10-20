#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/SPIRV-Headers 01e0577914a75a2569c846778c2f93aa8e6feddd "${DEST}/external/SPIRV-Headers"
