// SPDX-License-Identifier: AGPL-3.0-or-later
//! Drawing primitives on a grid of character cells.
//!
//! The layout works in cells so that the ASCII renderer can draw it as is.
//! The SVG renderer maps every cell to a fixed number of pixels.

use std::collections::BTreeMap;
use unicode_width::UnicodeWidthStr;

pub type Pt = (i32, i32);

/// Line directions in a cell, as bits.
pub const UP: u8 = 1;
pub const DOWN: u8 = 2;
pub const LEFT: u8 = 4;
pub const RIGHT: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

    pub fn delta(self) -> Pt {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }

    pub fn bit(self) -> u8 {
        match self {
            Dir::Up => UP,
            Dir::Down => DOWN,
            Dir::Left => LEFT,
            Dir::Right => RIGHT,
        }
    }

    pub fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }

    pub fn vertical(self) -> bool {
        matches!(self, Dir::Up | Dir::Down)
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// Direction of the straight move from `a` to `b`.
    pub fn between(a: Pt, b: Pt) -> Dir {
        if a.0 == b.0 {
            if b.1 > a.1 { Dir::Down } else { Dir::Up }
        } else if b.0 > a.0 {
            Dir::Right
        } else {
            Dir::Left
        }
    }
}

pub fn step(p: Pt, d: Dir) -> Pt {
    let (dx, dy) = d.delta();
    (p.0 + dx, p.1 + dy)
}

/// Display width of a text, in cells.
pub fn width(s: &str) -> i32 {
    i32::try_from(UnicodeWidthStr::width(s)).unwrap_or(i32::MAX)
}

/// ISO 5807 symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    /// Start and end (rounded ends).
    Terminator,
    /// Process (rectangle).
    Process,
    /// Predefined process (rectangle with double sides).
    Call,
    /// Data / input-output (parallelogram).
    Io,
    /// Preparation (hexagon).
    Prep,
    /// Decision (diamond).
    Decision,
    /// One value of a `switch` (small rounded symbol).
    Case,
}

/// Space between the border of a symbol and its text, in cells.
pub const PAD: i32 = 3;
/// Rows of a decision symbol, and the row of its side corners.
pub const DECISION_H: i32 = 7;
pub const DECISION_MID: i32 = 3;
/// Widest condition, in cells, that a decision shows on one line. Longer
/// conditions wrap onto the rows above and below the middle one.
pub const DECISION_LINE_MAX: i32 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub kind: ShapeKind,
    /// Top-left cell.
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub lines: Vec<String>,
}

impl Shape {
    /// A symbol whose center column is `x = 0` and whose top row is `y = 0`.
    pub fn new(kind: ShapeKind, text: &str) -> Shape {
        let lines: Vec<String> = text.split('\n').map(str::to_string).collect();
        let n = i32::try_from(lines.len()).unwrap_or(1).max(1);
        let tw = lines.iter().map(|l| width(l)).max().unwrap_or(0);
        let odd = |w: i32| if w % 2 == 0 { w + 1 } else { w };
        let (w, h) = match kind {
            ShapeKind::Process | ShapeKind::Prep => (odd(tw + 2 * PAD + 2), n + 2),
            ShapeKind::Call => (odd(tw + 2 * PAD + 6), n + 2),
            ShapeKind::Io => (odd(tw + 2 * PAD + 2 + n + 1), n + 2),
            // Even widths keep the text centered; the flow line then runs
            // half a cell right of the middle, which is not noticeable.
            ShapeKind::Terminator => (tw + 6, n),
            ShapeKind::Case => (tw + 4, 1),
            ShapeKind::Decision => {
                let (k, wrapped) = decision_text(&lines.join(" "));
                return Shape {
                    kind,
                    x: -(3 * k),
                    y: 0,
                    w: 6 * k,
                    h: DECISION_H,
                    lines: wrapped,
                };
            }
        };
        Shape {
            kind,
            x: -(w / 2),
            y: 0,
            w,
            h,
            lines,
        }
    }

