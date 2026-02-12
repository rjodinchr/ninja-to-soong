#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/KhronosGroup/OpenCL-ICD-Loader b1c57534df7ac82519b04606f51b71fb5d4053c3 "${DEST}/external/OpenCL-ICD-Loader"
