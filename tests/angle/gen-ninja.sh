#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]

SRC_PATH="$1"
BUILD_PATH="$2"
ANDROID_CPU="$3"

cd "${SRC_PATH}"

gn gen "${BUILD_PATH}"
gn args "${BUILD_PATH}" \
    --list \
    --overrides-only \
    --short \
    --args="target_cpu=\"${ANDROID_CPU}\" target_os=\"android\" \
    is_component_build=false \
    is_debug=false \
    dcheck_always_on=false \
    symbol_level=0 \
    angle_standalone=false \
    angle_build_all=false \
    angle_expose_non_conformant_extensions_and_versions=true \
    android32_ndk_api_level=26 \
    android64_ndk_api_level=26 \
    angle_enable_vulkan=true \
    angle_enable_gl=false \
    angle_enable_d3d9=false \
    angle_enable_d3d11=false \
    angle_enable_null=false \
    angle_enable_metal=false \
    angle_enable_wgpu=false \
    angle_enable_swiftshader=false \
    angle_enable_essl=false \
    angle_enable_glsl=false \
    angle_enable_hlsl=false \
    angle_enable_commit_id=false \
    angle_has_histograms=false \
    use_custom_libcxx=false \
    angle_has_rapidjson=false \
    build_angle_end2end_tests_aosp=true \
    build_angle_trace_tests=false \
    angle_test_enable_system_egl=true"
gn gen "${BUILD_PATH}"
