#!/usr/bin/env bash

set -xe

[ $# -eq 2 ]

SRC_PATH="$1"
BUILD_PATH="$2"

meson setup \
    -Dplatforms= \
    -Dvulkan-drivers= \
    -Dgallium-drivers= \
    -Dbuildtype=release \
    -Dmesa-clc=enabled \
    -Dinstall-mesa-clc=true \
    -Dtools=panfrost \
    -Dstrip=true \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
