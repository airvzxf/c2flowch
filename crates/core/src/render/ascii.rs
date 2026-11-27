// SPDX-License-Identifier: AGPL-3.0-or-later
//! Text rendering with plain ASCII characters, so the output looks the
//! same in every terminal, editor and font. Color is optional: when it is
//! on, ANSI escapes paint each symbol by its kind and nothing else changes.

use crate::layout::geom::{
    DECISION_MID, DOWN, Diagram, Dir, LEFT, PAD, Pt, RIGHT, Shape, ShapeKind, UP, width,
};
use std::collections::BTreeMap;
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Copy)]
enum Cell {
    Empty,
    /// A fixed character (symbol outline, text).
    Char(char),
    /// Second half of a double-width character.
    Wide,
    /// A border cell that lines may attach to; holds line directions.
    Border(u8),
}

/// What a cell belongs to, which picks its color.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ink {
    /// Text inside symbols, flow lines and empty space.
    Plain,
    /// The outline of a symbol.
    Outline(ShapeKind),
    /// Arrow heads.
    Arrow,
    /// Branch labels (True, False, case values).
    Label,
}

impl Ink {
    /// The ANSI SGR parameters of this ink, or `None` for the default.
    fn sgr(self) -> Option<&'static str> {
        Some(match self {
            Ink::Plain => return None,
            Ink::Outline(ShapeKind::Terminator | ShapeKind::Case) => "32",
            Ink::Outline(ShapeKind::Process) => "34",
            Ink::Outline(ShapeKind::Call) => "36",
            Ink::Outline(ShapeKind::Io) => "35",
            Ink::Outline(ShapeKind::Prep) => "31",
            Ink::Outline(ShapeKind::Decision) => "33",
            Ink::Arrow => "1",
            Ink::Label => "1;33",
        })
    }
}

struct Canvas {
    x0: i32,
    y0: i32,
    w: usize,
    cells: Vec<Cell>,
    inks: Vec<Ink>,
    /// The ink of the cells written from now on.
    pen: Ink,
}

impl Canvas {
    fn new(d: &Diagram) -> Canvas {
        let w = usize::try_from(d.width()).unwrap_or(0);
        let h = usize::try_from(d.height()).unwrap_or(0);
        Canvas {
            x0: d.min.0,
            y0: d.min.1,
            w,
            cells: vec![Cell::Empty; w * h],
            inks: vec![Ink::Plain; w * h],
            pen: Ink::Plain,
        }
    }

    fn idx(&self, p: Pt) -> Option<usize> {
        let x = usize::try_from(p.0 - self.x0).ok()?;
        let y = usize::try_from(p.1 - self.y0).ok()?;
        (x < self.w && y * self.w + x < self.cells.len()).then_some(y * self.w + x)
    }

    fn set(&mut self, p: Pt, c: Cell) {
        if let Some(i) = self.idx(p) {
            self.cells[i] = c;
            self.inks[i] = self.pen;
        }
    }

    fn put(&mut self, x: i32, y: i32, c: char) {
        self.set((x, y), Cell::Char(c));
    }

    /// Writes text with the plain ink, whatever the pen holds.
    fn text(&mut self, x: i32, y: i32, s: &str) {
        let pen = std::mem::replace(&mut self.pen, Ink::Plain);
        self.write(x, y, s);
        self.pen = pen;
    }

    fn write(&mut self, x: i32, y: i32, s: &str) {
        let mut x = x;
        for c in s.chars() {
            let cw = i32::try_from(c.width().unwrap_or(0)).unwrap_or(0);
            if cw == 0 {
                continue;
            }
            self.put(x, y, c);
            if cw == 2 {
                self.set((x + 1, y), Cell::Wide);
            }
            x += cw;
        }
    }

    fn fill(&mut self, x0: i32, x1: i32, y: i32, c: char) {
        for x in x0..=x1 {
            self.put(x, y, c);
        }
    }

    fn shape(&mut self, s: &Shape) {
        self.pen = Ink::Outline(s.kind);
        self.shape_cells(s);
        self.pen = Ink::Plain;
    }

