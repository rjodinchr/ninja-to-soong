#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-ICD-Loader ad770a1b64c6b8d5f2ed4e153f22e4f45939f27f "${DEST}/external/OpenCL-ICD-Loader"
bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/KhronosGroup/OpenCL-Headers 72b006a96de1863f1e7c581b22daacaaf1acc598 "${DEST}/external/OpenCL-Headers"
