//! c2flowch: C source code → ANSI/ISO 5807 flowcharts.
//!
//! The pipeline has three steps, each usable on its own:
//!
//! 1. [`parse`]: C source → [`Program`], a structured IR that can be saved
//!    as YAML and edited by hand;
//! 2. [`layout`](layout::layout): [`Program`] → one
//!    [`Diagram`](layout::geom::Diagram) per function;
//! 3. [`render_ascii`] / [`render_svg`]: drawing.
//!
//! The library does no I/O, so it can be built for WebAssembly.

mod detail;
pub mod i18n;
pub mod ir;
pub mod layout;
mod parse;
pub mod render;

pub use i18n::Language;
pub use ir::{Case, Function, Program, Stmt};
pub use parse::{ParseOptions, Parsed, Warning, parse};

use std::collections::HashSet;

/// Draws every function of a program as text, one below the other.
pub fn render_ascii(program: &Program) -> String {
    let mut out: Vec<String> = layout::layout(program)
        .iter()
        .map(render::ascii::render)
        .collect();
    out.retain(|s| !s.is_empty());
    let mut s = out.join("\n\n\n");
    s.push('\n');
    s
}

/// Draws every function of a program as one SVG image.
pub fn render_svg(program: &Program) -> String {
    render::svg::render(&program.title, &layout::layout(program))
}

/// Problems in a program that the drawing cannot show: a `break` or
/// `continue` outside of a loop, a `goto` to a missing label. Useful after
/// editing the YAML by hand.
pub fn validate(program: &Program) -> Vec<String> {
    let mut out = Vec::new();
    for f in &program.functions {
        let mut labels = HashSet::new();
        collect_labels(&f.body, &mut labels);
        check(&f.name, &f.body, false, false, &labels, &mut out);
    }
    out
}

fn collect_labels<'a>(body: &'a [Stmt], out: &mut HashSet<&'a str>) {
    for s in body {
        match s {
            Stmt::Label(l) => {
                out.insert(l);
            }
            Stmt::If {
                then, otherwise, ..
            } => {
                collect_labels(then, out);
                collect_labels(otherwise, out);
            }
            Stmt::While { body, .. } | Stmt::DoWhile { body, .. } | Stmt::For { body, .. } => {
                collect_labels(body, out);
            }
            Stmt::Switch { cases, .. } => cases.iter().for_each(|c| collect_labels(&c.body, out)),
            _ => {}
        }
    }
}

fn check(
    func: &str,
    body: &[Stmt],
    in_loop: bool,
    in_switch: bool,
    labels: &HashSet<&str>,
    out: &mut Vec<String>,
) {
    for s in body {
        match s {
            Stmt::Break if !in_loop && !in_switch => {
                out.push(format!(
                    "{func}: `break` outside of a loop or switch is not drawn"
                ));
            }
            Stmt::Continue if !in_loop => {
                out.push(format!("{func}: `continue` outside of a loop is not drawn"));
            }
            Stmt::Goto(l) if !labels.contains(l.as_str()) => {
                out.push(format!(
                    "{func}: `goto {l}` has no matching label and is not drawn"
                ));
            }
            Stmt::If {
                then, otherwise, ..
            } => {
                check(func, then, in_loop, in_switch, labels, out);
                check(func, otherwise, in_loop, in_switch, labels, out);
            }
            Stmt::While { body, .. } | Stmt::DoWhile { body, .. } | Stmt::For { body, .. } => {
                check(func, body, true, in_switch, labels, out);
            }
            Stmt::Switch { cases, .. } => {
                for c in cases {
                    check(func, &c.body, in_loop, true, labels, out);
                }
            }
            _ => {}
        }
    }
}
