#!/usr/bin/env bash

set -xe

[ $# -eq 2 ]

SRC_PATH="$1"
BUILD_PATH="$2"

meson setup \
    -Dplatforms= \
    -Dglx=disabled \
    -Dtools= \
    -Dbuild-tests=false \
    -Dvulkan-drivers= \
    -Dgallium-drivers= \
    -Dgallium-rusticl=false \
    -Dgallium-va=auto \
    -Dgallium-xa=disabled \
    -Dbuildtype=release \
    -Dintel-clc=enabled \
    -Dstrip=true \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
meson compile -C "${BUILD_PATH}"
