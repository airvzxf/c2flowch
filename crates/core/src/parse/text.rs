//! Turns C expressions (syntax tree nodes) into flowchart labels.
//!
//! The labels read as pseudocode: `=` becomes `←`, `==` becomes `=`,
//! `!=` becomes `≠`, pointer syntax (`*p`, `&x`, `p->f`) is dropped, casts
//! are removed and `printf`/`scanf` become "Print …"/"Read …".

use crate::i18n::Words;
use tree_sitter::Node;

pub(crate) struct Text<'a> {
    pub src: &'a str,
    pub words: &'static Words,
}

/// Library functions that write to the screen or a stream.
const OUTPUT: &[&str] = &[
    "printf", "fprintf", "puts", "fputs", "putchar", "putc", "fputc",
];
/// Library functions that read from the keyboard or a stream.
const INPUT: &[&str] = &[
    "scanf", "fscanf", "gets", "fgets", "getchar", "getc", "fgetc",
];

pub(crate) fn named_children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c)
        .filter(|ch| ch.kind() != "comment")
        .collect()
}

/// The name of the function a call expression calls, if it is a plain name.
pub(crate) fn callee<'s>(n: Node, src: &'s str) -> Option<&'s str> {
    if n.kind() != "call_expression" {
        return None;
    }
    let f = n.child_by_field_name("function")?;
    (f.kind() == "identifier").then(|| &src[f.byte_range()])
}

