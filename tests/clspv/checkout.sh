#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/google/clspv 1bd9b3fd84a9ef39033933073f2ea88f114134df "${DEST}/external/clspv"
