// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structured layout: IR -> [`Diagram`].
//!
//! Every statement becomes a [`Block`]: a rectangle of cells whose flow
//! enters at the top of column 0 and leaves at the bottom of column 0.
//! Control structures are built from the blocks of their children, so the
//! normal flow never needs a search. Only jumps (`break`, `continue`,
//! `goto`) are routed afterwards, by [`route`], around what is already
//! drawn.
//!
//! Conventions, the same as the classic hand-drawn style:
//! - the "False" branch of a decision goes straight down;
//! - the "True" branch goes to the right (or to the left, for an `if` whose
//!   branch leaves a loop with `break`, so the jump does not cross the loop);
//! - loops return on the right side, into a join just above the condition.

pub mod geom;
mod route;

use crate::i18n::Words;
use crate::ir::{Case, Function, Program, Stmt};
use geom::{
    DECISION_H as DH, DECISION_MID as MID, Diagram, Dir, End, Label, Path, Pt, Shape, ShapeKind,
    width,
};
use route::Route;

/// Rows between two blocks: one line and one arrowhead.
const GAP: i32 = 2;
/// Columns between a column and the next one on its right.
const HGAP: i32 = 4;

/// Lays out every function of a program, in order.
pub fn layout(program: &Program) -> Vec<Diagram> {
    let words = program.lang.words();
    program
        .functions
        .iter()
        .map(|f| {
            Layout {
                words,
                entry: f.entry,
            }
            .function(f)
        })
        .collect()
}

#[derive(Debug, Clone)]
enum JumpKind {
    Break,
    Continue,
    Goto(String),
}

#[derive(Debug, Clone)]
struct Jump {
    kind: JumpKind,
    at: Pt,
}

#[derive(Debug, Default)]
struct Block {
    shapes: Vec<Shape>,
    paths: Vec<Path>,
    labels: Vec<Label>,
    /// Jumps resolved to their targets, routed once the function is done.
    routes: Vec<Route>,
    /// Jumps whose target is outside this block.
    jumps: Vec<Jump>,
    /// `goto` targets.
    anchors: Vec<(String, Pt)>,
    left: i32,
    right: i32,
    height: i32,
    /// The flow enters on a line cell at (0, 0), not on a symbol.
    entry_line: bool,
    /// Row where the outgoing line starts, in column 0; `None` when the
    /// flow never leaves the block at the bottom.
    exit: Option<i32>,
}

impl Block {
    /// A single line cell: an empty branch, or a point where flow joins.
    fn line() -> Block {
        Block {
            height: 1,
            entry_line: true,
            exit: Some(0),
            ..Block::default()
        }
    }

    fn shape(kind: ShapeKind, text: &str) -> Block {
        let s = Shape::new(kind, text);
        let exit = if s.has_bottom_border() { s.h - 1 } else { s.h };
        Block {
            left: s.x,
            right: s.right(),
            height: s.h,
            exit: Some(exit),
            shapes: vec![s],
            ..Block::default()
        }
    }

    fn span(&mut self, x0: i32, x1: i32) {
        self.left = self.left.min(x0);
        self.right = self.right.max(x1);
    }

    fn label(&mut self, x: i32, y: i32, text: &str) {
        let l = Label {
            x,
            y,
            text: text.to_string(),
        };
        self.span(l.x, l.right());
        self.labels.push(l);
    }

    fn path(&mut self, points: Vec<Pt>, end: End) {
        for p in &points {
            self.span(p.0, p.0);
        }
        self.paths.push(Path::new(points, end));
    }

    /// Moves `other` by (dx, dy) and adds it to this block. Size and
    /// exit of this block are left to the caller.
    fn place(&mut self, other: Block, dx: i32, dy: i32) {
        let mv = |p: Pt| (p.0 + dx, p.1 + dy);
        self.span(other.left + dx, other.right + dx);
        self.shapes.extend(other.shapes.into_iter().map(|mut s| {
            s.x += dx;
            s.y += dy;
            s
        }));
        self.paths.extend(other.paths.into_iter().map(|mut p| {
            p.points.iter_mut().for_each(|q| *q = mv(*q));
            p
        }));
        self.labels.extend(other.labels.into_iter().map(|mut l| {
            l.x += dx;
            l.y += dy;
            l
        }));
        self.routes
            .extend(other.routes.into_iter().map(|r| r.moved(dx, dy)));
        self.jumps.extend(other.jumps.into_iter().map(|mut j| {
            j.at = mv(j.at);
            j
        }));
        self.anchors
            .extend(other.anchors.into_iter().map(|(n, p)| (n, mv(p))));
    }

