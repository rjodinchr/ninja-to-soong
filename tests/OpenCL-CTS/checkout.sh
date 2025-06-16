#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-CTS 9fc0d23b4cfccd84be8927363a77107dc554de30 "${DEST}/external/OpenCL-CTS"
bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-Headers 72b006a96de1863f1e7c581b22daacaaf1acc598 "${DEST}/external/OpenCL-Headers"
