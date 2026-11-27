#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# cloudflare-build.bash - Builds the browser demo (web/) on a machine that
# may lack the tools, such as the Cloudflare Pages build image.
#
# Cloudflare Pages settings: build command `bash scripts/cloudflare-build.bash`,
# build output directory `web`.
#
# Whatever is missing is downloaded under ~/.cache/c2flowch-tools:
# - Rust (rustup, minimal profile) and the wasm32-unknown-unknown target;
# - clang, from wasi-sdk, to compile the C grammar to WebAssembly;
# - wasm-bindgen, the prebuilt binary of the version in Cargo.lock.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"
cd "${PROJECT_ROOT}"

WASI_SDK_VERSION="34"
TOOLS="${HOME}/.cache/c2flowch-tools"
mkdir -p "${TOOLS}/bin"
export PATH="${TOOLS}/bin:${HOME}/.cargo/bin:${PATH}"

# Downloads URL and checks it against the sha256 at URL.sha256sum, when the
# release publishes one.
fetch() {
    local url="$1" out="$2"
    curl -fsSL --retry 3 -o "${out}" "${url}"
    if curl -fsSL --retry 3 -o "${out}.sha256sum" "${url}.sha256sum" 2>/dev/null; then
        echo "$(awk '{print $1}' "${out}.sha256sum")  ${out}" | sha256sum -c -
    fi
}

if ! command -v cc >/dev/null; then
    echo "error: no C compiler (cc) for the build scripts of Rust crates" >&2
    exit 1
fi

if ! command -v cargo >/dev/null; then
    echo "Installing Rust"
    curl -fsSL --retry 3 https://sh.rustup.rs |
        sh -s -- -y --no-modify-path --profile minimal --default-toolchain stable
fi
if command -v rustup >/dev/null; then
    rustup target add wasm32-unknown-unknown
fi

if ! command -v clang >/dev/null; then
    sdk="${TOOLS}/wasi-sdk-${WASI_SDK_VERSION}"
    if [ ! -x "${sdk}/bin/clang" ]; then
        echo "Installing clang (wasi-sdk ${WASI_SDK_VERSION})"
        name="wasi-sdk-${WASI_SDK_VERSION}.0-x86_64-linux"
        fetch "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_SDK_VERSION}/${name}.tar.gz" \
            "${TOOLS}/${name}.tar.gz"
        tar -xzf "${TOOLS}/${name}.tar.gz" -C "${TOOLS}"
        mv "${TOOLS}/${name}" "${sdk}"
        rm -f "${TOOLS}/${name}.tar.gz"*
    fi
    export CC_wasm32_unknown_unknown="${sdk}/bin/clang"
    export AR_wasm32_unknown_unknown="${sdk}/bin/llvm-ar"
fi

bindgen="$(awk '/^name = "wasm-bindgen"$/ {getline; gsub(/[^0-9.]/, ""); print}' Cargo.lock)"
if ! wasm-bindgen --version 2>/dev/null | grep -qx "wasm-bindgen ${bindgen}"; then
    echo "Installing wasm-bindgen ${bindgen}"
    name="wasm-bindgen-${bindgen}-x86_64-unknown-linux-musl"
    fetch "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${bindgen}/${name}.tar.gz" \
        "${TOOLS}/${name}.tar.gz"
    tar -xzf "${TOOLS}/${name}.tar.gz" -C "${TOOLS}"
    cp "${TOOLS}/${name}/wasm-bindgen" "${TOOLS}/bin/"
    rm -rf "${TOOLS}/${name}" "${TOOLS}/${name}.tar.gz"*
fi

bash scripts/build-wasm.bash web web/pkg