    /// Turns the pending jumps of one kind into routes to `targets`.
    fn resolve(&mut self, is_kind: fn(&JumpKind) -> bool, targets: &[(Pt, Dir)]) {
        let (hit, keep): (Vec<Jump>, Vec<Jump>) = std::mem::take(&mut self.jumps)
            .into_iter()
            .partition(|j| is_kind(&j.kind));
        self.jumps = keep;
        for j in hit {
            self.routes.push(Route {
                from: j.at,
                targets: targets.to_vec(),
            });
        }
    }
}

/// Vertical line from an exit row to the top of the next block.
fn connector(x: i32, from: i32, top: i32, entry_line: bool) -> Path {
    if entry_line {
        Path::new(vec![(x, from), (x, top)], End::Plain)
    } else {
        Path::new(vec![(x, from), (x, top - 1)], End::Shape)
    }
}

/// Every cell of a straight line from `a` to `b` (ends excluded) that a jump
/// can join from either side.
fn side_targets(a: Pt, b: Pt) -> Vec<(Pt, Dir)> {
    let d = Dir::between(a, b);
    let (s1, s2) = if d.vertical() {
        (Dir::Left, Dir::Right)
    } else {
        (Dir::Up, Dir::Down)
    };
    let mut out = Vec::new();
    let mut p = geom::step(a, d);
    while p != b {
        out.push((p, s1));
        out.push((p, s2));
        p = geom::step(p, d);
    }
    out
}

fn is_break(k: &JumpKind) -> bool {
    matches!(k, JumpKind::Break)
}

fn is_continue(k: &JumpKind) -> bool {
    matches!(k, JumpKind::Continue)
}

/// Whether `body` has a `break` or `continue` that leaves `body` itself,
/// that is, one not caught by an inner loop (or, for `break`, an inner
/// `switch`).
fn leaves_with(body: &[Stmt], brk: bool) -> bool {
    body.iter().any(|s| match s {
        Stmt::Break => brk,
        Stmt::Continue => !brk,
        Stmt::If {
            then, otherwise, ..
        } => leaves_with(then, brk) || leaves_with(otherwise, brk),
        Stmt::Switch { cases, .. } if !brk => cases.iter().any(|c| leaves_with(&c.body, brk)),
        _ => false,
    })
}

#[derive(Debug, Clone, Copy)]
struct Ctx {
    /// The innermost `break` target is a loop, whose exit line is on the
    /// left of its body.
    break_left: bool,
}

struct Layout {
    words: &'static Words,
    /// Laying out the program entry point (`main`).
    entry: bool,
}

impl Layout {
    fn function(&self, f: &Function) -> Diagram {
        let w = self.words;
        let start = if f.entry {
            w.start.to_string()
        } else {
            format!("{}: {}({})", w.start, f.name, f.params.join(", "))
        };
        let ctx = Ctx { break_left: true };
        let body = self.seq(&f.body, ctx);
        let falls_out = body.exit.is_some();
        let mut blocks = vec![Block::shape(ShapeKind::Terminator, &start), body];
        if falls_out {
            blocks.push(self.terminator(if f.entry { w.end } else { w.ret }));
        }
        let mut b = stack(blocks);

        for j in std::mem::take(&mut b.jumps) {
            // `break`/`continue` outside of any loop cannot be drawn.
            let JumpKind::Goto(name) = j.kind else {
                continue;
            };
            if let Some((_, p)) = b.anchors.iter().find(|(n, _)| *n == name) {
                b.routes.push(Route {
                    from: j.at,
                    targets: vec![(*p, Dir::Left), (*p, Dir::Right), (*p, Dir::Down)],
                });
            }
        }

        let mut paths = b.paths;
        let unrouted = route::route_all(&b.shapes, &b.labels, &mut paths, &b.routes);
        let mut d = Diagram::new(f.name.clone(), b.shapes, paths, b.labels);
        d.unrouted = unrouted;
        d
    }

