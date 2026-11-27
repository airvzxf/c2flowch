//! Levels of detail.
//!
//! | Level | Shows                                                           |
//! |-------|-----------------------------------------------------------------|
//! | 1     | Calls to functions of the file, and the structures around them  |
//! | 2     | Every control structure, input/output and calls                 |
//! | 3     | Level 2 plus assignments and other operations (default)         |
//! | 4     | Level 3 plus declarations without a value                       |
//! | 5     | Every statement in its own symbol (nothing is grouped)          |
//!
//! At levels 1 to 4, consecutive statements of the same kind share one
//! symbol, one statement per line.

use crate::ir::{Case, Stmt};

pub(crate) fn apply(body: Vec<Stmt>, level: u8) -> Vec<Stmt> {
    let body = filter(body, level);
    if level >= 5 { body } else { group(body) }
}

fn filter(body: Vec<Stmt>, level: u8) -> Vec<Stmt> {
    body.into_iter()
        .filter_map(|s| filter_stmt(s, level))
        .collect()
}

fn filter_stmt(s: Stmt, level: u8) -> Option<Stmt> {
    let keep = match &s {
        Stmt::Prep(_) => level >= 4,
        Stmt::Process(_) => level >= 3,
        Stmt::Io(_) => level >= 2,
        _ => true,
    };
    if !keep {
        return None;
    }
    let s = match s {
        Stmt::If {
            cond,
            then,
            otherwise,
        } => Stmt::If {
            cond,
            then: filter(then, level),
            otherwise: filter(otherwise, level),
        },
        Stmt::While { cond, body } => Stmt::While {
            cond,
            body: filter(body, level),
        },
        Stmt::DoWhile { body, cond } => Stmt::DoWhile {
            body: filter(body, level),
            cond,
        },
        Stmt::For {
            init,
            cond,
            update,
            body,
        } => {
            let (init, update) = if level >= 3 {
                (init, update)
            } else {
                (None, None)
            };
            Stmt::For {
                init,
                cond,
                update,
                body: filter(body, level),
            }
        }
        Stmt::Switch { expr, cases } => Stmt::Switch {
            expr,
            cases: cases
                .into_iter()
                .map(|c| Case {
                    body: filter(c.body, level),
                    ..c
                })
                .collect(),
        },
        other => other,
    };
    // The executive summary keeps a structure only if it leads to a call.
    let structure = matches!(
        s,
        Stmt::If { .. }
            | Stmt::While { .. }
            | Stmt::DoWhile { .. }
            | Stmt::For { .. }
            | Stmt::Switch { .. }
    );
    if level <= 1 && structure && !has_call(std::slice::from_ref(&s)) {
        return None;
    }
    Some(s)
}

fn has_call(body: &[Stmt]) -> bool {
    body.iter().any(|s| match s {
        Stmt::Call(_) => true,
        Stmt::If {
            then, otherwise, ..
        } => has_call(then) || has_call(otherwise),
        Stmt::While { body, .. } | Stmt::DoWhile { body, .. } | Stmt::For { body, .. } => {
            has_call(body)
        }
        Stmt::Switch { cases, .. } => cases.iter().any(|c| has_call(&c.body)),
        _ => false,
    })
}

/// Merges consecutive simple statements of the same kind.
fn group(body: Vec<Stmt>) -> Vec<Stmt> {
    let mut out: Vec<Stmt> = Vec::new();
    for s in body {
        let s = match s {
            Stmt::If {
                cond,
                then,
                otherwise,
            } => Stmt::If {
                cond,
                then: group(then),
                otherwise: group(otherwise),
            },
            Stmt::While { cond, body } => Stmt::While {
                cond,
                body: group(body),
            },
            Stmt::DoWhile { body, cond } => Stmt::DoWhile {
                body: group(body),
                cond,
            },
            Stmt::For {
                init,
                cond,
                update,
                body,
            } => Stmt::For {
                init,
                cond,
                update,
                body: group(body),
            },
            Stmt::Switch { expr, cases } => Stmt::Switch {
                expr,
                cases: cases
                    .into_iter()
                    .map(|c| Case {
                        body: group(c.body),
                        ..c
                    })
                    .collect(),
            },
            other => other,
        };
        match (out.last_mut(), s) {
            (Some(Stmt::Process(a)), Stmt::Process(b))
            | (Some(Stmt::Io(a)), Stmt::Io(b))
            | (Some(Stmt::Prep(a)), Stmt::Prep(b)) => {
                a.push('\n');
                a.push_str(&b);
            }
            (_, s) => out.push(s),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Stmt> {
        vec![
            Stmt::Prep("int x".into()),
            Stmt::Process("x ← 0".into()),
            Stmt::Process("y ← 1".into()),
            Stmt::Io("Print x".into()),
            Stmt::While {
                cond: "x < 3".into(),
                body: vec![Stmt::Process("x ← x + 1".into())],
            },
            Stmt::If {
                cond: "y".into(),
                then: vec![Stmt::Call("f()".into())],
                otherwise: vec![],
            },
        ]
    }

    #[test]
    fn every_level_is_different() {
        let all: Vec<Vec<Stmt>> = (1..=5).map(|l| apply(sample(), l)).collect();
        for i in 0..all.len() {
            for j in i + 1..all.len() {
                assert_ne!(all[i], all[j], "levels {} and {}", i + 1, j + 1);
            }
        }
    }

    #[test]
    fn level_one_keeps_only_paths_to_calls() {
        assert_eq!(
            apply(sample(), 1),
            vec![Stmt::If {
                cond: "y".into(),
                then: vec![Stmt::Call("f()".into())],
                otherwise: vec![],
            }]
        );
    }

    #[test]
    fn grouping() {
        assert_eq!(apply(sample(), 3)[0], Stmt::Process("x ← 0\ny ← 1".into()));
    }
}
