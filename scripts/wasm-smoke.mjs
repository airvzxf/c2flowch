// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Checks the WebAssembly package in Node.js against the command line:
//
//   scripts/build-wasm.bash nodejs target/wasm-node
//   node scripts/wasm-smoke.mjs target/wasm-node path/to/c2flowch
//
// For every file in resources/c, the text drawing made by the package must
// be the same as the one made by `c2flowch parse` + `c2flowch render`.
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const [pkgDir, cli] = process.argv.slice(2).map((p) => resolve(p));
const w = createRequire(import.meta.url)(join(pkgDir, "c2flowch_wasm.js"));
const tmp = mkdtempSync(join(tmpdir(), "c2flowch-"));
const dir = "resources/c";
let failed = 0;
let count = 0;

for (const name of readdirSync(dir).filter((f) => f.endsWith(".c")).sort()) {
  const path = join(dir, name);
  const yaml = join(tmp, `${name}.yaml`);
  const txt = join(tmp, `${name}.txt`);
  execFileSync(cli, ["parse", path, "-o", yaml], { stdio: "ignore" });
  execFileSync(cli, ["render", yaml, "-f", "ascii", "--color", "never", "-o", txt], {
    stdio: "ignore",
  });
  const parsed = w.parse(readFileSync(path, "utf8"), name, 3, "en");
  const ascii = w.render(parsed.yaml, "ascii");
  if (ascii !== readFileSync(txt, "utf8")) {
    console.error(`FAIL ${name}: the drawing differs from the command line`);
    failed++;
  }
  if (!w.render(parsed.yaml, "svg").includes("<svg")) {
    console.error(`FAIL ${name}: no SVG`);
    failed++;
  }
  count++;
}

const multi = w.parseFiles(["a.c", "b.c"], ["int main(void) { f(); }", "void f(void) {}"], 3, "es");
if (!multi.yaml.includes("title: a.c, b.c")) {
  console.error("FAIL parseFiles: wrong title");
  failed++;
}
for (const bad of [() => w.render("x", "png"), () => w.parse("", "x.c", 9, "en")]) {
  try {
    bad();
    console.error("FAIL: a bad argument did not throw");
    failed++;
  } catch {
    // expected
  }
}

console.log(`c2flowch-wasm ${w.version()}: ${count} files checked, ${failed} failures`);
process.exit(failed ? 1 : 0);
