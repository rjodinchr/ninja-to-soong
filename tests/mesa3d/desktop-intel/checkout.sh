#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../checkout.sh" https://gitlab.freedesktop.org/mesa/mesa a30a465cba30e2c1aea3b153792910cf891f3aeb "${DEST}/vendor/google/graphics/mesa3d/desktop-intel"

for patch in \
    "mesa-a8ab696033e.patch" \
    "mesa-2f9fd1768ae.patch" \
    "mesa-cb86341829d.patch" \
    "mesa-MR-37742.patch" \
    "mesa-MR-37785.patch" \
    "mesa-MR-37789.patch"
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
