#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/rjodinchr/clvk d5304bc4f3735cb5b3eb6a6a180faab4708b3150 "${DEST}/external/clvk"
