#!/usr/bin/env bash
# Install a pinned binaryen release (wasm-opt) on a Linux x86_64 CI runner.
#
# The Ubuntu apt package lags several major versions behind and does not
# understand the WebAssembly proposals used by the WASI build, so CI downloads
# the upstream release tarball and verifies it against the published checksum.
set -euo pipefail

BINARYEN_VERSION="version_132"
# SHA-256 of binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz, taken from the
# .sha256 asset of the GitHub release. Update it together with the version.
BINARYEN_SHA256="195ddc94f9bc89f45abdabb0b9eea86023d727ba90eac8b35b80f2544fc30572"
ARCHIVE="binaryen-${BINARYEN_VERSION}-x86_64-linux.tar.gz"
BASE_URL="https://github.com/WebAssembly/binaryen/releases/download/${BINARYEN_VERSION}"
INSTALL_DIR="${RUNNER_TEMP:-/tmp}/binaryen"

mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"
curl -fsSL --retry 3 -o "$ARCHIVE" "${BASE_URL}/${ARCHIVE}"
echo "${BINARYEN_SHA256}  ${ARCHIVE}" | sha256sum -c -
tar -xzf "$ARCHIVE"
rm -f "$ARCHIVE"

# Expose wasm-opt to later workflow steps; when run outside GitHub Actions
# just report where it landed.
if [ -n "${GITHUB_PATH:-}" ]; then
  echo "${INSTALL_DIR}/binaryen-${BINARYEN_VERSION}/bin" >> "$GITHUB_PATH"
else
  echo "wasm-opt installed to ${INSTALL_DIR}/binaryen-${BINARYEN_VERSION}/bin (add it to PATH)"
fi
"${INSTALL_DIR}/binaryen-${BINARYEN_VERSION}/bin/wasm-opt" --version
