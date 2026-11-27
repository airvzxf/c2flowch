#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# build-wasm.bash - Builds the WebAssembly package of c2flowch.
#
# Usage: scripts/build-wasm.bash [web|nodejs|bundler] [OUT_DIR]
#
# Needs the wasm32-unknown-unknown Rust target, clang (it compiles the C
# grammar) and wasm-bindgen-cli of the same version as the wasm-bindgen
# crate in Cargo.lock:
#
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <version> --locked
#
# The default writes web/pkg and web/examples, which the demo page in web/
# loads: serve web/ with any static file server to try it.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"
cd "${PROJECT_ROOT}"

TARGET="${1:-web}"
OUT_DIR="${2:-web/pkg}"

# Flags such as -C target-cpu=native in a global Cargo configuration break
# the wasm-bindgen output, so the build uses none unless RUSTFLAGS is set.
export CARGO_ENCODED_RUSTFLAGS="${RUSTFLAGS:-}"

cargo build --release --locked -p c2flowch-wasm --target wasm32-unknown-unknown
wasm-bindgen --target "${TARGET}" --out-dir "${OUT_DIR}" \
    target/wasm32-unknown-unknown/release/c2flowch_wasm.wasm
if [ "${TARGET}" = "web" ]; then
    # The demo page loads its examples next to the package.
    mkdir -p "${OUT_DIR}/../examples"
    cp resources/c/*.c "${OUT_DIR}/../examples/"
fi
echo "Saved ${OUT_DIR}"
