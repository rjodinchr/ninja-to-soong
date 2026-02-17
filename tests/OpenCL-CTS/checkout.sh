#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/OpenCL-CTS 67fbbe4ee24a15eed6d8875b2540da31af515495 "${DEST}/external/OpenCL-CTS"
