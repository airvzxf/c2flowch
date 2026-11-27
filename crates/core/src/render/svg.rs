// SPDX-License-Identifier: AGPL-3.0-or-later
//! Vector rendering. Every cell of the layout grid becomes `CW x CH`
//! pixels; lines run through the centers of their cells.

use crate::layout::geom::{
    DECISION_MID, Diagram, Dir, End, PAD, Path, Pt, Shape, ShapeKind, step, width,
};
use std::fmt::Write;

const CW: f64 = 9.0;
const CH: f64 = 18.0;
const MARGIN: f64 = 24.0;
const TITLE_H: f64 = 40.0;
const SECTION_GAP: f64 = 48.0;
/// Half the height of terminators and case symbols, in rows.
const ROUND_HALF: f64 = 0.75;

/// IBM Plex Mono advances 0.6 em per character: 15 px x 0.6 = 9 px = CW.
const STYLE: &str = "text{font-family:'IBM Plex Mono',monospace;font-size:15px;fill:#1f2328}\
.lbl{font-size:13px;fill:#57606a}\
.title{font-family:sans-serif;font-size:20px;font-weight:bold}\
.fn{font-family:sans-serif;font-size:15px;font-weight:bold;fill:#57606a}\
.sym{fill:#fff;stroke:#1f2328;stroke-width:1.6}\
.line{fill:none;stroke:#1f2328;stroke-width:1.4}";

struct Frame {
    x0: i32,
    y0: i32,
    ox: f64,
    oy: f64,
}

impl Frame {
    fn x(&self, x: i32) -> f64 {
        self.ox + f64::from(x - self.x0) * CW + CW / 2.0
    }

    fn y(&self, y: i32) -> f64 {
        self.oy + f64::from(y - self.y0) * CH + CH / 2.0
    }

