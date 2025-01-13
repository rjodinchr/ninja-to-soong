#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../utils/checkout.sh" https://github.com/kpet/clvk 5c59aa58b782ab3e06a8c8727bac50dac7e4282c "${DEST}/external/clvk"