    fn shape_cells(&mut self, s: &Shape) {
        let (x, y, r, b) = (s.x, s.y, s.right(), s.bottom());
        let border = |c: &mut Canvas, x0: i32, x1: i32, y: i32| {
            for x in x0..=x1 {
                c.set((x, y), Cell::Border(LEFT | RIGHT));
            }
        };
        match s.kind {
            ShapeKind::Process | ShapeKind::Call => {
                for row in y..=b {
                    self.fill(x, r, row, ' ');
                    self.put(x, row, '|');
                    self.put(r, row, '|');
                }
                self.fill(x + 1, r - 1, y, '-');
                border(self, x + 1, r - 1, b);
                self.put(x, y, '+');
                self.put(r, y, '+');
                self.put(x, b, '+');
                self.put(r, b, '+');
                let mut tx = x + 1 + PAD;
                if s.kind == ShapeKind::Call {
                    for ix in [x + 2, r - 2] {
                        for row in y + 1..b {
                            self.put(ix, row, '|');
                        }
                        self.put(ix, y, '+');
                        self.put(ix, b, '+');
                    }
                    tx += 2;
                }
                for (j, line) in s.lines.iter().enumerate() {
                    self.text(tx, y + 1 + row(j), line);
                }
            }
            ShapeKind::Io => {
                let off = s.h - 1;
                let inner = s.w - off;
                for j in 0..s.h {
                    let sx = x + off - j;
                    let edge = if j == 0 || j == s.h - 1 { '*' } else { '/' };
                    self.fill(sx, sx + inner - 1, y + j, ' ');
                    self.put(sx, y + j, edge);
                    self.put(sx + inner - 1, y + j, edge);
                }
                self.fill(x + off + 1, x + off + inner - 2, y, '-');
                border(self, x + 1, x + inner - 2, b);
                for (j, line) in s.lines.iter().enumerate() {
                    let j = row(j);
                    self.text(x + off - (j + 1) + 1 + PAD, y + 1 + j, line);
                }
            }
            ShapeKind::Prep => {
                for row in y..=b {
                    self.fill(x, r, row, ' ');
                }
                self.put(x + 1, y, '/');
                self.fill(x + 2, r - 2, y, '-');
                self.put(r - 1, y, '\\');
                for row in y + 1..b {
                    let (l, rr) = if s.h == 3 { ('<', '>') } else { ('|', '|') };
                    self.put(x, row, l);
                    self.put(r, row, rr);
                }
                self.put(x + 1, b, '\\');
                border(self, x + 2, r - 2, b);
                self.put(r - 1, b, '/');
                for (j, line) in s.lines.iter().enumerate() {
                    self.text(x + 1 + PAD, y + 1 + row(j), line);
                }
            }
            ShapeKind::Terminator | ShapeKind::Case => {
                for row in y..=b {
                    self.fill(x, r, row, ' ');
                    self.put(x, row, '(');
                    self.put(r, row, ')');
                }
                for (j, line) in s.lines.iter().enumerate() {
                    self.text(x + (s.w - width(line)) / 2, y + row(j), line);
                }
            }
            ShapeKind::Decision => {
                let cx = s.cx();
                let k = s.w / 6;
                for j in 0..=DECISION_MID {
                    let off = if j == DECISION_MID { j * k - 1 } else { j * k };
                    let (l, rr) = (cx - 1 - off, cx + off);
                    self.fill(l, rr, y + j, ' ');
                    let (cl, cr) = if j == DECISION_MID {
                        ('<', '>')
                    } else {
                        ('/', '\\')
                    };
                    self.put(l, y + j, cl);
                    self.put(rr, y + j, cr);
                }
                for j in 0..DECISION_MID {
                    let off = (DECISION_MID - 1 - j) * k;
                    let (l, rr) = (cx - 1 - off, cx + off);
                    let row = y + DECISION_MID + 1 + j;
                    self.fill(l, rr, row, ' ');
                    self.put(l, row, '\\');
                    self.put(rr, row, '/');
                }
                for (j, line) in s.lines.iter().enumerate() {
                    let tx = x + 1 + (s.w - 2 - width(line)) / 2;
                    self.text(tx, y + s.text_row(j), line);
                }
            }
        }
    }

