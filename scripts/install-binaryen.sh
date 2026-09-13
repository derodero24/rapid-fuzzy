#!/usr/bin/env bash
# Install a pinned binaryen release (wasm-opt) on a Linux x86_64 CI runner.
#
# The Ubuntu apt package lags several major versions behind and does not
# understand the WebAssembly proposals used by the WASI build, so CI downloads
# the upstream release tarball and verifies it against the published checksum.
set -euo pipefail

BINARYEN_VERSION="version_132"
ARCHIVE="binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz"
BASE_URL="https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}"
INSTALL_DIR="${RUNNER_TEMP:-/tmp}/binaryen"

mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"
curl -fsSL --retry 3 -o "$ARCHIVE" "${BASE_URL}/${ARCHIVE}"
curl -fsSL --retry 3 -o "${ARCHIVE}.sha256" "${BASE_URL}/${ARCHIVE}.sha256"
sha256sum -c "${ARCHIVE}.sha256"
tar -xzf "$ARCHIVE"
rm -f "$ARCHIVE" "${ARCHIVE}.sha256"

echo "${INSTALL_DIR}/binaryen-${BINARYEN_VERSION}/bin" >> "$GITHUB_PATH"
"${INSTALL_DIR}/binaryen-${BINARYEN_VERSION}/bin/wasm-opt" --version
