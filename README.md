# c2flowch: C to flowchart (ANSI/ISO 5807)

**c2flowch** converts C source code into flowcharts with the ANSI/ISO 5807
symbols, as plain ASCII text, SVG or PNG.

Try it in the browser, nothing to install:
<https://airvzxf.github.io/c2flowch/>.

```c
int main() {
  int n = 0;
  while (1) {
    scanf("%d", &n);
    if (n < 0)
      break;
    if (n == 0) {
      puts("zero");
      continue;
    }
    printf("%d\n", n * 2);
  }
  return 0;
}
```

![Flowchart of the program above](docs/while_break.png)

## How it works

The conversion has two steps, with a YAML file in between that you can edit
by hand:

1. `parse`: C source -> YAML, one flowchart per function;
2. `render`: YAML -> text, SVG or PNG.

The YAML is a tree of statements that follows the structure of the code.
Every text is the label printed in its symbol:

```yaml
title: while_break.c
lang: en
functions:
- name: main
  entry: true
  body:
  - process: n <- 0
  - while:
      cond: '1'
      body:
      - io: Read n
      - if:
          cond: n < 0
          then:
          - break
      - if:
          cond: n = 0
          then:
          - io: Print "zero"
          - continue
      - io: Print n * 2
  - return: '0'
```

| Statement                              | Symbol                         |
|----------------------------------------|--------------------------------|
| `process: text`                        | Process (rectangle)            |
| `io: text`                             | Data (parallelogram)           |
| `call: text`                           | Predefined process             |
| `prep: text`                           | Preparation (hexagon)          |
| `if`, `while`, `do_while`, `for`       | Decision (diamond)             |
| `switch` (`expr`, `cases`)             | Decision, one symbol per case  |
| `break`, `continue`, `goto`, `label`   | Lines to their target          |
| `return: value`                        | Terminator                     |

A `switch` case is `{values: [...], default: true|false, body: [...]}`.
Without a trailing `break`, it falls through into the next case, as in C.

Variables declared outside of any function go in an optional `globals`
list, next to `functions`, with the same statements (`process`, `prep`,
...). They are drawn at the start of the entry point (`main`), right after
its start terminator, and follow the `--level` filter like any other
declaration:

```yaml
globals:
- process: counter <- 0
- prep: int values[LIMIT]
```

## Usage

```bash
# 1. C -> YAML (writes main.yaml)
c2flowch parse main.c --lang es --level 3
c2flowch parse main.c list.c util.c       # one program in several files

# 2. YAML -> drawing
c2flowch render main.yaml                          # text, to the terminal
c2flowch render main.yaml --color always | less -R # text with ANSI colors
c2flowch render main.yaml -f svg -o main.svg
c2flowch render main.yaml -f png -o main.png
c2flowch render main.yaml -f svg -o main.svg -s    # one file per function
```

`--level` sets how much detail is drawn:

| Level | Shows                                                          |
|-------|----------------------------------------------------------------|
| 1     | Calls to functions of the file, and the structures around them |
| 2     | Every control structure, input/output and calls                |
| 3     | Level 2 plus assignments and other operations (default)        |
| 4     | Level 3 plus declarations without a value                      |
| 5     | Every statement in its own symbol (nothing is grouped)         |

`--lang` sets the language of the labels: `en` (default) or `es`.

`parse` reports what it cannot draw faithfully (syntax errors, `#if/#else`
inside a function) on standard error. `render` reports a `goto` without its
label, or a `break`/`continue` outside of a loop.

### What the labels look like

| C                              | Label                    |
|--------------------------------|--------------------------|
| `x = y + 1;`                   | `x <- y + 1`              |
| `x += a * b;` / `i++;`         | `x <- x + (a * b)` / `i <- i + 1` |
| `a == b`, `a != b`, `a <= b`   | `a = b`, `a != b`, `a <= b` |
| `a && !b`                      | `a and not b`            |
| `strcmp(s, t) == 0`            | `s = t`                  |
| `p->next = *q;`                | `p.next <- q`             |
| `printf("x = %d\n", x);`       | `Print "x = ", x`        |
| `scanf("%d", &n);`             | `Read n`                 |

Calls to functions defined in the same file are drawn as predefined
processes, and each of those functions gets its own flowchart: first
`main`, then the functions it calls, in call order.

## Library

`c2flowch-core` does no I/O, so it can be used from other programs and
built for WebAssembly:

```rust
use c2flowch_core::{parse, render_ascii, render_svg, ParseOptions};

let parsed = parse(code, &ParseOptions::default())?;
let text = render_ascii(&parsed.program);
let svg = render_svg(&parsed.program);
```

## WebAssembly

`c2flowch-wasm` exposes the same steps to JavaScript, in a browser or
Node.js. Each release has a `c2flowch-wasm-vX.Y.Z.tar.gz` with the package
for the web; to build it yourself:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version <the wasm-bindgen version in Cargo.lock> --locked
bash scripts/build-wasm.bash                 # web/pkg, for browsers
bash scripts/build-wasm.bash nodejs out/     # for Node.js
```

The C grammar is compiled with `clang`, which must be installed.

```js
import init, { parse, parseFiles, render, validate, version } from "./pkg/c2flowch_wasm.js";

await init();
const parsed = parse(code, "main.c", 3, "en"); // code, title, level 1-5, en|es
console.log(parsed.warnings);                  // ["main.c:line 4: ..."]
const svg = render(parsed.yaml, "svg");        // or "ascii"
const multi = parseFiles(["a.c", "b.c"], [codeA, codeB], 3, "en");
```

Functions take and return the YAML text of a program, so it can be edited
between the steps; errors are thrown as `Error`. The `web/` directory is a
demo page: `build-wasm.bash` writes it from the sources in `web/src`, then
serve it with any static file server (`python3 -m http.server -d web`) and
open it. Every asset URL of the page carries `?v=<version>-<commit>`, such
as `?v=2.5.0-c209660`, so browsers and caches fetch the new files after
each deployment, whatever cache lifetime the server sends.

The demo is deployed to GitHub Pages by the release workflow. To host it
elsewhere, `scripts/cloudflare-build.bash` builds `web/` on a machine that
only has `cc`, `curl` and `tar`: it downloads Rust, clang (wasi-sdk) and
wasm-bindgen when they are missing. On Cloudflare Pages, connect the
repository with the build command `bash scripts/cloudflare-build.bash`
and the build output directory `web`.

## Development

```bash
cargo test                      # unit, snapshot and CLI tests
cargo insta review              # review changed snapshots
bash scripts/pipeline.bash      # render every file of resources/c
bash scripts/build-wasm.bash nodejs target/wasm-node \
  && cargo build && node scripts/wasm-smoke.mjs target/wasm-node target/debug/c2flowch
                                # WebAssembly gives the same drawings
```

Every file in `resources/c` is a test case: its YAML and text drawing are
kept as snapshots, and its layout is checked at every level of detail (no
line through a symbol, no label over a line, every jump routed).

See [PROJECT.md](./PROJECT.md) for the design and the roadmap, and
[CHANGELOG.md](./CHANGELOG.md) for the changes.

## License

c2flowch is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option)
any later version (`AGPL-3.0-or-later`). See [LICENSE](LICENSE).

The IBM Plex Mono fonts in `resources/fonts` are distributed under the
SIL Open Font License 1.1.
