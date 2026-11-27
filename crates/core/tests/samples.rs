// SPDX-License-Identifier: AGPL-3.0-or-later
//! Every C file in `resources/c`: snapshots of the YAML and of the text
//! drawing, plus layout invariants that must hold for any input.

use c2flowch_core::layout::geom::{Diagram, width};
use c2flowch_core::{Language, ParseOptions, Program, layout::layout, parse, render_ascii};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn samples() -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/c");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("resources/c exists")
        .map(|e| e.expect("readable entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "c"))
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|p| {
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            (name, fs::read_to_string(&p).unwrap())
        })
        .collect()
}

fn program(name: &str, code: &str, level: u8, lang: Language) -> Program {
    let opts = ParseOptions {
        title: format!("{name}.c"),
        level,
        lang,
    };
    parse(code, &opts).unwrap().program
}

#[test]
fn snapshots() {
    for (name, code) in samples() {
        let p = program(&name, code.as_str(), 3, Language::En);
        insta::assert_snapshot!(format!("{name}.yaml"), p.to_yaml().unwrap());
        insta::assert_snapshot!(format!("{name}.txt"), render_ascii(&p));
    }
}

#[test]
fn spanish() {
    let (name, code) = samples()
        .into_iter()
        .find(|(n, _)| n == "complex_calculator")
        .unwrap();
    insta::assert_snapshot!(render_ascii(&program(&name, &code, 3, Language::Es)));
}

#[test]
fn yaml_round_trip() {
    for (name, code) in samples() {
        for level in 1..=5 {
            let p = program(&name, &code, level, Language::Es);
            let back = Program::from_yaml(&p.to_yaml().unwrap()).unwrap();
            assert_eq!(p, back, "{name} at level {level}");
        }
    }
}

fn check(d: &Diagram, what: &str) {
    assert_eq!(d.unrouted, 0, "{what}: a jump found no route");
    let mut used: HashSet<(i32, i32)> = HashSet::new();
    for p in &d.paths {
        let cells = p.cells();
        for (i, c) in cells.iter().enumerate() {
            used.insert(*c);
            for s in &d.shapes {
                // A line may only start on the bottom border it leaves from.
                let allowed = i == 0 && s.has_bottom_border() && c.1 == s.bottom();
                assert!(
                    !s.contains(*c) || allowed,
                    "{what}: line {:?} goes through {:?} at {c:?}",
                    p.points,
                    s.lines
                );
            }
        }
    }
    for l in &d.labels {
        for x in l.x..l.x + width(&l.text) {
            let c = (x, l.y);
            assert!(
                !used.contains(&c),
                "{what}: label {:?} is on a line",
                l.text
            );
            for s in &d.shapes {
                assert!(
                    !s.contains(c),
                    "{what}: label {:?} is on {:?}",
                    l.text,
                    s.lines
                );
            }
        }
    }
    for (i, a) in d.shapes.iter().enumerate() {
        for b in &d.shapes[i + 1..] {
            let apart = a.right() < b.x || b.right() < a.x || a.bottom() < b.y || b.bottom() < a.y;
            assert!(apart, "{what}: {:?} overlaps {:?}", a.lines, b.lines);
        }
    }
}

#[test]
fn layout_invariants() {
    for (name, code) in samples() {
        for level in 1..=5 {
            for lang in [Language::En, Language::Es] {
                let p = program(&name, &code, level, lang);
                for d in layout(&p) {
                    check(&d, &format!("{name} level {level} {lang:?} {}", d.name));
                }
            }
        }
    }
}
