#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../checkout.sh" https://gitlab.freedesktop.org/mesa/mesa 7fc153c93fd5e44643dc53a7b5189896b67aed2b "${DEST}/vendor/google/graphics/mesa3d/desktop-intel"

for patch in \
    "mesa-0719638dfdc.patch" \
    "mesa-d0de915c0c7.patch" \
    "mesa-ed0c18ae4aa.patch" \
    "mesa-a8ab696033e.patch" \
    "mesa-2f9fd1768ae.patch" \
    "mesa-c8b10b4512c.patch" 
do
    git -C "${DEST}/vendor/google/graphics/mesa3d/desktop-intel" apply "${SCRIPT_DIR}/../patches/${patch}"
done

MAJOR_VERSION=19
sudo apt install \
     meson-1.5 \
     libclang-${MAJOR_VERSION}-dev \
     libclang-cpp${MAJOR_VERSION}{,-dev} \
     libclc-${MAJOR_VERSION}{,-dev} \
     libllvmspirvlib-${MAJOR_VERSION}-dev \
     llvm-${MAJOR_VERSION}-{dev,tools} \
     glslang-tools \
     python3-{mako,ply}
