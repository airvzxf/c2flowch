# Changelog

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
