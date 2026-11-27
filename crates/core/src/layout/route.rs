// SPDX-License-Identifier: AGPL-3.0-or-later
//! Routing of jumps (`break`, `continue`, `goto`) once everything else is
//! drawn.
//!
//! A jump is a line from the point where the jump happens to one of the
//! cells where it may join its target (the exit line of a loop, the back
//! edge, a label...). The search (A*) runs on the cell grid: symbols and
//! labels are walls, a line may cross another straight line at a right
//! angle, and every bend and crossing costs extra, so the shortest clean
//! route wins.

use super::geom::{DOWN, Dir, End, LEFT, Label, Path, Pt, RIGHT, Shape, UP, step};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub(crate) struct Route {
    pub from: Pt,
    /// Cells the jump may end on, with the direction it must travel to
    /// enter them.
    pub targets: Vec<(Pt, Dir)>,
}

impl Route {
    pub fn moved(mut self, dx: i32, dy: i32) -> Route {
        self.from = (self.from.0 + dx, self.from.1 + dy);
        for t in &mut self.targets {
            t.0 = (t.0.0 + dx, t.0.1 + dy);
        }
        self
    }
}

const BEND: u32 = 4;
const CROSS: u32 = 25;
const MARGIN: i32 = 8;

struct Grid {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    wall: Vec<bool>,
    mask: Vec<u8>,
}

impl Grid {
    fn idx(&self, p: Pt) -> Option<usize> {
        let (x, y) = (p.0 - self.x0, p.1 - self.y0);
        (x >= 0 && y >= 0 && x < self.w && y < self.h)
            .then(|| usize::try_from(y * self.w + x).expect("inside the grid"))
    }

    fn add(&mut self, path: &Path) {
        for (p, m) in path.masks() {
            if let Some(i) = self.idx(p) {
                self.mask[i] |= m;
            }
        }
        if let Some((p, _)) = path.arrow()
            && let Some(i) = self.idx(p)
        {
            self.wall[i] = true;
        }
    }
}

/// Routes every jump and adds its line to `paths`. A jump that finds no
/// route is left out; returns how many were.
pub(crate) fn route_all(
    shapes: &[Shape],
    labels: &[Label],
    paths: &mut Vec<Path>,
    routes: &[Route],
) -> usize {
    if routes.is_empty() {
        return 0;
    }
    let mut min = (i32::MAX, i32::MAX);
    let mut max = (i32::MIN, i32::MIN);
    let mut add = |p: Pt| {
        min = (min.0.min(p.0), min.1.min(p.1));
        max = (max.0.max(p.0), max.1.max(p.1));
    };
    for s in shapes {
        add((s.x, s.y));
        add((s.right(), s.bottom()));
    }
    for p in paths.iter() {
        p.points.iter().for_each(|&q| add(q));
    }
    for r in routes {
        add(r.from);
        r.targets.iter().for_each(|t| add(t.0));
    }
    let (x0, y0) = (min.0 - MARGIN, min.1 - 2);
    let (w, h) = (max.0 - x0 + MARGIN + 1, max.1 - y0 + MARGIN);
    let size = usize::try_from(w * h).expect("positive size");
    let mut grid = Grid {
        x0,
        y0,
        w,
        h,
        wall: vec![false; size],
        mask: vec![0; size],
    };
    for s in shapes {
        for y in s.y..=s.bottom() {
            for x in s.x - 1..=s.right() + 1 {
                if let Some(i) = grid.idx((x, y)) {
                    grid.wall[i] = true;
                }
            }
        }
    }
    for l in labels {
        for x in l.x - 1..=l.right() + 1 {
            if let Some(i) = grid.idx((x, l.y)) {
                grid.wall[i] = true;
            }
        }
    }
    for p in paths.iter() {
        grid.add(p);
    }
    let mut failed = 0;
    for r in routes {
        if let Some(p) = search(&grid, r) {
            grid.add(&p);
            paths.push(p);
        } else {
            failed += 1;
        }
    }
    failed
}

fn search(g: &Grid, r: &Route) -> Option<Path> {
    let start = g.idx(r.from)?;
    let states = g.mask.len() * 4;
    let mut cost = vec![u32::MAX; states];
    let mut prev = vec![usize::MAX; states];
    let mut heap = BinaryHeap::new();
    let state = |i: usize, d: Dir| i * 4 + d.index();
    let pt = |i: usize| -> Pt {
        let w = usize::try_from(g.w).expect("positive width");
        (
            g.x0 + i32::try_from(i % w).expect("fits"),
            g.y0 + i32::try_from(i / w).expect("fits"),
        )
    };
    let h = |p: Pt| -> u32 {
        r.targets
            .iter()
            .map(|(t, _)| (t.0 - p.0).unsigned_abs() + (t.1 - p.1).unsigned_abs())
            .min()
            .unwrap_or(0)
    };
    let straight = |m: u8| m == UP | DOWN || m == LEFT | RIGHT;
    // A cell the line may enter while moving in direction `d`, and whether
    // entering it is a crossing.
    let enter = |p: Pt, d: Dir| -> Option<(usize, bool)> {
        let i = g.idx(p)?;
        if g.wall[i] {
            return None;
        }
        match g.mask[i] {
            0 => Some((i, false)),
            m if straight(m) && m & d.bit() == 0 => Some((i, true)),
            _ => None,
        }
    };

    for d in Dir::ALL {
        if g.mask[start] & d.bit() != 0 {
            continue;
        }
        if let Some((i, cross)) = enter(step(r.from, d), d) {
            let c = 1 + if cross { CROSS } else { 0 };
            let s = state(i, d);
            cost[s] = c;
            prev[s] = usize::MAX - 1;
            heap.push(Reverse((c + h(pt(i)), c, s)));
        }
    }

    let mut goal: Option<(usize, Pt)> = None;
    while let Some(Reverse((_, c, s))) = heap.pop() {
        if c > cost[s] {
            continue;
        }
        let (i, d) = (s / 4, Dir::ALL[s % 4]);
        let p = pt(i);
        let crossing = g.mask[i] != 0;
        // The arrowhead goes on this cell, so it must be free.
        if !crossing
            && let Some(&(t, _)) = r.targets.iter().find(|(t, td)| {
                // The last move is straight, so the arrowhead is no corner.
                *td == d
                    && step(p, *td) == *t
                    && g.idx(*t)
                        .is_some_and(|ti| !g.wall[ti] && g.mask[ti] & td.opposite().bit() == 0)
            })
        {
            goal = Some((s, t));
            break;
        }
        for nd in Dir::ALL {
            if nd == d.opposite() || (crossing && nd != d) {
                continue;
            }
            let Some((ni, cross)) = enter(step(p, nd), nd) else {
                continue;
            };
            let nc = c + 1 + if nd != d { BEND } else { 0 } + if cross { CROSS } else { 0 };
            let ns = state(ni, nd);
            if nc < cost[ns] {
                cost[ns] = nc;
                prev[ns] = s;
                heap.push(Reverse((nc + h(pt(ni)), nc, ns)));
            }
        }
    }

    let (mut s, target) = goal?;
    let mut cells = vec![target];
    loop {
        cells.push(pt(s / 4));
        let p = prev[s];
        if p == usize::MAX - 1 {
            break;
        }
        s = p;
    }
    cells.push(r.from);
    cells.reverse();
    Some(Path::new(cells, End::Join))
}
