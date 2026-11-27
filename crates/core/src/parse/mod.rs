// SPDX-License-Identifier: AGPL-3.0-or-later
//! C source -> structured IR, using the tree-sitter C grammar.

mod text;

use crate::detail;
use crate::i18n::Language;
use crate::ir::{Case, Function, Program, Stmt};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use text::{Text, callee, named_children};
use tree_sitter::{Node, Parser};

/// Options of [`parse`].
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Title of the program, usually the file name.
    pub title: String,
    /// Level of detail, from 1 (function calls only) to 5 (every statement).
    pub level: u8,
    pub lang: Language,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            title: String::new(),
            level: 3,
            lang: Language::En,
        }
    }
}

/// Something in the source that c2flowch could not draw faithfully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// 1-based source line.
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Result of [`parse`].
#[derive(Debug, Clone)]
pub struct Parsed {
    pub program: Program,
    pub warnings: Vec<Warning>,
}

struct FunctionDef<'t> {
    name: String,
    params: Vec<String>,
    body: Node<'t>,
}

/// Parses C source code into a [`Program`].
///
/// With a `main` function, the program starts with `main`, followed by the
/// functions it calls (breadth first, in call order), followed by any
/// other function of the file. Without `main`, functions keep their order
/// in the file.
pub fn parse(code: &str, opts: &ParseOptions) -> Result<Parsed> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_c::LANGUAGE.into())
        .context("loading the C grammar")?;
    let tree = parser.parse(code, None).context("parsing the C source")?;
    let root = tree.root_node();

    let mut warnings = Vec::new();
    syntax_errors(root, &mut warnings);

    let mut defs = Vec::new();
    let mut global_decls = Vec::new();
    collect_functions(root, code, &mut defs, &mut global_decls);
    let known: HashSet<String> = defs.iter().map(|d| d.name.clone()).collect();

    let order = function_order(&defs, code, &known);
    let mut conv = Converter {
        text: Text {
            src: code,
            words: opts.lang.words(),
        },
        known: &known,
        warnings,
    };

    let mut globals = Vec::new();
    for d in global_decls {
        conv.declaration(d, &mut globals);
    }
    let globals = detail::apply(globals, opts.level);

    let mut functions = Vec::new();
    for i in order {
        let def = &defs[i];
        let body = conv.block(def.body);
        functions.push(Function {
            name: def.name.clone(),
            params: def.params.clone(),
            entry: def.name == "main",
            body: detail::apply(body, opts.level),
        });
    }

    let mut warnings = conv.warnings;
    warnings.sort_by_key(|w| w.line);
    warnings.dedup();
    Ok(Parsed {
        program: Program {
            title: opts.title.clone(),
            lang: opts.lang,
            globals,
            functions,
        },
        warnings,
    })
}

fn syntax_errors(root: Node, out: &mut Vec<Warning>) {
    if !root.has_error() {
        return;
    }
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.is_error() || n.is_missing() {
            out.push(Warning {
                line: n.start_position().row + 1,
                message: if n.is_missing() {
                    format!("syntax error: missing `{}`", n.kind())
                } else {
                    "syntax error: this part of the code is skipped".to_string()
                },
            });
            continue;
        }
        if n.has_error() {
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
    }
}

