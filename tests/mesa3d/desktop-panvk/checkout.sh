#!/usr/bin/env bash

set -xe

[ $# -eq 1 ]
DEST="$1"
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
DEST_DIR="${DEST}/vendor/google/graphics/mesa3d/desktop-panvk"

bash "${SCRIPT_DIR}/../../checkout.sh" https://gitlab.freedesktop.org/mesa/mesa ad3759f342ee988929ef32022d5486f99484221f "${DEST_DIR}"

for patch in \
    "mesa-762be5eae1e.patch"
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
