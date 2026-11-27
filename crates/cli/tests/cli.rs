// SPDX-License-Identifier: AGPL-3.0-or-later
use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;

const CODE: &str = r#"
#include <stdio.h>
int twice(int x) { return 2 * x; }
int main() {
  int n;
  scanf("%d", &n);
  if (n > 0) printf("%d\n", twice(n));
  return 0;
}
"#;

fn cmd() -> Command {
    cargo_bin_cmd!("c2flowch")
}

#[test]
fn parse_then_render_every_format() {
    let dir = tempfile::tempdir().unwrap();
    let c = dir.path().join("prog.c");
    fs::write(&c, CODE).unwrap();

    cmd()
        .args(["parse", c.to_str().unwrap(), "--lang", "es"])
        .assert()
        .success();
    let yaml = dir.path().join("prog.yaml");
    let text = fs::read_to_string(&yaml).unwrap();
    assert!(text.contains("io: Leer n"), "{text}");
    assert!(text.contains("name: twice"), "{text}");

    cmd()
        .args(["render", yaml.to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Inicio").and(predicate::str::contains("Retornar: 2 * x")),
        );

    let svg = dir.path().join("prog.svg");
    cmd()
        .args([
            "render",
            yaml.to_str().unwrap(),
            "-f",
            "svg",
            "-o",
            svg.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(fs::read_to_string(&svg).unwrap().starts_with("<svg"));

    let png = dir.path().join("prog.png");
    cmd()
        .args([
            "render",
            yaml.to_str().unwrap(),
            "-f",
            "png",
            "-o",
            png.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(&fs::read(&png).unwrap()[..4], b"\x89PNG");
}

#[test]
fn split_writes_one_file_per_function() {
    let dir = tempfile::tempdir().unwrap();
    let c = dir.path().join("prog.c");
    fs::write(&c, CODE).unwrap();
    cmd()
        .args(["parse", c.to_str().unwrap()])
        .assert()
        .success();
    let out = dir.path().join("flow.txt");
    cmd()
        .args([
            "render",
            "-s",
            "-o",
            out.to_str().unwrap(),
            dir.path().join("prog.yaml").to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(fs::read_to_string(&out).unwrap().contains("Start"));
    assert!(
        fs::read_to_string(dir.path().join("flow-twice.txt"))
            .unwrap()
            .contains("Start: twice(x)")
    );
}

#[test]
fn warns_about_what_cannot_be_drawn() {
    let dir = tempfile::tempdir().unwrap();
    let yaml = dir.path().join("p.yaml");
    fs::write(
        &yaml,
        "title: p\nfunctions:\n- name: main\n  entry: true\n  body:\n  - goto: nowhere\n",
    )
    .unwrap();
    cmd()
        .args(["render", yaml.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("goto nowhere"));
}

#[test]
fn rejects_unknown_language() {
    cmd()
        .args(["parse", "x.c", "--lang", "fr"])
        .assert()
        .failure();
}
