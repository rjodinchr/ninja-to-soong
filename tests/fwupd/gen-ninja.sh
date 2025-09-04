#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
MESON_LOCAL_PATH="${HOME}/.local/share/meson/cross"
AOSP_AMD64="aosp-amd64"
ANDROID_PLATFORM="35"
mkdir -p "${MESON_LOCAL_PATH}"
ANDROID_PLATFORM="${ANDROID_PLATFORM}" NDK_PATH="${NDK_PATH}" \
envsubst < "${SCRIPT_DIR}/${AOSP_AMD64}.template" > "${MESON_LOCAL_PATH}/${AOSP_AMD64}"

meson setup \
    --cross-file "${AOSP_AMD64}" \
    --prefix=/data/fwupd \
    --reconfigure \
    --wipe \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
