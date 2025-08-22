#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-ICD-Loader 02134b05bdff750217bf0c4c11a9b13b63957b04 "${DEST}/external/OpenCL-ICD-Loader"
bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-Headers 72b006a96de1863f1e7c581b22daacaaf1acc598 "${DEST}/external/OpenCL-Headers"
