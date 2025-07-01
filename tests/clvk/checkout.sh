#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/rjodinchr/clvk 2b1084c13bfad719fdf4b1970318b9b293a993d9 "${DEST}/external/clvk"
