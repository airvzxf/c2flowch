# Changelog

## Unreleased

### Changed

- PNG output uses resvg 0.50, which shapes and reads fonts with
  harfrust and skrifa instead of the unmaintained rustybuzz and
  ttf-parser (RUSTSEC-2026-0206, RUSTSEC-2026-0192). The 36 samples
  render to the same PNG bytes as with resvg 0.47.

### Security

- memmap2 0.9.12 and rustix 1.1.6 fix RUSTSEC-2026-0186 and
  RUSTSEC-2026-0486.
- CI runs `cargo audit --deny warnings` on every pull request, on
  `main` and once a week.

## 2.5.2

### Added

- Links to the demo show a preview image: a C program next to its
  flowchart.

### Security

- The demo sends a Content-Security-Policy and other security headers
  (`web/src/_headers`): it runs only its own scripts and WebAssembly,
  fetches only its own files and cannot be framed.

### Changed

- Dependencies updated: anyhow 1.0.105, clap 4.6.10, serde 1.0.229,
  unicode-width 0.2.3, assert_cmd 2.2.2 and tempfile 3.27.0.
- Dependabot opens weekly pull requests for GitHub Actions and Cargo
  dependencies.

## 2.5.1

### Added

- `scripts/cloudflare-build.bash` builds the browser demo on a machine
  that only has `cc`, `curl` and `tar`, such as the Cloudflare Pages
  build image. The demo is online at <https://c2flowch.rovisoft.net/>.
- The demo has an icon, a description and Open Graph tags for link
  previews, and a "Page not found" page for unknown paths.
- The release workflow checks that <https://c2flowch.rovisoft.net/>
  serves the released version.

### Fixed

- Browsers and caches no longer keep old files of the demo after a
  deployment: every asset URL carries `?v=<version>-<commit>`.

## 2.5.0

### Added

- WebAssembly: the new `c2flowch-wasm` crate exposes `parse`,
  `parseFiles`, `render` (ASCII or SVG), `validate` and `version` to
  JavaScript through wasm-bindgen. They work on the YAML text of a
  program, like the command line. `scripts/build-wasm.bash` builds the
  package for the web, Node.js or bundlers, and every release attaches
  `c2flowch-wasm-vX.Y.Z.tar.gz`.
- A browser demo in `web/`: type C code, choose the level of detail and
  the language, see the flowchart as SVG or text, edit the YAML, and
  download the result. Everything runs in the browser.
- CI builds the WebAssembly package and checks that it draws every
  sample in `resources/c` exactly like the command line.

### Changed

- tree-sitter-c 0.24.2, which builds for `wasm32-unknown-unknown`.

## 2.4.0

### Added

- Multiple files: `parse main.c list.c util.c` reads several C files of
  one program into a single YAML. A call to a function defined in any
  of the files is a call symbol, every function gets its own flowchart
  (`main` first, then what it calls, then the rest in file order), and
  the global variables of every file are drawn in `main`. Warnings name
  the file they belong to. When two files define a function with the
  same name, the first one is kept, with a warning. The YAML goes next
  to the first file unless `-o` is given; its title lists the files.
  The library adds `parse_files` and `Source`, and `Warning` gains a
  `file` field.

### Changed

- `extern` variable declarations are no longer drawn as globals: they
  only refer to a variable defined elsewhere.

## 2.3.0

### Added

- Colored text output: `render --color auto|always|never`. Symbol
  outlines are colored by kind (terminators green, processes blue,
  calls cyan, input/output magenta, preparations red, decisions
  yellow); arrows and branch labels are bold. `auto`, the default,
  colors only when the text goes to a terminal and `NO_COLOR` is not
  set; `always` also colors into a file. Without color the output is
  byte for byte the same as before. The library adds
  `render_ascii_color`.

## 2.2.0

### Added

- Global variables: declarations outside of any function are kept in a
  new optional `globals` list of the YAML and drawn at the start of
  `main`, right after its start terminator. They follow `--level` like
  local declarations: a global with a value is a process from level 3,
  one without a value is a preparation from level 4. Function prototypes
  are still ignored, and YAML files without `globals` read as before.

## 2.1.0

### Added

- Long conditions wrap: a decision whose condition is wider than 30
  cells splits it at spaces into up to three lines, on the middle row and
  the rows above and below it, instead of growing one very wide diamond.
  Text and SVG output both wrap the same way; a condition without spaces
  stays on one line.

### Fixed

- `render --help` described the text format as Unicode box drawing; it is
  plain ASCII since 2.0.0.

## 2.0.1

### Changed

- The project is licensed `AGPL-3.0-or-later`: the crates declare it and
  every source file carries an SPDX header.
- YAML is read and written with `serde_norway`, a maintained fork of the
  unmaintained `serde_yaml`; the YAML format does not change.

### Added

- Continuous integration: formatting, Clippy and tests on Linux, macOS and
  Windows for every pull request.
- Release binaries for Linux (x86_64, aarch64), macOS (aarch64) and
  Windows (x86_64), attached to each GitHub release.

## 2.0.0

A rewrite of the parser, the intermediate representation and the layout.

### Breaking changes

- The YAML format changed: it is now a tree of statements per function
  (`functions: [{name, body: [...]}]`) instead of a graph of nodes and
  edges. YAML files written by 0.x must be generated again with `parse`.
- The library API changed: `parse`, `render_ascii`, `render_svg`,
  `validate` and `layout::layout` replace `parser::parse_source`,
  `render::render_flowchart` and `render::render_svg`.
- `--lang` accepts only `en` and `es`; other values are an error instead of
  falling back to English.
- Status messages go to standard error, so standard output carries only the
  drawing.
- The text output and the labels use only ASCII characters, so they look
  the same in every terminal, editor and font: lines are `-`, `|` and `+`,
  arrowheads are `v ^ < >`, decisions are drawn with `/ \ < >`, and labels
  write `<-`, `!=`, `<=` and `>=` instead of Unicode symbols.

### Fixed

- Statements in bodies without braces (`if (x) y = 0;`, `while (c) i++;`)
  were dropped.
- `continue` was ignored and left duplicated edges.
- An `if` whose branches end in `return` got an extra "False" edge and an
  unreachable end symbol.
- `goto` and labels were dropped, together with the statement after the
  label.
- A function was drawn as a subroutine when its name ended another name
  (`sum` inside `checksum(`).
- Without `main`, the function drawn was chosen at random.
- Text inside string literals was taken for code (`"printf("`).
- Levels 1–3 and 4–5 gave the same result.
- Renaming a node id in the YAML collapsed the whole drawing without an
  error.

### Added

- `switch`: grouped `case` labels, `default` anywhere, `break` inside nested
  statements.
- `strcmp`/`strncmp` comparisons, `&&`/`||`/`!` as words, casts and pointer
  syntax removed from labels.
- Warnings for syntax errors, `#if/#else` inside functions and anything that
  cannot be drawn.
- Junction dots in SVG; crossings drawn without a junction.
- Snapshot tests for every file in `resources/c`, layout invariant tests and
  CLI tests.
