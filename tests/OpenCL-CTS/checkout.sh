#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-CTS 9bb8a89bbd86e485fcd6f452d05b0380292c35a1 "${DEST}/external/OpenCL-CTS"
bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-Headers 886dc9b7ff48366c2adca31d8587cf47169ba893 "${DEST}/external/OpenCL-Headers"
