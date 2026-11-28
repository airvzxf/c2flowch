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
C source --parse--> Program (IR, YAML) --layout--> Diagram --> text / SVG / PNG
```

1. **Parser** (`crates/core/src/parse`): tree-sitter C syntax tree ->
   structured IR. Labels are built from the syntax tree (never by searching
   the source text), so strings, comments and names that contain other
   names cannot confuse it. Every statement kind is handled in any position
   (inside braces or not, after a label...); anything unknown is drawn as
   plain text and reported, never dropped silently.
2. **IR** (`crates/core/src/ir.rs`): a tree of statements (`process`, `io`,
   `call`, `if`, `while`, `for`, `switch`, `break`, `goto`...) with the final
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
- [x] Colored terminal output: `render --color auto|always|never`.
- [x] Multiple files: `parse a.c b.c ...` follows calls into the other files.
- [x] WebAssembly: the `c2flowch-wasm` crate exposes parse and render to
      JavaScript, with a browser demo in `web/`.

### Next

Nothing planned; ideas and bug reports are welcome as issues.

## Pinned dependencies

`crates/wasm/Cargo.toml` pins `tree-sitter-language = "=0.1.7"`, and
`tree-sitter` stays on 0.26. Dependabot ignores both
(`.github/dependabot.yml`), so they are updated by hand.

- tree-sitter-c 0.24.2 compiles the C library shims of
  tree-sitter-language for `wasm32-unknown-unknown`.
- tree-sitter-language 0.1.8 and later ship those shims only for
  tree-sitter 0.27 and stop the build with tree-sitter 0.26.
- tree-sitter 0.27 needs tree-sitter-language 0.1.9 or later, and with
  it tree-sitter-c 0.24.2 does not build for wasm32 either (checked with
  tree-sitter 0.27.2 and tree-sitter-language 0.1.9).

Remove the pin when a tree-sitter-c release supports tree-sitter 0.27:
update `tree-sitter`, `tree-sitter-c` and `tree-sitter-language`
together, then check that `scripts/build-wasm.bash` and the CI
WebAssembly job pass, and remove the two `ignore` entries.
