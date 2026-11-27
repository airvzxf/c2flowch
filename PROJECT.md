# c2flowch: design and roadmap

## Goal

Convert C source code into clean, readable ANSI/ISO 5807 flowcharts:

- **Human in the loop:** an intermediate YAML that can be edited by hand
  before rendering.
- **Levels of detail:** from an executive summary (level 1) to every
  statement (level 5).
- **Clarity:** no crossing or tangled lines when the code is structured.
- **Standard symbols:** terminator, process, predefined process, data,
  preparation, decision.

## Design (version 2)

```
C source ──parse──▶ Program (IR, YAML) ──layout──▶ Diagram ──▶ text / SVG / PNG
```

1. **Parser** (`crates/core/src/parse`): tree-sitter C syntax tree →
   structured IR. Labels are built from the syntax tree (never by searching
   the source text), so strings, comments and names that contain other
   names cannot confuse it. Every statement kind is handled in any position
   (inside braces or not, after a label…); anything unknown is drawn as
   plain text and reported, never dropped silently.
2. **IR** (`crates/core/src/ir.rs`): a tree of statements (`process`, `io`,
   `call`, `if`, `while`, `for`, `switch`, `break`, `goto`…) with the final
   label of every symbol. It has no node ids, join nodes or layout hints, so
   editing it cannot break the drawing.
3. **Levels of detail** (`crates/core/src/detail.rs`): filtering and grouping
   of the IR.
4. **Layout** (`crates/core/src/layout`): every statement becomes a block
   with one entry at the top and one exit at the bottom; control structures
   are built from the blocks of their children, so the normal flow needs no
   search. Jumps (`break`, `continue`, `goto`) are routed last with an A*
   search on the cell grid that avoids symbols and labels and penalizes
   bends and crossings.
5. **Renderers** (`crates/core/src/render`): text and SVG draw the same
   cell-grid diagram; the CLI rasterizes the SVG to PNG with `resvg` and an
   embedded font.

The core library does no I/O, ready for a WebAssembly build.

## Roadmap

### Done in version 2

- [x] Structured IR, editable YAML without ids.
- [x] Parser on the syntax tree: braceless bodies, `continue`, `goto` and
      labels, early `return`, `switch` fall-through and grouped `case`
      labels, deterministic function order.
- [x] Five distinct levels of detail.
- [x] Recursive layout and jump routing; crossings drawn so that they cannot
      be mistaken for joins.
- [x] Warnings for what cannot be drawn faithfully.
- [x] Snapshot tests for every sample, layout invariants at every level, CLI
      tests.

### Next

- [ ] **WebAssembly:** a `c2flowch-wasm` crate exposing parse and render to
      the browser.
- [ ] **Global variables:** show them before the start of `main`.
- [ ] **Multiple files:** follow calls into other `.c` files of a project.
- [ ] **Long labels:** wrap long conditions instead of widening the diamond.
- [ ] **Text output:** an option for plain ASCII arrows and diamonds, for
      terminals whose font lacks the "Symbols for Legacy Computing" block.
- [ ] **Colored terminal output.**