    fn terminator(&self, text: &str) -> Block {
        let mut b = Block::shape(ShapeKind::Terminator, text);
        b.exit = None;
        b
    }

    fn seq(&self, stmts: &[Stmt], ctx: Ctx) -> Block {
        stack(stmts.iter().map(|s| self.stmt(s, ctx)).collect())
    }

    fn stmt(&self, s: &Stmt, ctx: Ctx) -> Block {
        let w = self.words;
        match s {
            Stmt::Process(t) => Block::shape(ShapeKind::Process, t),
            Stmt::Io(t) => Block::shape(ShapeKind::Io, t),
            Stmt::Call(t) => Block::shape(ShapeKind::Call, t),
            Stmt::Prep(t) => Block::shape(ShapeKind::Prep, t),
            Stmt::If {
                cond,
                then,
                otherwise,
            } => self.if_block(cond, then, otherwise, ctx),
            Stmt::While { cond, body } => self.loop_block(cond, body, None),
            Stmt::For {
                init,
                cond,
                update,
                body,
            } => {
                let cond = cond.as_deref().unwrap_or(w.always);
                let lp = self.loop_block(cond, body, update.as_deref());
                match init {
                    Some(i) => stack(vec![Block::shape(ShapeKind::Process, i), lp]),
                    None => lp,
                }
            }
            Stmt::DoWhile { body, cond } => self.do_block(body, cond),
            Stmt::Switch { expr, cases } => self.switch_block(expr, cases),
            Stmt::Break => jump(JumpKind::Break),
            Stmt::Continue => jump(JumpKind::Continue),
            Stmt::Goto(l) => jump(JumpKind::Goto(l.clone())),
            Stmt::Return(v) => {
                // Returning from the entry point ends the program.
                let text = match v {
                    _ if self.entry => w.end.to_string(),
                    Some(v) if !v.is_empty() => format!("{}: {v}", w.ret),
                    _ => w.ret.to_string(),
                };
                self.terminator(&text)
            }
            Stmt::Label(name) => {
                let mut b = Block::line();
                b.anchors.push((name.clone(), (0, 0)));
                let text = format!("{name}:");
                b.label(-2 - width(&text), 0, &text);
                b
            }
        }
    }

    fn decision(&self, cond: &str, y: i32) -> Shape {
        let mut d = Shape::new(ShapeKind::Decision, cond);
        d.y = y;
        d
    }

    fn if_block(&self, cond: &str, then: &[Stmt], otherwise: &[Stmt], ctx: Ctx) -> Block {
        let (yes, no) = (self.words.yes, self.words.no);
        let d = self.decision(cond, 0);
        let (d_left, d_right) = (d.x, d.right());
        let mut out = Block {
            left: d_left,
            right: d_right,
            ..Block::default()
        };
        out.shapes.push(d);

        let t = self.seq(then, ctx);
        let e = self.seq(otherwise, ctx);
        let on_left = ctx.break_left && leaves_with(then, true) && !leaves_with(then, false);

        // "False": straight down.
        let e_top = if e.entry_line { DH + 1 } else { DH + GAP };
        out.paths.push(connector(0, DH, e_top, e.entry_line));
        out.label(-2 - width(no), DH, no);
        let e_exit = e.exit.map(|v| v + e_top);
        let e_bottom = e_top + e.height;
        out.place(e, 0, e_top);

        // "True": to one side.
        let t_top = MID + 3;
        let t_end = t_top - i32::from(!t.entry_line);
        let t_endk = if t.entry_line { End::Plain } else { End::Shape };
        let tx = if on_left {
            let tx = out.left - HGAP - t.right;
            out.path(vec![(d_left - 1, MID), (tx, MID), (tx, t_end)], t_endk);
            out.label(d_left - 1 - width(yes), MID - 1, yes);
            tx
        } else {
            let tx = out.right.max(d_right + 2) + HGAP - t.left;
            out.path(vec![(d_right + 1, MID), (tx, MID), (tx, t_end)], t_endk);
            out.label(d_right + 2, MID - 1, yes);
            tx
        };
        let t_exit = t.exit.map(|v| v + t_top);
        let t_bottom = t_top + t.height;
        out.place(t, tx, t_top);

        let bottom = e_bottom.max(t_bottom);
        if t_exit.is_none() {
            // Nothing comes back from the side: no join needed.
            out.height = bottom;
            out.exit = e_exit;
            return out;
        }
        let m = bottom + 1;
        if let Some(ee) = e_exit {
            out.path(vec![(0, ee), (0, m)], End::Plain);
        }
        if let Some(te) = t_exit {
            let end = if e_exit.is_some() {
                End::Join
            } else {
                End::Plain
            };
            out.path(vec![(tx, te), (tx, m), (0, m)], end);
        }
        out.height = m + 1;
        out.exit = Some(m);
        out
    }

