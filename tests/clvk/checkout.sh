#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/kpet/clvk 15a4052a79b94484e34e3391d36762c4ef8f31b9 "${DEST}/external/clvk"
