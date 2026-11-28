#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# pipeline.bash - Turns every C sample into flowcharts, in parallel
#
set -euo pipefail

# Run from the project root, wherever the script is called from
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"
cd "${PROJECT_ROOT}"

# Paths
C_DIR="resources/c"
YAML_DIR="resources/yaml"
ASCII_DIR="resources/ascii"
ASCII_SPLIT_DIR="resources/ascii-split"
SVG_DIR="resources/svg"
SVG_SPLIT_DIR="resources/svg-split"
PNG_DIR="resources/png"
PNG_SPLIT_DIR="resources/png-split"
BIN="./target/release/c2flowch"

# Processes one C file
process_file() {
    local c_file="${1}"
    local base_name
    base_name=$(basename "${c_file}" .c)

    local yaml_output="${YAML_DIR}/${base_name}.yaml"
    local ascii_output="${ASCII_DIR}/${base_name}.txt"
    local ascii_split_output="${ASCII_SPLIT_DIR}/${base_name}.txt"
    local svg_output="${SVG_DIR}/${base_name}.svg"
    local svg_split_output="${SVG_SPLIT_DIR}/${base_name}.svg"
    local png_output="${PNG_DIR}/${base_name}.png"
    local png_split_output="${PNG_SPLIT_DIR}/${base_name}.png"

    echo "Processing: ${base_name}"

    # 1. Parse: C -> YAML
    if "${BIN}" parse "${c_file}" --output "${yaml_output}" --lang es > /dev/null 2>&1; then
        # Render: YAML -> the other formats

        # ASCII
        "${BIN}" render "${yaml_output}" --output "${ascii_output}" > /dev/null 2>&1
        "${BIN}" render "${yaml_output}" --output "${ascii_split_output}" --split-subroutines > /dev/null 2>&1

        # SVG
        "${BIN}" render "${yaml_output}" --format svg --output "${svg_output}" > /dev/null 2>&1
        "${BIN}" render "${yaml_output}" --format svg --output "${svg_split_output}" --split-subroutines > /dev/null 2>&1

        # PNG
        if "${BIN}" render "${yaml_output}" --format png --output "${png_output}" > /dev/null 2>&1; then
            echo "  [OK] PNG (Std): ${base_name}"
        else
            echo "  [ERROR] PNG (Std): ${base_name}"
            return 1
        fi

        # PNG (Split)
        if "${BIN}" render "${yaml_output}" --format png --output "${png_split_output}" --split-subroutines > /dev/null 2>&1; then
            echo "  [OK] PNG (Split): ${base_name}"
        else
            echo "  [ERROR] PNG (Split): ${base_name}"
            return 1
        fi
    else
        echo "  [ERROR] Parsing failed: ${c_file}"
        return 1
    fi
}

# Export the variables and the function for the bash subshells started by xargs
export -f process_file
export BIN YAML_DIR ASCII_DIR ASCII_SPLIT_DIR SVG_DIR SVG_SPLIT_DIR PNG_DIR PNG_SPLIT_DIR

# With --single, process one file only (xargs calls the script this way)
if [[ "${1:-}" == "--single" ]]; then
    process_file "${2}"
    exit 0
fi

# --- Main ---

# Recreate the output directories
rm -fR "${YAML_DIR}" "${ASCII_DIR}" "${ASCII_SPLIT_DIR}" "${SVG_DIR}" "${SVG_SPLIT_DIR}" "${PNG_DIR}" "${PNG_SPLIT_DIR}"
mkdir -p "${YAML_DIR}" "${ASCII_DIR}" "${ASCII_SPLIT_DIR}" "${SVG_DIR}" "${SVG_SPLIT_DIR}" "${PNG_DIR}" "${PNG_SPLIT_DIR}"

echo "=== Building c2flowch (release) ==="
cargo build --release --quiet

echo "=== Running the c2flowch pipeline (parallel) ==="
CORES=$(nproc 2>/dev/null || echo 4)
echo "Using ${CORES} cores..."

# Total time
start_time=$(date +%s)

# Find the .c files and process them in parallel with xargs
find "${C_DIR}" -name "*.c" -print0 | xargs -0 -I {} -P "${CORES}" bash "${0}" --single "{}"

end_time=$(date +%s)
duration=$((end_time - start_time))

echo "------------------------------------------------"
echo "=== Finished in ${duration} seconds ==="