    /// `while` and `for`: join, condition, body (and update) on the right,
    /// back edge on the far right, "False" straight down.
    fn loop_block(&self, cond: &str, body: &[Stmt], update: Option<&str>) -> Block {
        let (yes, no) = (self.words.yes, self.words.no);
        let ctx = Ctx { break_left: true };
        let mut out = Block {
            entry_line: true,
            ..Block::default()
        };
        let d0 = 3;
        let d = self.decision(cond, d0);
        let (d_left, d_right) = (d.x, d.right());
        out.span(d_left, d_right);
        out.path(vec![(0, 0), (0, d0 - 1)], End::Shape);
        out.shapes.push(d);

        let b = self.seq(body, ctx);
        // `continue` joins just above the update; without one, the back edge.
        let (col, mut cont_targets) = match update {
            Some(u) => {
                let ub = Block::shape(ShapeKind::Process, u);
                let mut c = Block {
                    entry_line: b.entry_line,
                    ..Block::default()
                };
                let (b_exit, b_h) = (b.exit, b.height);
                c.place(b, 0, 0);
                let j = b_h;
                let u_top = j + 2;
                if let Some(e) = b_exit {
                    c.path(vec![(0, e), (0, j)], End::Plain);
                }
                c.path(vec![(0, j), (0, u_top - 1)], End::Shape);
                c.exit = ub.exit.map(|v| v + u_top);
                c.height = u_top + ub.height;
                c.place(ub, 0, u_top);
                (
                    c,
                    vec![
                        ((0, j), Dir::Left),
                        ((0, j), Dir::Right),
                        ((0, j), Dir::Down),
                    ],
                )
            }
            None => (b, Vec::new()),
        };

        let b_top = d0 + MID + 3;
        let bx = d_right + HGAP - col.left;
        let b_end = b_top - i32::from(!col.entry_line);
        let b_endk = if col.entry_line {
            End::Plain
        } else {
            End::Shape
        };
        out.path(
            vec![(d_right + 1, d0 + MID), (bx, d0 + MID), (bx, b_end)],
            b_endk,
        );
        out.label(d_right + 2, d0 + MID - 1, yes);
        for t in &mut cont_targets {
            t.0 = (t.0.0 + bx, t.0.1 + b_top);
        }
        let col_exit = col.exit.map(|v| v + b_top);
        let be = b_top + col.height + 1;
        // The back edge passes right of the "True" label.
        let rx = (bx + col.right + 3).max(d_right + 3 + width(yes));
        let needs_rail = update.is_none() && leaves_with(body, false);
        out.place(col, bx, b_top);

        if let Some(e) = col_exit {
            out.path(
                vec![(bx, e), (bx, be), (rx, be), (rx, 0), (0, 0)],
                End::Join,
            );
        } else if needs_rail {
            out.path(vec![(rx, be), (rx, 0), (0, 0)], End::Join);
            cont_targets.push(((rx, be), Dir::Right));
            cont_targets.push(((rx, be), Dir::Up));
        }
        if update.is_none() {
            for y in 1..be {
                cont_targets.push(((rx, y), Dir::Right));
            }
        }

        let xe = be + 1;
        out.path(vec![(0, d0 + DH), (0, xe)], End::Plain);
        out.label(-2 - width(no), d0 + DH, no);
        let break_targets = side_targets((0, d0 + DH), (0, xe));

        out.resolve(is_break, &break_targets);
        out.resolve(is_continue, &cont_targets);
        out.height = xe + 1;
        out.exit = Some(xe);
        out
    }

