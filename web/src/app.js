// SPDX-License-Identifier: AGPL-3.0-or-later
import init, { parse, render, validate, version } from "./pkg/c2flowch_wasm.js?v=__VERSION__";

// Copied from resources/c by scripts/build-wasm.bash.
const EXAMPLES = [
  "while_break.c",
  "functions_basic.c",
  "switch_01.c",
  "globals.c",
  "complex_array_sort.c",
];

const $ = (id) => document.getElementById(id);
const code = $("code");
const yamlBox = $("view-yaml");
let view = "svg";
let yaml = "";
let timer;

function showError(e) {
  $("error").hidden = !e;
  $("error").textContent = e ? String(e.message ?? e) : "";
}

function draw() {
  try {
    $("view-svg").innerHTML = render(yaml, "svg");
    $("view-ascii").textContent = render(yaml, "ascii");
    showError(null);
    return true;
  } catch (e) {
    showError(e);
    return false;
  }
}

function setWarnings(list) {
  $("warnings").replaceChildren(
    ...list.map((w) => Object.assign(document.createElement("li"), { textContent: w })),
  );
}

function fromCode() {
  try {
    const name = $("example").value || "main.c";
    const parsed = parse(code.value, name, Number($("level").value), $("lang").value);
    yaml = parsed.yaml;
    yamlBox.value = yaml;
    setWarnings(parsed.warnings);
    draw();
  } catch (e) {
    showError(e);
  }
}

function fromYaml() {
  yaml = yamlBox.value;
  if (draw()) {
    try {
      setWarnings(validate(yaml));
    } catch (e) {
      showError(e);
    }
  }
}

function later(fn) {
  clearTimeout(timer);
  timer = setTimeout(fn, 250);
}

function select(name) {
  view = name;
  for (const tab of document.querySelectorAll("[role=tab]")) {
    tab.setAttribute("aria-selected", String(tab.dataset.view === name));
  }
  for (const v of ["svg", "ascii", "yaml"]) $(`view-${v}`).hidden = v !== name;
}

function download() {
  const [text, type, ext] = {
    svg: [render(yaml, "svg"), "image/svg+xml", "svg"],
    ascii: [render(yaml, "ascii"), "text/plain", "txt"],
    yaml: [yaml, "application/yaml", "yaml"],
  }[view];
  const base = ($("example").value || "main.c").replace(/\.c$/, "");
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([text], { type }));
  a.download = `${base}.${ext}`;
  a.click();
  URL.revokeObjectURL(a.href);
}

async function loadExample(name) {
  const res = await fetch(`examples/${name}?v=__VERSION__`);
  if (!res.ok) throw new Error(`examples/${name}: ${res.status}`);
  code.value = await res.text();
  fromCode();
}

await init({ module_or_path: new URL("./pkg/c2flowch_wasm_bg.wasm?v=__VERSION__", import.meta.url) });
$("version").textContent = `v${version()}`;
for (const name of EXAMPLES) $("example").append(new Option(name, name));
$("example").addEventListener("change", () => loadExample($("example").value).catch(showError));
code.addEventListener("input", () => later(fromCode));
yamlBox.addEventListener("input", () => later(fromYaml));
$("level").addEventListener("change", fromCode);
$("lang").addEventListener("change", fromCode);
$("download").addEventListener("click", download);
for (const tab of document.querySelectorAll("[role=tab]")) {
  tab.addEventListener("click", () => select(tab.dataset.view));
}
loadExample(EXAMPLES[0]).catch(showError);