/// Function definitions and variable declarations at file level, also
/// inside `#if` blocks.
fn collect_functions<'t>(
    n: Node<'t>,
    src: &str,
    out: &mut Vec<FunctionDef<'t>>,
    globals: &mut Vec<Node<'t>>,
) {
    let mut c = n.walk();
    for child in n.named_children(&mut c) {
        match child.kind() {
            "function_definition" => {
                let Some(body) = child.child_by_field_name("body") else {
                    continue;
                };
                let Some(fd) = child
                    .child_by_field_name("declarator")
                    .and_then(function_declarator)
                else {
                    continue;
                };
                let Some(name) = fd.child_by_field_name("declarator").and_then(declared_name)
                else {
                    continue;
                };
                let params = fd
                    .child_by_field_name("parameters")
                    .map(|p| {
                        named_children(p)
                            .into_iter()
                            .filter_map(|p| p.child_by_field_name("declarator"))
                            .filter_map(declared_name)
                            .map(|n| src[n.byte_range()].to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                out.push(FunctionDef {
                    name: src[name.byte_range()].to_string(),
                    params,
                    body,
                });
            }
            "declaration" => globals.push(child),
            k if k.starts_with("preproc_") => collect_functions(child, src, out, globals),
            _ => {}
        }
    }
}

fn function_declarator(mut n: Node) -> Option<Node> {
    loop {
        if n.kind() == "function_declarator" {
            return Some(n);
        }
        n = n.child_by_field_name("declarator")?;
    }
}

/// The identifier inside a declarator such as `*p`, `a[10]` or `(*f)(int)`.
fn declared_name(mut n: Node) -> Option<Node> {
    loop {
        match n.kind() {
            "identifier" | "field_identifier" => return Some(n),
            "parenthesized_declarator" => n = named_children(n).into_iter().next()?,
            _ => n = n.child_by_field_name("declarator")?,
        }
    }
}

/// Indices into `defs`: main, what it calls (breadth first), the rest.
fn function_order(defs: &[FunctionDef], src: &str, known: &HashSet<String>) -> Vec<usize> {
    let index: HashMap<&str, usize> = defs
        .iter()
        .enumerate()
        .map(|(i, d)| (d.name.as_str(), i))
        .collect();
    let mut order = Vec::new();
    let mut seen = HashSet::new();
    if let Some(&main) = index.get("main") {
        let mut queue = VecDeque::from([main]);
        seen.insert(main);
        while let Some(i) = queue.pop_front() {
            order.push(i);
            let mut calls = Vec::new();
            collect_calls(defs[i].body, src, known, &mut calls);
            for name in calls {
                let j = index[name.as_str()];
                if seen.insert(j) {
                    queue.push_back(j);
                }
            }
        }
    }
    for i in 0..defs.len() {
        if seen.insert(i) {
            order.push(i);
        }
    }
    order
}

/// Calls to functions of the file, in source order.
fn collect_calls(n: Node, src: &str, known: &HashSet<String>, out: &mut Vec<String>) {
    if let Some(name) = callee(n, src)
        && known.contains(name)
    {
        out.push(name.to_string());
    }
    let mut c = n.walk();
    for child in n.children(&mut c) {
        collect_calls(child, src, known, out);
    }
}

struct Converter<'a> {
    text: Text<'a>,
    known: &'a HashSet<String>,
    warnings: Vec<Warning>,
}

impl Converter<'_> {
    fn warn(&mut self, n: Node, message: impl Into<String>) {
        self.warnings.push(Warning {
            line: n.start_position().row + 1,
            message: message.into(),
        });
    }

    fn calls_known(&self, n: Node) -> bool {
        let mut calls = Vec::new();
        collect_calls(n, self.text.src, self.known, &mut calls);
        !calls.is_empty()
    }

    fn block(&mut self, n: Node) -> Vec<Stmt> {
        let mut out = Vec::new();
        self.stmt(n, &mut out);
        out
    }

    fn stmt(&mut self, n: Node, out: &mut Vec<Stmt>) {
        match n.kind() {
            "compound_statement" => {
                for child in named_children(n) {
                    self.stmt(child, out);
                }
            }
            "expression_statement" => {
                if let Some(e) = named_children(n).first() {
                    self.expression(*e, out);
                }
            }
            "declaration" => self.declaration(n, out),
            "if_statement" => {
                let cond = self.condition(n);
                let then = n
                    .child_by_field_name("consequence")
                    .map(|c| self.block(c))
                    .unwrap_or_default();
                let otherwise = n
                    .child_by_field_name("alternative")
                    .map(|alt| {
                        let mut v = Vec::new();
                        for s in named_children(alt) {
                            self.stmt(s, &mut v);
                        }
                        v
                    })
                    .unwrap_or_default();
                out.push(Stmt::If {
                    cond,
                    then,
                    otherwise,
                });
            }
            "while_statement" => {
                let cond = self.condition(n);
                let body = self.body(n);
                out.push(Stmt::While { cond, body });
            }
            "do_statement" => {
                let cond = self.condition(n);
                let body = self.body(n);
                out.push(Stmt::DoWhile { body, cond });
            }
            "for_statement" => self.for_statement(n, out),
            "switch_statement" => self.switch(n, out),
            "break_statement" => out.push(Stmt::Break),
            "continue_statement" => out.push(Stmt::Continue),
            "return_statement" => {
                let value = named_children(n).first().map(|v| self.text.expr(*v));
                out.push(Stmt::Return(value));
            }
            "goto_statement" => {
                if let Some(l) = n.child_by_field_name("label") {
                    out.push(Stmt::Goto(self.text.raw(l)));
                }
            }
            "labeled_statement" => {
                let label = n.child_by_field_name("label");
                if let Some(l) = label {
                    out.push(Stmt::Label(self.text.raw(l)));
                }
                for child in named_children(n) {
                    if Some(child) != label {
                        self.stmt(child, out);
                    }
                }
            }
            "attributed_statement" => {
                for child in named_children(n) {
                    if child.kind() != "attribute_declaration" {
                        self.stmt(child, out);
                    }
                }
            }
            "preproc_if" | "preproc_ifdef" => {
                let skip: Vec<Node> = ["name", "condition", "alternative"]
                    .iter()
                    .filter_map(|f| n.child_by_field_name(f))
                    .collect();
                if n.child_by_field_name("alternative").is_some() {
                    self.warn(
                        n,
                        "#if/#else inside a function: only the first branch is drawn",
                    );
                }
                for child in named_children(n) {
                    if !skip.contains(&child) {
                        self.stmt(child, out);
                    }
                }
            }
            // Type definitions and preprocessor lines do not change the flow.
            "type_definition" | "struct_specifier" | "union_specifier" | "enum_specifier" => {}
            k if k.starts_with("preproc_") => {}
            "ERROR" => {}
            k => {
                self.warn(
                    n,
                    format!("unsupported statement `{k}`, drawn as plain text"),
                );
                out.push(Stmt::Process(self.text.raw(n)));
            }
        }
    }

    fn condition(&self, n: Node) -> String {
        n.child_by_field_name("condition")
            .map(|c| self.text.condition(c))
            .unwrap_or_default()
    }

    fn body(&mut self, n: Node) -> Vec<Stmt> {
        n.child_by_field_name("body")
            .map(|b| self.block(b))
            .unwrap_or_default()
    }

    /// An expression used as a statement. `a = 1, b = 2;` gives two.
    fn expression(&mut self, e: Node, out: &mut Vec<Stmt>) {
        if e.kind() == "comma_expression" {
            for part in ["left", "right"] {
                if let Some(p) = e.child_by_field_name(part) {
                    self.expression(p, out);
                }
            }
            return;
        }
        if let Some(io) = self.text.io(e, None) {
            out.push(Stmt::Io(io));
            return;
        }
        if e.kind() == "assignment_expression"
            && let (Some(l), Some(r)) = (
                e.child_by_field_name("left"),
                e.child_by_field_name("right"),
            )
            && e.child_by_field_name("operator")
                .is_some_and(|o| self.text.raw(o) == "=")
            && let Some(io) = self.text.io(r, Some(&self.text.expr(l)))
        {
            out.push(Stmt::Io(io));
            return;
        }
        let label = self.text.statement(e);
        out.push(if self.calls_known(e) {
            Stmt::Call(label)
        } else {
            Stmt::Process(label)
        });
    }

    fn declaration(&mut self, n: Node, out: &mut Vec<Stmt>) {
        let ty = n.child_by_field_name("type").map(|t| self.text.raw(t));
        let mut c = n.walk();
        let declarators: Vec<Node> = n.children_by_field_name("declarator", &mut c).collect();
        let mut plain = Vec::new();
        let flush = |plain: &mut Vec<String>, out: &mut Vec<Stmt>| {
            if !plain.is_empty() {
                let names = plain.join(", ");
                out.push(Stmt::Prep(match &ty {
                    Some(t) => format!("{t} {names}"),
                    None => names,
                }));
                plain.clear();
            }
        };
        for d in declarators {
            match d.kind() {
                "init_declarator" => {
                    flush(&mut plain, out);
                    let (Some(target), Some(value)) = (
                        d.child_by_field_name("declarator"),
                        d.child_by_field_name("value"),
                    ) else {
                        continue;
                    };
                    let name = declared_name(target)
                        .map_or_else(|| self.text.raw(target), |t| self.text.raw(t));
                    if let Some(io) = self.text.io(value, Some(&name)) {
                        out.push(Stmt::Io(io));
                        continue;
                    }
                    let label = format!("{name} <- {}", self.text.expr(value));
                    out.push(if self.calls_known(value) {
                        Stmt::Call(label)
                    } else {
                        Stmt::Process(label)
                    });
                }
                // A prototype declares nothing that runs.
                "function_declarator" => {}
                _ => plain.push(self.text.raw(d)),
            }
        }
        flush(&mut plain, out);
    }

    fn for_statement(&mut self, n: Node, out: &mut Vec<Stmt>) {
        let lines = |conv: &mut Self, node: Option<Node>| -> Option<String> {
            let node = node?;
            let mut stmts = Vec::new();
            if node.kind() == "declaration" {
                conv.declaration(node, &mut stmts);
            } else {
                conv.expression(node, &mut stmts);
            }
            let texts: Vec<String> = stmts
                .into_iter()
                .filter_map(|s| match s {
                    Stmt::Process(t) | Stmt::Io(t) | Stmt::Call(t) | Stmt::Prep(t) => Some(t),
                    _ => None,
                })
                .collect();
            (!texts.is_empty()).then(|| texts.join("\n"))
        };
        let init = lines(self, n.child_by_field_name("initializer"));
        let update = lines(self, n.child_by_field_name("update"));
        let cond = n
            .child_by_field_name("condition")
            .map(|c| self.text.condition(c));
        let body = self.body(n);
        out.push(Stmt::For {
            init,
            cond,
            update,
            body,
        });
    }

    fn switch(&mut self, n: Node, out: &mut Vec<Stmt>) {
        let expr = self.condition(n);
        let mut cases: Vec<Case> = Vec::new();
        if let Some(body) = n.child_by_field_name("body") {
            for child in named_children(body) {
                if child.kind() != "case_statement" {
                    self.warn(child, "statement before the first `case` is never run");
                    continue;
                }
                let value = child.child_by_field_name("value");
                // Labels without statements share the body of the next label.
                let case = match cases.last_mut() {
                    Some(last) if last.body.is_empty() => last,
                    _ => {
                        cases.push(Case {
                            values: Vec::new(),
                            default: false,
                            body: Vec::new(),
                        });
                        cases.last_mut().expect("just pushed")
                    }
                };
                match value {
                    Some(v) => case.values.push(self.text.expr(v)),
                    None => case.default = true,
                }
                let mut stmts = Vec::new();
                for s in named_children(child) {
                    if Some(s) != value {
                        self.stmt(s, &mut stmts);
                    }
                }
                cases.last_mut().expect("not empty").body = stmts;
            }
        }
        out.push(Stmt::Switch { expr, cases });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(code: &str) -> Vec<Stmt> {
        let opts = ParseOptions {
            level: 5,
            ..Default::default()
        };
        let p = parse(code, &opts).unwrap();
        p.program.functions[0].body.clone()
    }

    fn one(stmt: &str) -> Stmt {
        let mut v = labels(&format!("int main() {{ {stmt} }}"));
        assert_eq!(v.len(), 1, "{v:?}");
        v.remove(0)
    }

    #[test]
    fn assignments() {
        assert_eq!(one("x = y * z;"), Stmt::Process("x <- y * z".into()));
        assert_eq!(one("x += 2;"), Stmt::Process("x <- x + 2".into()));
        assert_eq!(one("x *= a + b;"), Stmt::Process("x <- x * (a + b)".into()));
        assert_eq!(one("i++;"), Stmt::Process("i <- i + 1".into()));
        assert_eq!(one("--i;"), Stmt::Process("i <- i - 1".into()));
        assert_eq!(one("a = b == 3;"), Stmt::Process("a <- b = 3".into()));
        assert_eq!(one("arr[i++] = 0;"), Stmt::Process("arr[i++] <- 0".into()));
        assert_eq!(one("p->next = *q;"), Stmt::Process("p.next <- q".into()));
        assert_eq!(one("x = (float) y;"), Stmt::Process("x <- y".into()));
    }

    #[test]
    fn declarations() {
        assert_eq!(
            labels("int main() { int a = 1, b, *p = &a; }"),
            vec![
                Stmt::Process("a <- 1".into()),
                Stmt::Prep("int b".into()),
                Stmt::Process("p <- a".into()),
            ]
        );
        assert_eq!(
            one(r#"char *s = "int x = 5; printf(";"#),
            Stmt::Process(r#"s <- "int x = 5; printf(""#.into())
        );
    }

    #[test]
    fn globals_are_file_scope_declarations() {
        let opts = ParseOptions {
            level: 5,
            ..Default::default()
        };
        let code = "int n = 1, m;\nint f(int);\n#ifdef X\nchar c;\n#endif\nint main() { int k; }";
        let p = parse(code, &opts).unwrap().program;
        assert_eq!(
            p.globals,
            vec![
                Stmt::Process("n <- 1".into()),
                Stmt::Prep("int m".into()),
                Stmt::Prep("char c".into()),
            ]
        );
        assert_eq!(p.functions[0].body, vec![Stmt::Prep("int k".into())]);
        let yaml = p.to_yaml().unwrap();
        assert_eq!(Program::from_yaml(&yaml).unwrap(), p);
    }

    #[test]
    fn input_output() {
        assert_eq!(
            one(r#"printf("Value: %d, %5.2f%%\n", x, y);"#),
            Stmt::Io(r#"Print "Value: ", x, ", ", y, "%""#.into())
        );
        assert_eq!(one(r#"puts("Hi\n");"#), Stmt::Io(r#"Print "Hi""#.into()));
        assert_eq!(
            one(r#"scanf("%d %d", &a, &b);"#),
            Stmt::Io("Read a, b".into())
        );
        assert_eq!(one("c = getchar();"), Stmt::Io("Read c".into()));
        assert_eq!(one("fgets(buf, 10, stdin);"), Stmt::Io("Read buf".into()));
        assert_eq!(one(r#"printf("\n");"#), Stmt::Io(r#"Print """#.into()));
    }

    #[test]
    fn conditions() {
        let cond = |c: &str| match one(&format!("if ({c}) {{}}")) {
            Stmt::If { cond, .. } => cond,
            s => panic!("{s:?}"),
        };
        assert_eq!(cond("a == b && c != d"), "a = b and c != d");
        assert_eq!(cond("strcmp(s, \"x\") == 0"), "s = \"x\"");
        assert_eq!(cond("!strcmp(s, t)"), "s = t");
        assert_eq!(cond("strcmp(s, t)"), "s != t");
        assert_eq!(cond("0 < strcmp(s, t)"), "s > t");
        assert_eq!(cond("!(x <= 3)"), "not (x <= 3)");
    }

    #[test]
    fn calls_match_whole_names() {
        let p = parse(
            "int sum(int a, int b) { return a + b; }\n\
             int checksum(int x) { return x; }\n\
             int main() { int r = checksum(5); return 0; }",
            &ParseOptions::default(),
        )
        .unwrap();
        let names: Vec<&str> = p
            .program
            .functions
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["main", "checksum", "sum"]);
        assert_eq!(
            p.program.functions[0].body[0],
            Stmt::Call("r <- checksum(5)".into())
        );
    }

    #[test]
    fn braceless_bodies_keep_their_statements() {
        assert_eq!(
            one("if (x > 2) x = 0; else y++;"),
            Stmt::If {
                cond: "x > 2".into(),
                then: vec![Stmt::Process("x <- 0".into())],
                otherwise: vec![Stmt::Process("y <- y + 1".into())],
            }
        );
        assert_eq!(
            one("while (x < 5) x++;"),
            Stmt::While {
                cond: "x < 5".into(),
                body: vec![Stmt::Process("x <- x + 1".into())],
            }
        );
    }

    #[test]
    fn switch_groups_labels() {
        let Stmt::Switch { cases, .. } =
            one("switch (x) { case 1: case 2: y = 2; break; default: y = 0; }")
        else {
            panic!()
        };
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].values, ["1", "2"]);
        assert_eq!(cases[0].body.last(), Some(&Stmt::Break));
        assert!(cases[1].default);
    }

    #[test]
    fn labels_and_gotos() {
        assert_eq!(
            labels("int main() { again: x = x > 5 ? 0 : x + 1; if (x < 3) goto again; }"),
            vec![
                Stmt::Label("again".into()),
                Stmt::Process("x <- x > 5 ? 0 : x + 1".into()),
                Stmt::If {
                    cond: "x < 3".into(),
                    then: vec![Stmt::Goto("again".into())],
                    otherwise: vec![],
                },
            ]
        );
    }

    #[test]
    fn syntax_errors_are_reported() {
        let p = parse("int main() { x = ; }", &ParseOptions::default()).unwrap();
        assert!(!p.warnings.is_empty());
    }
}