    /// `do ... while`: join, body, condition, "True" back up on the right.
    fn do_block(&self, body: &[Stmt], cond: &str) -> Block {
        let (yes, no) = (self.words.yes, self.words.no);
        let ctx = Ctx { break_left: true };
        let mut out = Block {
            entry_line: true,
            ..Block::default()
        };
        let b = self.seq(body, ctx);
        let b_top = 3;
        out.paths.push(connector(0, 0, b_top, b.entry_line));
        let (b_exit, b_h) = (b.exit.map(|v| v + b_top), b.height);
        out.place(b, 0, b_top);

        let j = b_top + b_h;
        let d0 = j + 2;
        if let Some(e) = b_exit {
            out.path(vec![(0, e), (0, j)], End::Plain);
        }
        out.path(vec![(0, j), (0, d0 - 1)], End::Shape);
        let d = self.decision(cond, d0);
        let d_right = d.right();
        out.span(d.x, d_right);
        out.shapes.push(d);

        let rx = (out.right.max(d_right) + 3).max(d_right + 3 + width(yes));
        out.path(
            vec![(d_right + 1, d0 + MID), (rx, d0 + MID), (rx, 0), (0, 0)],
            End::Join,
        );
        out.label(d_right + 2, d0 + MID - 1, yes);

        let xe = d0 + DH + 2;
        out.path(vec![(0, d0 + DH), (0, xe)], End::Plain);
        out.label(-2 - width(no), d0 + DH, no);

        let cont = [
            ((0, j), Dir::Left),
            ((0, j), Dir::Right),
            ((0, j), Dir::Down),
        ];
        out.resolve(is_continue, &cont);
        out.resolve(is_break, &side_targets((0, d0 + DH), (0, xe)));
        out.height = xe + 1;
        out.exit = Some(xe);
        out
    }

