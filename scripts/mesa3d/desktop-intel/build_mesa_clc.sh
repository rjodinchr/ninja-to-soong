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
    -Dbuildtype=release \
    -Dmesa-clc=enabled \
    -Dinstall-mesa-clc=true \
    -Dintel-elk=false \
    -Dstrip=true \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
meson compile -C "${BUILD_PATH}"

mkdir -p "${BUILD_PATH}/bin"
cp "${BUILD_PATH}/src/compiler/clc/mesa_clc"       "${BUILD_PATH}/bin"
cp "${BUILD_PATH}/src/compiler/spirv/vtn_bindgen2" "${BUILD_PATH}/bin"
