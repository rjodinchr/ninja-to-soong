#!/usr/bin/env bash

set -xe

[ $# -eq 3 ]

REPO="$1"
VERSION="$2"
DEST="$3"

git init "${DEST}"
git -C "${DEST}" remote add origin "${REPO}"
git -C "${DEST}" fetch --depth 1 origin "${VERSION}"
git -C "${DEST}" checkout "${VERSION}"
