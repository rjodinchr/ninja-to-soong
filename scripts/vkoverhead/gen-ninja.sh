#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
NDK_PATH="$3"

SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
MESON_LOCAL_PATH="${HOME}/.local/share/meson/cross"
AOSP_X86_64="aosp-x86_64"
ANDROID_PLATFORM="35"
mkdir -p "${MESON_LOCAL_PATH}"
ANDROID_PLATFORM="${ANDROID_PLATFORM}" NDK_PATH="${NDK_PATH}" \
envsubst < "${SCRIPT_DIR}/${AOSP_X86_64}.template" > "${MESON_LOCAL_PATH}/${AOSP_X86_64}"

meson setup \
    --cross-file "${AOSP_X86_64}" \
    --reconfigure \
    --wipe \
    -Dplatforms=android \
    -Dplatform-sdk-version=${ANDROID_PLATFORM} \
    "${BUILD_PATH}" \
    "${SRC_PATH}"
