#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../../utils/checkout.sh" https://gitlab.freedesktop.org/mattst88/mesa.git a69c558f155a62b79bd6d1f8a2c800296f86afc2 "${DEST}/vendor/google/graphics/mesa3d/desktop-intel"

MAJOR_VERSION=17
sudo apt install \
     meson-1.5 \
     libclang-${MAJOR_VERSION}-dev \
     libclang-cpp${MAJOR_VERSION}{,-dev} \
     libclc-${MAJOR_VERSION}{,-dev} \
     libllvmspirvlib-${MAJOR_VERSION}-dev \
     llvm-${MAJOR_VERSION}-{dev,tools} \
     glslang-tools \
     python3-{mako,ply}
