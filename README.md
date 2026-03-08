# C2Flowch - C to Flowchart (ANSI/ISO 5807)

**c2flowch** is a high-performance Rust library and CLI tool that converts C source code into clean, professional flowcharts adhering to ANSI and ISO 5807 standards.

## Architecture
The project is designed as a modular **Cargo Workspace**:
- **`c2flowch-core`**: A pure Rust library (no-std compatible in the future) containing the parser and renderers.
- **`c2flowch`**: A CLI wrapper for the core library.
- **WebAssembly (Planned)**: The core library is designed to be easily compiled to Wasm for web integration.

## Status: Alpha
Currently supports:
- **Standards Compliance**: Adheres to ANSI and ISO 5807 flowchart symbols.
- **Multiple Formats**: Renders as clean ASCII (for terminals) and professional SVG (for documentation).
- **Human-in-the-loop**: Uses an intermediate YAML representation for manual adjustments.
- **Advanced Logic**: Supports basic C syntax, nested branches, loops, and automatic subroutine discovery.
- **i18n**: Built-in support for English and Spanish labels.
- **UTF-8 Support**: Full support for multi-byte characters (ñ, á, etc.) with correct visual padding.

## Usage

### 1. Parse C to YAML IR
```bash
cargo run -- parse main.c -o flow.yaml --level 5 --lang es
```
- `--level 1-5`: Adjust detail level (5 is full line-by-line).
- `--lang en|es`: Set the language for labels (default: en).

### 2. Render YAML to ASCII or SVG
```bash
# Render as ASCII (default)
cargo run -- render flow.yaml --output diagram.txt

# Render as SVG
cargo run -- render flow.yaml --format svg --output diagram.svg
```

## Testing
To run the automated unit tests:
```bash
cargo test
```

## Features
- **ANSI Standard Shapes**: Boxes (Process), Diamonds (Decision), Parallelograms (I/O), Parenthesized (Terminator), Double-lined Rectangles (Predefined Process/Subroutine).
- **Professional SVG Export**: Native vector output with precise coordinate mapping, support for multiline headers (custom info), and automatic subroutine stacking.
- **Static i18n Support**: Automatically translates UI labels and C keywords (e.g., "Inicio", "Fin", "Si", "No") based on the `--lang` flag (currently supports English and Spanish).
- **Correct Padding**: Every node has symmetric 3-space padding, accounting for visual width of multi-byte characters like `ñ` and `í`.
- **Level of Detail**: Filter out variable declarations or focus only on function calls.
- **Human-in-the-loop**: The YAML intermediate format allows users to refine the diagram before final rendering.

## Project Management
For detailed information about the project vision, roadmap, and current state, please refer to [PROJECT.md](./PROJECT.md).

## To-Do
Please see the roadmap in [PROJECT.md](./PROJECT.md) for the current progress and upcoming features.