    fn render(&self, color: bool) -> String {
        let w = self.w.max(1);
        let mut rows: Vec<String> = self
            .cells
            .chunks(w)
            .zip(self.inks.chunks(w))
            .map(|(row, inks)| {
                let chars: Vec<(char, Ink)> = row
                    .iter()
                    .zip(inks)
                    .filter_map(|(c, &ink)| match c {
                        Cell::Empty => Some((' ', ink)),
                        Cell::Char(ch) => Some((*ch, ink)),
                        Cell::Wide => None,
                        Cell::Border(m) => Some((line_char(*m), ink)),
                    })
                    .collect();
                let end = chars
                    .iter()
                    .rposition(|(c, _)| *c != ' ')
                    .map_or(0, |i| i + 1);
                let mut s = String::new();
                let mut cur = Ink::Plain;
                for &(ch, ink) in &chars[..end] {
                    // Spaces keep the current ink, so escapes only wrap
                    // visible characters.
                    if color && ch != ' ' && ink != cur {
                        if cur != Ink::Plain {
                            s.push_str("\x1b[0m");
                        }
                        if let Some(sgr) = ink.sgr() {
                            s.push_str("\x1b[");
                            s.push_str(sgr);
                            s.push('m');
                        }
                        cur = ink;
                    }
                    s.push(ch);
                }
                if cur != Ink::Plain {
                    s.push_str("\x1b[0m");
                }
                s
            })
            .collect();
        while rows.last().is_some_and(String::is_empty) {
            rows.pop();
        }
        rows.join("\n")
    }
}

fn row(j: usize) -> i32 {
    i32::try_from(j).unwrap_or(i32::MAX)
}

fn line_char(m: u8) -> char {
    match m {
        x if x == UP | DOWN | LEFT | RIGHT => '+',
        x if x == UP | DOWN | RIGHT => '+',
        x if x == UP | DOWN | LEFT => '+',
        x if x == DOWN | LEFT | RIGHT => '+',
        x if x == UP | LEFT | RIGHT => '+',
        x if x == DOWN | RIGHT => '+',
        x if x == DOWN | LEFT => '+',
        x if x == UP | RIGHT => '+',
        x if x == UP | LEFT => '+',
        x if x & (LEFT | RIGHT) != 0 && x & (UP | DOWN) == 0 => '-',
        _ => '|',
    }
}

fn arrow_char(d: Dir) -> char {
    match d {
        Dir::Down => 'v',
        Dir::Up => '^',
        Dir::Left => '<',
        Dir::Right => '>',
    }
}

/// Draws one diagram as text; `color` adds ANSI escapes.
pub fn render(d: &Diagram, color: bool) -> String {
    let mut c = Canvas::new(d);
    for s in &d.shapes {
        c.shape(s);
    }
    let (masks, crossings) = d.line_masks();
    let mut arrows: BTreeMap<Pt, Dir> = BTreeMap::new();
    for p in &d.paths {
        if let Some((at, dir)) = p.arrow() {
            arrows.insert(at, dir);
        }
    }
    for (&p, &m) in &masks {
        let Some(i) = c.idx(p) else { continue };
        c.cells[i] = match c.cells[i] {
            Cell::Border(b) => Cell::Border(b | m),
            Cell::Empty => {
                c.inks[i] = Ink::Plain;
                Cell::Border(m)
            }
            other => other,
        };
    }
    // Where two lines cross, the vertical one is drawn whole, so the
    // horizontal one reads as passing behind it.
    for &p in crossings.keys() {
        c.set(p, Cell::Char('|'));
    }
    c.pen = Ink::Arrow;
    for (&p, &dir) in &arrows {
        c.set(p, Cell::Char(arrow_char(dir)));
    }
    c.pen = Ink::Label;
    for l in &d.labels {
        c.write(l.x, l.y, &l.text);
    }
    c.render(color)
}
