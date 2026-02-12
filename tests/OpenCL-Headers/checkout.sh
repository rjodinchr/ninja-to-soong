#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/OpenCL-Headers dfb5284280a7a0bce6a21c057184ef653d9e21c1 "${DEST}/external/OpenCL-Headers"
