#!/usr/bin/env bash

set -xe

[ $# -eq 4 ]

SRC_PATH="$1"
BUILD_PATH="$2"
MESA_CLC_PATH="$3"
NDK_PATH="$4"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
MESON_LOCAL_PATH="${HOME}/.local/share/meson/cross"
AOSP_AARCH64="aosp-aarch64"
ANDROID_PLATFORM="35"
mkdir -p "${MESON_LOCAL_PATH}"
ANDROID_PLATFORM="${ANDROID_PLATFORM}" NDK_PATH="${NDK_PATH}" \
envsubst < "${SCRIPT_DIR}/${AOSP_AARCH64}.template" > "${MESON_LOCAL_PATH}/${AOSP_AARCH64}"

PKG_CONFIG_PATH="${PKG_CONFIG_PATH}:${SCRIPT_DIR}" \
meson setup \
    --libdir lib64 \
    --sysconfdir=/system/vendor/etc \
    -Degl=disabled \
    -Dtools= \
    -Dvulkan-drivers=swrast \
    -Dgallium-drivers= \
    -Dvideo-codecs= \
    -Dgles1=disabled \
    -Dgles2=disabled \
    -Dopengl=false \
    -Dbuildtype=release \
    -Dallow-fallback-for=libdrm,perfetto \
    -Dstrip=true \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
