#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
DEPOT_TOOLS="${DEST}/depot_tools"
ANGLE="${DEST}/external/angle"

git clone --depth 1 https://chromium.googlesource.com/chromium/tools/depot_tools.git "${DEPOT_TOOLS}"
export PATH="${DEPOT_TOOLS}:$PATH"
mkdir -p "${ANGLE}"
cd "${ANGLE}"
fetch --no-history angle
echo "target_os = [\"android\"]" >> .gclient
gclient sync --no-history --shallow --revision=b0d686ae30e51e4376297261f3984922d4e7d8da
sudo ./build/install-build-deps.sh