    /// Row of the `j`-th text line, relative to the top of the symbol.
    pub fn text_row(&self, j: usize) -> i32 {
        let j = i32::try_from(j).unwrap_or(i32::MAX);
        match self.kind {
            ShapeKind::Decision if self.lines.len() > 1 => DECISION_MID - 1 + j,
            ShapeKind::Decision => DECISION_MID,
            ShapeKind::Terminator | ShapeKind::Case => j,
            _ => 1 + j,
        }
    }

    pub fn right(&self) -> i32 {
        self.x + self.w - 1
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.h - 1
    }

    /// Center column, where the flow lines enter and leave.
    pub fn cx(&self) -> i32 {
        self.x + self.w / 2
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.0 >= self.x && p.0 <= self.right() && p.1 >= self.y && p.1 <= self.bottom()
    }

    /// Whether the bottom row is a border line that a flow line can leave
    /// from (drawn as `+`).
    pub fn has_bottom_border(&self) -> bool {
        matches!(
            self.kind,
            ShapeKind::Process | ShapeKind::Call | ShapeKind::Io | ShapeKind::Prep
        )
    }
}

/// Size factor `k` (the diamond is `6 * k` cells wide) and the text lines
/// of a decision.
///
/// A short condition stays on the middle row. A longer one is split at
/// spaces into up to three lines: the middle row holds `6k - 10` cells and
/// the rows above and below it, where the diamond is narrower, `4k - 6`.
/// The margins keep the text clear of the slanted sides in the SVG drawing.
/// The smallest `k` that fits wins.
fn decision_text(text: &str) -> (i32, Vec<String>) {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let tw = width(&text);
    let one = ((tw + 6 + 5) / 6).max(2);
    if tw <= DECISION_LINE_MAX || !text.contains(' ') {
        return (one, vec![text]);
    }
    let words: Vec<&str> = text.split(' ').collect();
    for k in 2..one {
        if let Some(lines) = fill(&words, &[4 * k - 6, 6 * k - 10, 4 * k - 6]) {
            return (k, lines);
        }
    }
    (one, vec![text])
}

/// Greedy word wrap into lines of the given widths; `None` if the words do
/// not fit.
fn fill(words: &[&str], widths: &[i32]) -> Option<Vec<String>> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut slot = 0;
    for w in words {
        let next = if cur.is_empty() {
            (*w).to_string()
        } else {
            format!("{cur} {w}")
        };
        if width(&next) <= *widths.get(slot)? {
            cur = next;
            continue;
        }
        if cur.is_empty() {
            return None;
        }
        lines.push(std::mem::take(&mut cur));
        slot += 1;
        if width(w) > *widths.get(slot)? {
            return None;
        }
        cur = (*w).to_string();
    }
    lines.push(cur);
    Some(lines)
}

/// How a flow line ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// No arrowhead: the line continues into another line.
    Plain,
    /// The last cell is an arrowhead pointing at a symbol.
    Shape,
    /// The last cell joins another line; the cell before it is an arrowhead.
    Join,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// Corners of an orthogonal line, first to last.
    pub points: Vec<Pt>,
    pub end: End,
}

impl Path {
    pub fn new(points: Vec<Pt>, end: End) -> Path {
        let mut p = Path { points, end };
        p.simplify();
        p
    }