    /// Pixel box of a symbol outline: left, top, right, bottom.
    fn outline(&self, s: &Shape) -> (f64, f64, f64, f64) {
        match s.kind {
            ShapeKind::Terminator | ShapeKind::Case => (
                self.x(s.x) - CW * 0.2,
                self.y(s.y) - CH * ROUND_HALF,
                self.x(s.right()) + CW * 0.2,
                self.y(s.bottom()) + CH * ROUND_HALF,
            ),
            ShapeKind::Decision => {
                let cx = self.x(s.cx());
                let half = (f64::from(s.w) / 2.0 - 0.5) * CW;
                (cx - half, self.y(s.y), cx + half, self.y(s.bottom()))
            }
            _ => (
                self.x(s.x),
                self.y(s.y),
                self.x(s.right()),
                self.y(s.bottom()),
            ),
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn text(out: &mut String, x: f64, y: f64, s: &str, class: Option<&str>) {
    let class = class.map(|c| format!(" class=\"{c}\"")).unwrap_or_default();
    let _ = writeln!(
        out,
        "<text x=\"{x:.1}\" y=\"{y:.1}\" dominant-baseline=\"central\" xml:space=\"preserve\"{class}>{}</text>",
        esc(s)
    );
}

fn shape(out: &mut String, f: &Frame, s: &Shape) {
    let (l, t, r, b) = f.outline(s);
    match s.kind {
        ShapeKind::Process => {
            let _ = writeln!(
                out,
                "<rect class=\"sym\" x=\"{l:.1}\" y=\"{t:.1}\" width=\"{:.1}\" height=\"{:.1}\"/>",
                r - l,
                b - t
            );
        }
        ShapeKind::Call => {
            let _ = writeln!(
                out,
                "<rect class=\"sym\" x=\"{l:.1}\" y=\"{t:.1}\" width=\"{:.1}\" height=\"{:.1}\"/>",
                r - l,
                b - t
            );
            for x in [l + 2.0 * CW, r - 2.0 * CW] {
                let _ = writeln!(
                    out,
                    "<line class=\"line\" x1=\"{x:.1}\" y1=\"{t:.1}\" x2=\"{x:.1}\" y2=\"{b:.1}\"/>"
                );
            }
        }
        ShapeKind::Io => {
            let off = f64::from(s.h - 1) * CW;
            let _ = writeln!(
                out,
                "<polygon class=\"sym\" points=\"{:.1},{t:.1} {r:.1},{t:.1} {:.1},{b:.1} {l:.1},{b:.1}\"/>",
                l + off,
                r - off
            );
        }
        ShapeKind::Prep => {
            let m = (t + b) / 2.0;
            let _ = writeln!(
                out,
                "<polygon class=\"sym\" points=\"{:.1},{t:.1} {:.1},{t:.1} {r:.1},{m:.1} {:.1},{b:.1} {:.1},{b:.1} {l:.1},{m:.1}\"/>",
                l + CW,
                r - CW,
                r - CW,
                l + CW
            );
        }
        ShapeKind::Terminator | ShapeKind::Case => {
            let rad = (b - t) / 2.0;
            let _ = writeln!(
                out,
                "<rect class=\"sym\" x=\"{l:.1}\" y=\"{t:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"{rad:.1}\"/>",
                r - l,
                b - t
            );
        }
        ShapeKind::Decision => {
            let cx = f.x(s.cx());
            let m = f.y(s.y + DECISION_MID);
            let _ = writeln!(
                out,
                "<polygon class=\"sym\" points=\"{cx:.1},{t:.1} {r:.1},{m:.1} {cx:.1},{b:.1} {l:.1},{m:.1}\"/>"
            );
        }
    }

    // Text: same cells as in the text rendering.
    let left_of = |cell: i32| f.x(cell) - CW / 2.0;
    match s.kind {
        ShapeKind::Decision => {
            let line = s.lines.join(" ");
            let x = s.x + 1 + (s.w - 2 - width(&line)) / 2;
            text(out, left_of(x), f.y(s.y + DECISION_MID), &line, None);
        }
        ShapeKind::Terminator | ShapeKind::Case => {
            for (j, line) in s.lines.iter().enumerate() {
                let x = s.x + (s.w - width(line)) / 2;
                text(out, left_of(x), f.y(s.y + row(j)), line, None);
            }
        }
        _ => {
            for (j, line) in s.lines.iter().enumerate() {
                let j = row(j);
                let x = match s.kind {
                    ShapeKind::Call => s.x + 3 + PAD,
                    ShapeKind::Io => s.x + (s.h - 1) - (j + 1) + 1 + PAD,
                    _ => s.x + 1 + PAD,
                };
                text(out, left_of(x), f.y(s.y + 1 + j), line, None);
            }
        }
    }
}

fn row(j: usize) -> i32 {
    i32::try_from(j).unwrap_or(i32::MAX)
}

/// Pixel points of a line. Ends next to a symbol stretch to its outline.
fn points(f: &Frame, d: &Diagram, p: &Path) -> Vec<(f64, f64)> {
    let mut pts: Vec<(f64, f64)> = p.points.iter().map(|&(x, y)| (f.x(x), f.y(y))).collect();
    let snap = |pt: &mut (f64, f64), cell: Pt, toward: Dir| {
        let near = step(cell, toward);
        if let Some(s) = d
            .shapes
            .iter()
            .find(|s| s.contains(near) || s.contains(cell))
        {
            let (l, t, r, b) = f.outline(s);
            match toward {
                Dir::Up => pt.1 = b,
                Dir::Down => pt.1 = t,
                Dir::Left => pt.0 = r,
                Dir::Right => pt.0 = l,
            }
        }
    };
    if let Some(d0) = p.first_dir() {
        snap(&mut pts[0], p.points[0], d0.opposite());
    }
    if p.end == End::Shape
        && let Some(dn) = p.last_dir()
    {
        let n = pts.len() - 1;
        snap(&mut pts[n], p.points[n], dn);
    }
    pts
}

fn diagram(out: &mut String, d: &Diagram, ox: f64, oy: f64) {
    let f = Frame {
        x0: d.min.0,
        y0: d.min.1,
        ox,
        oy,
    };
    let (masks, crossings) = d.line_masks();
    for p in &d.paths {
        let pts: Vec<String> = points(&f, d, p)
            .iter()
            .map(|(x, y)| format!("{x:.1},{y:.1}"))
            .collect();
        let marker = if p.end == End::Plain {
            ""
        } else {
            " marker-end=\"url(#arrow)\""
        };
        let _ = writeln!(
            out,
            "<polyline class=\"line\" points=\"{}\"{marker}/>",
            pts.join(" ")
        );
    }
    for (&(x, y), &m) in &masks {
        if m.count_ones() >= 3 && !crossings.contains_key(&(x, y)) {
            let _ = writeln!(
                out,
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3\" fill=\"#1f2328\"/>",
                f.x(x),
                f.y(y)
            );
        }
    }
    for s in &d.shapes {
        shape(out, &f, s);
    }
    for l in &d.labels {
        text(out, f.x(l.x) - CW / 2.0, f.y(l.y), &l.text, Some("lbl"));
    }
}

/// Draws diagrams one below the other, under a title.
pub fn render(title: &str, diagrams: &[Diagram]) -> String {
    let show_names = diagrams.len() > 1;
    let name_h = if show_names { 28.0 } else { 0.0 };
    let mut body = String::new();
    let mut y = MARGIN + TITLE_H;
    let mut w: f64 = 320.0;
    for (i, d) in diagrams.iter().enumerate() {
        if i > 0 {
            y += SECTION_GAP;
        }
        if show_names {
            text(&mut body, MARGIN, y + 8.0, &d.name, Some("fn"));
            y += name_h;
        }
        diagram(&mut body, d, MARGIN, y);
        y += f64::from(d.height()) * CH;
        w = w.max(MARGIN * 2.0 + f64::from(d.width()) * CW);
    }
    let h = y + MARGIN;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\">"
    );
    let _ = writeln!(
        out,
        "<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"10\" refY=\"5\" markerWidth=\"7\" markerHeight=\"7\" orient=\"auto-start-reverse\"><path d=\"M0,0 L10,5 L0,10 z\" fill=\"#1f2328\"/></marker></defs>"
    );
    let _ = writeln!(out, "<style>{STYLE}</style>");
    let _ = writeln!(out, "<rect width=\"100%\" height=\"100%\" fill=\"#fff\"/>");
    text(&mut out, MARGIN, MARGIN + 12.0, title, Some("title"));
    out.push_str(&body);
    out.push_str("</svg>\n");
    out
}
