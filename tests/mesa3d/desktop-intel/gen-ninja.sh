#!/usr/bin/env bash

set -xe

[ $# -eq 4 ]

SRC_PATH="$1"
BUILD_PATH="$2"
MESA_CLC_PATH="$3"
NDK_PATH="$4"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
MESON_LOCAL_PATH="${HOME}/.local/share/meson/cross"
AOSP_X86_64="aosp-x86_64"
ANDROID_PLATFORM="35"
mkdir -p "${MESON_LOCAL_PATH}"
ANDROID_PLATFORM="${ANDROID_PLATFORM}" NDK_PATH="${NDK_PATH}" \
envsubst < "${SCRIPT_DIR}/${AOSP_X86_64}.template" > "${MESON_LOCAL_PATH}/${AOSP_X86_64}"

PATH="${MESA_CLC_PATH}:${PATH}" \
meson setup \
    --cross-file "${AOSP_X86_64}" \
    --libdir lib64 \
    --sysconfdir=/system/vendor/etc \
    -Dllvm=disabled \
    -Dglx=disabled \
    -Dgbm=disabled \
    -Degl=enabled \
    -Dplatform-sdk-version=${ANDROID_PLATFORM} \
    -Dandroid-stub=true \
    -Dplatforms=android \
    -Dperfetto=true \
    -Degl-lib-suffix=_mesa \
    -Dgles-lib-suffix=_mesa \
    -Dcpp_rtti=false \
    -Dtools= \
    -Dvulkan-drivers=intel \
    -Dgallium-drivers=iris \
    -Dgallium-rusticl=false \
    -Dgallium-va=disabled \
    -Dgallium-xa=disabled \
    -Dbuildtype=release \
    -Dmesa-clc=system \
    -Dintel-rt=enabled \
    -Dstrip=true \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