    /// `switch`: the expression, then one case symbol per branch down the
    /// left, bodies in a column on the right. A trailing `break` leaves
    /// through a rail on the far right.
    fn switch_block(&self, expr: &str, cases: &[Case]) -> Block {
        let (yes, no) = (self.words.yes, self.words.no);
        let ctx = Ctx { break_left: false };
        let d = self.decision(expr, 0);
        let d_right = d.right();
        let mut out = Block {
            left: d.x,
            right: d_right,
            ..Block::default()
        };
        out.shapes.push(d);

        struct Branch {
            node: Shape,
            body: Block,
            brk: bool,
            default: bool,
        }
        let mut branches: Vec<Branch> = cases
            .iter()
            .map(|c| {
                let brk = c.body.last() == Some(&Stmt::Break);
                let body = if brk {
                    &c.body[..c.body.len() - 1]
                } else {
                    &c.body[..]
                };
                let mut text: Vec<&str> = c.values.iter().map(String::as_str).collect();
                if c.default {
                    text.push(self.words.default);
                }
                Branch {
                    node: Shape::new(ShapeKind::Case, &text.join(", ")),
                    body: self.seq(body, ctx),
                    brk,
                    default: c.default,
                }
            })
            .collect();
        let n = branches.len();
        let default_at = branches.iter().position(|b| b.default);
        let default_middle = default_at.is_some_and(|i| i + 1 != n);

        let nodes_right = branches.iter().map(|b| b.node.right()).max().unwrap_or(0);
        let body_left = branches.iter().map(|b| b.body.left).min().unwrap_or(0);
        let body_right = branches.iter().map(|b| b.body.right).max().unwrap_or(0);
        let edge = nodes_right.max(d_right);
        let bx = (edge + HGAP - body_left).max(edge + 3 + width(yes));
        let kx = bx + body_right + 3;

        let mut r = DH + GAP;
        let mut chain: Vec<i32> = Vec::new();
        let mut default_row = None;
        let mut prev_exit: Option<i32> = None;
        let mut break_exits = Vec::new();
        for (i, br) in branches.iter_mut().enumerate() {
            let body = std::mem::take(&mut br.body);
            let b_top = r + 2;
            let b_end = b_top - i32::from(!body.entry_line);
            let b_endk = if body.entry_line {
                End::Plain
            } else {
                End::Shape
            };
            if default_middle && Some(i) == default_at {
                // Reached from the case symbol placed at the end of the chain.
                default_row = Some(r);
                out.path(vec![(bx, r), (bx, b_end)], b_endk);
            } else {
                let mut node = br.node.clone();
                node.y = r;
                out.path(vec![(node.right() + 1, r), (bx, r), (bx, b_end)], b_endk);
                out.label(node.right() + 2, r - 1, yes);
                out.span(node.x, node.right());
                out.shapes.push(node);
                chain.push(r);
            }
            if let Some(pe) = prev_exit {
                out.path(vec![(bx, pe), (bx, r)], End::Plain);
            }
            let exit = body.exit.map(|v| v + b_top);
            let end_row = b_top + body.height;
            out.place(body, bx, b_top);
            prev_exit = None;
            match exit {
                Some(e) if br.brk => break_exits.push(e),
                Some(e) => prev_exit = Some(e),
                None => {}
            }
            r = (r + 3).max(end_row + 2);
        }

        let has_default = default_at.is_some();
        let xe = if default_middle { r + 3 } else { r };
        if let (true, Some(di)) = (default_middle, default_at) {
            let mut node = branches[di].node.clone();
            node.y = r;
            out.span(node.x, node.right());
            if let Some(dr) = default_row {
                out.routes.push(Route {
                    from: (node.right() + 1, r),
                    targets: vec![((bx, dr), Dir::Right)],
                });
            }
            out.shapes.push(node);
            chain.push(r);
        }
        if let Some(&first) = chain.first() {
            out.path(vec![(0, DH), (0, first - 1)], End::Shape);
        }
        for w in chain.windows(2) {
            out.path(vec![(0, w[0] + 1), (0, w[1] - 1)], End::Shape);
            out.label(-2 - width(no), w[0] + 1, no);
        }

        let mut break_targets = vec![
            ((0, xe), Dir::Down),
            ((0, xe), Dir::Left),
            ((0, xe), Dir::Right),
        ];
        let end = if has_default { End::Plain } else { End::Join };
        if !has_default && let Some(&last) = chain.last() {
            out.path(vec![(0, last + 1), (0, xe)], End::Plain);
            out.label(-2 - width(no), last + 1, no);
            break_targets.extend(side_targets((0, last + 1), (0, xe)));
        }
        for &e in &break_exits {
            out.path(
                vec![(bx, e), (bx, e + 1), (kx, e + 1), (kx, xe), (0, xe)],
                end,
            );
            break_targets.extend(side_targets((bx, e + 1), (kx, e + 1)));
            break_targets.extend(
                side_targets((kx, e + 1), (kx, xe))
                    .into_iter()
                    .filter(|t| t.1 == Dir::Right),
            );
        }
        if let Some(pe) = prev_exit {
            out.path(vec![(bx, pe), (bx, xe), (0, xe)], end);
        }
        if !break_exits.is_empty() || prev_exit.is_some() {
            let far = if break_exits.is_empty() { bx } else { kx };
            break_targets.extend(
                side_targets((far, xe), (0, xe))
                    .into_iter()
                    .filter(|t| t.1 == Dir::Down && t.0.0 > 1 && t.0.0 != bx),
            );
        }
        out.resolve(is_break, &break_targets);

        let exits = !has_default || !break_exits.is_empty() || prev_exit.is_some();
        out.height = xe + 1;
        out.exit = exits.then_some(xe);
        if n == 0 {
            out.exit = Some(xe);
            out.path(vec![(0, DH), (0, xe)], End::Plain);
        }
        out
    }
}

fn jump(kind: JumpKind) -> Block {
    let mut b = Block::line();
    b.exit = None;
    b.jumps.push(Jump { kind, at: (0, 0) });
    b
}

/// Blocks one below the other, connected in column 0.
fn stack(blocks: Vec<Block>) -> Block {
    let Some(first) = blocks.first() else {
        return Block::line();
    };
    let mut out = Block {
        entry_line: first.entry_line,
        ..Block::default()
    };
    let mut y = 0;
    let mut prev_exit = None;
    for (i, b) in blocks.into_iter().enumerate() {
        let top = match i {
            0 => 0,
            _ if b.entry_line => y + GAP - 1,
            _ => y + GAP,
        };
        if let Some(e) = prev_exit {
            out.paths.push(connector(0, e, top, b.entry_line));
        }
        prev_exit = b.exit.map(|v| v + top);
        y = top + b.height;
        out.place(b, 0, top);
    }
    out.height = y;
    out.exit = prev_exit;
    out
}
