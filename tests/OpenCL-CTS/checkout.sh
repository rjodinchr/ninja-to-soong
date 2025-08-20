#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-CTS 909095f60a45d2ea131586a8a06411b3072a1bdd "${DEST}/external/OpenCL-CTS"
bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-Headers 72b006a96de1863f1e7c581b22daacaaf1acc598 "${DEST}/external/OpenCL-Headers"
