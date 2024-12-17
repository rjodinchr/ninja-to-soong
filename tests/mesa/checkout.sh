#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://gitlab.freedesktop.org/mattst88/mesa d5ec20187041cbcf1562f4c77a25fe91b8ce0e74 "${DEST}/external/mesa"
