#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/krrishnarraj/clpeak b2e647ffb8f42aa22ce4b0194d6ef6d16d5002b0 "${DEST}/external/clpeak"
