#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/kpet/clvk 03c8a4092f9d79aac8e2817e753168c6adef43e6 "${DEST}/external/clvk"
