# C2Flowch - C to Flowchart (ANSI/ASCII)

Convert C source code into clean, readable ASCII flowcharts with an intermediate YAML representation.

## Status: Alpha
Currently supports:
- Basic C syntax (declarations, assignments, if/else, do/while, for).
- Intermediate YAML representation for manual adjustment.
- ASCII rendering with a lane-based layout.
- Symmetric 3-space padding and full support for Spanish characters (UTF-8).

## Usage

### 1. Parse C to YAML IR
```bash
./target/debug/c2flowch parse --level 5 main.c -o flow.yaml
```
- `--level 1-5`: Adjust detail level (5 is full line-by-line).

### 2. Render YAML to ASCII
```bash
./target/debug/c2flowch render flow.yaml --output diagram.txt
```

## Features
- **ANSI Standard Shapes**: Boxes (Process), Diamonds (Decision), Parallelograms (I/O), Rounded Rects (Terminator).
- **Correct Padding**: Every node has symmetric 3-space padding, accounting for visual width of multi-byte characters like `ñ` and `í`.
- **Level of Detail**: Filter out variable declarations or focus only on function calls.
- **Human-in-the-loop**: The YAML intermediate format allows users to refine the diagram before final rendering.

## To-Do
- [ ] Refine branch routing for complex nested logic.
- [ ] Add SVG export.
- [ ] Formal i18n support.
- [ ] Support for global variables.
