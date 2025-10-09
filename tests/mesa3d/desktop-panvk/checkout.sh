#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"

bash "${SCRIPT_DIR}/../../checkout.sh" https://gitlab.freedesktop.org/mesa/mesa 29ccbb21f30553d6a39967ae013748063b73005a "${DEST}/vendor/google/graphics/mesa3d/desktop-panvk"

for patch in \
    "mesa-cb86341829d.patch" \
    "mesa-c8b10b4512c.patch" 
do
    git -C "${DEST}/vendor/google/graphics/mesa3d/desktop-panvk" apply "${SCRIPT_DIR}/../patches/${patch}"
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