impl Text<'_> {
    /// Source text of a node with all runs of whitespace collapsed.
    pub fn raw(&self, n: Node) -> String {
        self.src[n.byte_range()]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Label for an expression used as a whole statement.
    pub fn statement(&self, n: Node) -> String {
        match n.kind() {
            "assignment_expression" => self.assignment(n),
            "update_expression" => {
                let arg = n.child_by_field_name("argument").map(|a| self.expr(a));
                let op = n.child_by_field_name("operator").map(|o| self.raw(o));
                match (arg, op.as_deref()) {
                    (Some(a), Some("++")) => format!("{a} ← {a} + 1"),
                    (Some(a), Some("--")) => format!("{a} ← {a} - 1"),
                    _ => self.raw(n),
                }
            }
            "parenthesized_expression" => named_children(n)
                .first()
                .map_or_else(|| self.raw(n), |inner| self.statement(*inner)),
            _ => self.expr(n),
        }
    }

    /// Label for a condition (if, while, for, do-while).
    pub fn condition(&self, n: Node) -> String {
        if n.kind() == "parenthesized_expression"
            && let [inner] = named_children(n)[..]
        {
            return self.condition(inner);
        }
        self.boolean(n)
    }

    fn assignment(&self, n: Node) -> String {
        let (Some(l), Some(op), Some(r)) = (
            n.child_by_field_name("left"),
            n.child_by_field_name("operator"),
            n.child_by_field_name("right"),
        ) else {
            return self.raw(n);
        };
        let left = self.expr(l);
        let right = self.expr(r);
        let op = self.raw(op);
        if op == "=" {
            return format!("{left} ← {right}");
        }
        let bin = op.trim_end_matches('=');
        let compound = matches!(
            r.kind(),
            "binary_expression" | "conditional_expression" | "assignment_expression"
        );
        if compound {
            format!("{left} ← {left} {bin} ({right})")
        } else {
            format!("{left} ← {left} {bin} {right}")
        }
    }

    /// An expression in a true/false position. `strcmp(a, b)` alone means
    /// "a and b differ", so it is shown as `a ≠ b`.
    fn boolean(&self, n: Node) -> String {
        match n.kind() {
            "call_expression" => match self.strcmp_args(n) {
                Some((a, b)) => format!("{a} ≠ {b}"),
                None => self.expr(n),
            },
            "parenthesized_expression" => match named_children(n)[..] {
                [inner] => format!("({})", self.boolean(inner)),
                _ => self.raw(n),
            },
            _ => self.expr(n),
        }
    }

    /// Arguments of a `strcmp`/`strncmp` call.
    fn strcmp_args(&self, n: Node) -> Option<(String, String)> {
        let name = callee(n, self.src)?;
        if name != "strcmp" && name != "strncmp" {
            return None;
        }
        let args = named_children(n.child_by_field_name("arguments")?);
        match args[..] {
            [a, b, ..] => Some((self.expr(a), self.expr(b))),
            _ => None,
        }
    }

    pub fn expr(&self, n: Node) -> String {
        let w = self.words;
        match n.kind() {
            "parenthesized_expression" => match named_children(n)[..] {
                [inner] => format!("({})", self.expr(inner)),
                _ => self.raw(n),
            },
            "assignment_expression" => self.assignment(n),
            "update_expression" => {
                let arg = n.child_by_field_name("argument");
                let op = n.child_by_field_name("operator");
                match (arg, op) {
                    (Some(a), Some(o)) if o.start_byte() < a.start_byte() => {
                        format!("{}{}", self.raw(o), self.expr(a))
                    }
                    (Some(a), Some(o)) => format!("{}{}", self.expr(a), self.raw(o)),
                    _ => self.raw(n),
                }
            }
            "binary_expression" => self.binary(n),
            "unary_expression" => {
                let (Some(op), Some(arg)) = (
                    n.child_by_field_name("operator"),
                    n.child_by_field_name("argument"),
                ) else {
                    return self.raw(n);
                };
                let op = self.raw(op);
                if op == "!" {
                    if let Some((a, b)) = self.strcmp_args(arg) {
                        return format!("{a} = {b}");
                    }
                    return format!("{} {}", w.not, self.boolean(arg));
                }
                format!("{op}{}", self.expr(arg))
            }
            // `*p` and `&x` read as plain `p` and `x` in a flowchart.
            "pointer_expression" => n
                .child_by_field_name("argument")
                .map_or_else(|| self.raw(n), |a| self.expr(a)),
            "field_expression" => {
                let (Some(a), Some(f)) = (
                    n.child_by_field_name("argument"),
                    n.child_by_field_name("field"),
                ) else {
                    return self.raw(n);
                };
                format!("{}.{}", self.expr(a), self.raw(f))
            }
            "call_expression" => {
                let f = n
                    .child_by_field_name("function")
                    .map_or_else(String::new, |f| self.expr(f));
                let args = n
                    .child_by_field_name("arguments")
                    .map(|a| self.args(a).join(", "))
                    .unwrap_or_default();
                format!("{f}({args})")
            }
            "conditional_expression" => {
                let part = |name| n.child_by_field_name(name).map(|c| self.expr(c));
                match (part("condition"), part("consequence"), part("alternative")) {
                    (Some(c), Some(a), Some(b)) => format!("{c} ? {a} : {b}"),
                    _ => self.raw(n),
                }
            }
            "cast_expression" => n
                .child_by_field_name("value")
                .map_or_else(|| self.raw(n), |v| self.expr(v)),
            "subscript_expression" => match named_children(n)[..] {
                [a, i] => format!("{}[{}]", self.expr(a), self.expr(i)),
                _ => self.raw(n),
            },
            "comma_expression" => {
                let part = |name| n.child_by_field_name(name).map(|c| self.expr(c));
                match (part("left"), part("right")) {
                    (Some(l), Some(r)) => format!("{l}, {r}"),
                    _ => self.raw(n),
                }
            }
            "string_literal" => strip_newlines(&self.raw(n)),
            "concatenated_string" => {
                let parts: Vec<String> = named_children(n)
                    .into_iter()
                    .map(|p| self.expr(p))
                    .collect();
                parts.join(" ")
            }
            "initializer_list" => {
                let items: Vec<String> = named_children(n)
                    .into_iter()
                    .map(|p| self.expr(p))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            _ => self.raw(n),
        }
    }

    fn args(&self, list: Node) -> Vec<String> {
        named_children(list)
            .into_iter()
            .map(|a| self.expr(a))
            .collect()
    }

    fn binary(&self, n: Node) -> String {
        let w = self.words;
        let (Some(l), Some(op), Some(r)) = (
            n.child_by_field_name("left"),
            n.child_by_field_name("operator"),
            n.child_by_field_name("right"),
        ) else {
            return self.raw(n);
        };
        let op = self.raw(op);

        // strcmp(a, b) == 0  →  a = b, and the same for the other relations.
        let zero = |x: Node| x.kind() == "number_literal" && self.raw(x) == "0";
        if let Some(rel) = relation(&op) {
            if zero(r)
                && let Some((a, b)) = self.strcmp_args(l)
            {
                return format!("{a} {rel} {b}");
            }
            if zero(l)
                && let Some((a, b)) = self.strcmp_args(r)
            {
                return format!("{a} {} {b}", relation(&flip(&op)).unwrap_or(rel));
            }
        }

        match op.as_str() {
            "&&" => format!("{} {} {}", self.boolean(l), w.and, self.boolean(r)),
            "||" => format!("{} {} {}", self.boolean(l), w.or, self.boolean(r)),
            _ => {
                let shown = relation(&op).unwrap_or(&op);
                format!("{} {shown} {}", self.expr(l), self.expr(r))
            }
        }
    }

    /// Label for an input/output call, or `None` when `n` is not one.
    /// `target` is the variable that receives the value of the call, as in
    /// `c = getchar()`.
    pub fn io(&self, n: Node, target: Option<&str>) -> Option<String> {
        let name = callee(n, self.src)?;
        let args: Vec<Node> = n
            .child_by_field_name("arguments")
            .map(named_children)
            .unwrap_or_default();
        let w = self.words;
        if OUTPUT.contains(&name) {
            if target.is_some() {
                return None;
            }
            let shown = match (name, &args[..]) {
                ("printf", [fmt, rest @ ..]) | ("fprintf", [_, fmt, rest @ ..]) => {
                    self.format(*fmt, rest)
                }
                (_, [first, ..]) => self.expr(*first),
                _ => String::new(),
            };
            return Some(if shown.is_empty() {
                w.print.to_string()
            } else {
                format!("{} {shown}", w.print)
            });
        }
        if INPUT.contains(&name) {
            let vars: Vec<String> = match (name, &args[..]) {
                ("scanf", [_, rest @ ..]) | ("fscanf", [_, _, rest @ ..]) => {
                    rest.iter().map(|a| self.expr(*a)).collect()
                }
                ("gets" | "fgets", [buf, ..]) => vec![self.expr(*buf)],
                _ => target.map(str::to_string).into_iter().collect(),
            };
            return Some(if vars.is_empty() {
                w.read.to_string()
            } else {
                format!("{} {}", w.read, vars.join(", "))
            });
        }
        None
    }

    /// `printf("x = %d\n", x)` → `"x = ", x`.
    fn format(&self, fmt: Node, args: &[Node]) -> String {
        let mut parts = Vec::new();
        let mut args = args.iter().map(|a| self.expr(*a));
        if fmt.kind() != "string_literal" {
            parts.push(self.expr(fmt));
            parts.extend(args);
            return parts.join(", ");
        }
        let raw = &self.src[fmt.byte_range()];
        let body = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(raw);
        let mut literal = String::new();
        let mut chars = body.chars().peekable();
        let flush = |literal: &mut String, parts: &mut Vec<String>| {
            let text = strip_newlines(literal);
            if !text.is_empty() {
                parts.push(format!("\"{text}\""));
            }
            literal.clear();
        };
        while let Some(c) = chars.next() {
            if c != '%' {
                literal.push(c);
                continue;
            }
            if chars.peek() == Some(&'%') {
                chars.next();
                literal.push('%');
                continue;
            }
            // Skip flags, width, precision and length up to the conversion.
            for s in chars.by_ref() {
                if s == '*' {
                    args.next();
                }
                if s.is_ascii_alphabetic() && !"hljztL".contains(s) {
                    break;
                }
            }
            flush(&mut literal, &mut parts);
            if let Some(a) = args.next() {
                parts.push(a);
            }
        }
        flush(&mut literal, &mut parts);
        parts.extend(args);
        if parts.is_empty() {
            parts.push("\"\"".to_string());
        }
        parts.join(", ")
    }
}

/// How a comparison operator is written in a flowchart.
fn relation(op: &str) -> Option<&'static str> {
    Some(match op {
        "==" => "=",
        "!=" => "≠",
        "<=" => "≤",
        ">=" => "≥",
        "<" => "<",
        ">" => ">",
        _ => return None,
    })
}

/// The operator that keeps the meaning when both sides swap places.
fn flip(op: &str) -> String {
    match op {
        "<" => ">",
        ">" => "<",
        "<=" => ">=",
        ">=" => "<=",
        other => other,
    }
    .to_string()
}

/// Line breaks inside string literals mean nothing in a flowchart.
fn strip_newlines(s: &str) -> String {
    s.replace("\\r", "").replace("\\n", "")
}
