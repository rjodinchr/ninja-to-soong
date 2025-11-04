#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
DEST_DIR="${DEST}/vendor/google/graphics/mesa3d/desktop-intel"

bash "${SCRIPT_DIR}/../../checkout.sh" https://gitlab.freedesktop.org/mesa/mesa ab462ae6b7064ed63b8c94f31d5e3be60dcfede6 "${DEST_DIR}"

for patch in \
    "mesa-0719638dfdc.patch" \
    "mesa-d0de915c0c7.patch" \
    "mesa-ed0c18ae4aa.patch" \
    "mesa-a8ab696033e.patch" \
    "mesa-2f9fd1768ae.patch" \
    "mesa-c0f332f1cba.patch" \
    "mesa-a25e88cd84b.patch" \
    "mesa-c8b10b4512c.patch" \
    "mesa-be9e0f2f6a3.patch"
do
    git -C "${DEST_DIR}" apply "${SCRIPT_DIR}/../patches/${patch}"
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