    /// Removes repeated points and points in the middle of a straight run.
    fn simplify(&mut self) {
        self.points.dedup();
        let mut i = 1;
        while i + 1 < self.points.len() {
            let (a, b, c) = (self.points[i - 1], self.points[i], self.points[i + 1]);
            if (a.0 == b.0 && b.0 == c.0) || (a.1 == b.1 && b.1 == c.1) {
                self.points.remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Every cell of the line, in order.
    pub fn cells(&self) -> Vec<Pt> {
        let mut out = vec![self.points[0]];
        for w in self.points.windows(2) {
            let d = Dir::between(w[0], w[1]);
            let mut p = w[0];
            while p != w[1] {
                p = step(p, d);
                out.push(p);
            }
        }
        out
    }

    /// The line directions that this path adds to every cell it crosses.
    pub fn masks(&self) -> BTreeMap<Pt, u8> {
        let cells = self.cells();
        let mut out = BTreeMap::new();
        for w in cells.windows(2) {
            let d = Dir::between(w[0], w[1]);
            *out.entry(w[0]).or_insert(0) |= d.bit();
            *out.entry(w[1]).or_insert(0) |= d.opposite().bit();
        }
        if cells.len() == 1 {
            out.insert(cells[0], 0);
        }
        out
    }

    /// The cell drawn as an arrowhead, and the direction it points to.
    pub fn arrow(&self) -> Option<(Pt, Dir)> {
        let cells = self.cells();
        let n = cells.len();
        match self.end {
            End::Plain => None,
            End::Shape if n >= 2 => Some((cells[n - 1], Dir::between(cells[n - 2], cells[n - 1]))),
            End::Join if n >= 3 => Some((cells[n - 2], Dir::between(cells[n - 3], cells[n - 2]))),
            _ => None,
        }
    }

    pub fn first_dir(&self) -> Option<Dir> {
        (self.points.len() >= 2).then(|| Dir::between(self.points[0], self.points[1]))
    }

    pub fn last_dir(&self) -> Option<Dir> {
        let n = self.points.len();
        (n >= 2).then(|| Dir::between(self.points[n - 2], self.points[n - 1]))
    }
}

/// Free text: branch labels ("True", "False") and `goto` labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub x: i32,
    pub y: i32,
    pub text: String,
}

impl Label {
    pub fn right(&self) -> i32 {
        self.x + width(&self.text) - 1
    }
}

/// The flowchart of one function, ready to be drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    pub name: String,
    pub shapes: Vec<Shape>,
    pub paths: Vec<Path>,
    pub labels: Vec<Label>,
    /// Bounding box, both corners included.
    pub min: Pt,
    pub max: Pt,
    /// Jumps for which no route was found (and that are not drawn).
    pub unrouted: usize,
}

impl Diagram {
    pub fn new(name: String, shapes: Vec<Shape>, paths: Vec<Path>, labels: Vec<Label>) -> Self {
        let mut d = Diagram {
            name,
            shapes,
            paths,
            labels,
            min: (0, 0),
            max: (0, 0),
            unrouted: 0,
        };
        d.fit();
        d
    }

    fn fit(&mut self) {
        let mut min = (i32::MAX, i32::MAX);
        let mut max = (i32::MIN, i32::MIN);
        let mut add = |p: Pt| {
            min = (min.0.min(p.0), min.1.min(p.1));
            max = (max.0.max(p.0), max.1.max(p.1));
        };
        for s in &self.shapes {
            add((s.x, s.y));
            add((s.right(), s.bottom()));
        }
        for p in &self.paths {
            p.points.iter().for_each(|&q| add(q));
        }
        for l in &self.labels {
            add((l.x, l.y));
            add((l.right(), l.y));
        }
        if min.0 > max.0 {
            min = (0, 0);
            max = (0, 0);
        }
        self.min = min;
        self.max = max;
    }

    pub fn width(&self) -> i32 {
        self.max.0 - self.min.0 + 1
    }

    pub fn height(&self) -> i32 {
        self.max.1 - self.min.1 + 1
    }

    /// Line directions of every cell, with the cells where two lines cross
    /// without joining. A cell is a crossing when a line passes straight
    /// through a straight line that goes the other way.
    pub fn line_masks(&self) -> (BTreeMap<Pt, u8>, BTreeMap<Pt, u8>) {
        let mut masks: BTreeMap<Pt, u8> = BTreeMap::new();
        let mut crossings: BTreeMap<Pt, u8> = BTreeMap::new();
        for path in &self.paths {
            for (p, m) in path.masks() {
                let old = masks.get(&p).copied().unwrap_or(0);
                let straight = |m: u8| m == UP | DOWN || m == LEFT | RIGHT;
                if straight(old) && straight(m) && old & m == 0 {
                    crossings.insert(p, m);
                } else {
                    masks.insert(p, old | m);
                }
            }
        }
        (masks, crossings)
    }
}
