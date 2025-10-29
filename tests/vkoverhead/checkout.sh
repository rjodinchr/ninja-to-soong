#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../checkout.sh" https://github.com/zmike/vkoverhead.git 62c27e9439ca1ddf52b3f3ea119237d8aea688f7 "${DEST}/external/vkoverhead"
