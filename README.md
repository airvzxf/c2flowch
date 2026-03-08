# C2Flowch - C to Flowchart (ANSI/ASCII)

Convert C source code into clean, readable ASCII flowcharts with an intermediate YAML representation.

## Status: Alpha
Currently supports:
- Basic C syntax (declarations, assignments, if/else, do/while, for).
- Subroutines and function calls (automatic sub-diagram generation).
- Intermediate YAML representation for manual adjustment.
- ASCII and professional SVG rendering with a lane-based layout.
- High-precision vector export with intelligent label alignment and multiline titles.
- Symmetric 3-space padding and full support for Spanish characters (UTF-8).

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
