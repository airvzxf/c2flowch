// SPDX-License-Identifier: AGPL-3.0-or-later
//! Structured intermediate representation.
//!
//! The IR is a tree of statements that mirrors the control flow of the C
//! source. It is what `c2flowch parse` writes as YAML and what
//! `c2flowch render` reads back, so it is designed to be edited by hand:
//! there are no node ids, no join nodes and no layout hints. Every text is
//! the final label that will be printed inside a flowchart symbol.
//!
//! ```yaml
//! title: example.c
//! lang: en
//! functions:
//! - name: main
//!   entry: true
//!   body:
//!   - process: i <- 0
//!   - while:
//!       cond: i < 5
//!       body:
//!       - io: Print i
//!       - process: i <- i + 1
//!   - return: null
//! ```

use crate::i18n::Language;
use serde::{Deserialize, Serialize};

/// A whole C file: one flowchart per function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    pub title: String,
    #[serde(default)]
    pub lang: Language,
    /// File-scope variables, drawn at the start of the entry point (`main`),
    /// right after its start terminator. Only declarations are expected
    /// here: `prep`, `process`, `io` or `call` statements.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "serde_norway::with::singleton_map_recursive"
    )]
    pub globals: Vec<Stmt>,
    pub functions: Vec<Function>,
}

/// One function, drawn as its own flowchart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    /// Parameter names, shown in the start terminator of subroutines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<String>,
    /// The program entry point (`main`): its terminators read "Start"/"End"
    /// instead of "Start: name(params)"/"Return".
    #[serde(default, skip_serializing_if = "is_false")]
    pub entry: bool,
    #[serde(with = "serde_norway::with::singleton_map_recursive")]
    pub body: Vec<Stmt>,
}

/// One statement. Simple statements hold the text of their symbol; a text
/// may span several lines, and each line is printed on its own row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stmt {
    /// Process symbol (rectangle): assignments and other operations.
    Process(String),
    /// Data symbol (parallelogram): input and output.
    Io(String),
    /// Predefined process symbol: a call to a function of the same file.
    Call(String),
    /// Preparation symbol (hexagon): declarations without a value.
    Prep(String),
    If {
        cond: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        then: Vec<Stmt>,
        #[serde(default, rename = "else", skip_serializing_if = "Vec::is_empty")]
        otherwise: Vec<Stmt>,
    },
    While {
        cond: String,
        #[serde(default)]
        body: Vec<Stmt>,
    },
    DoWhile {
        #[serde(default)]
        body: Vec<Stmt>,
        cond: String,
    },
    For {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        init: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cond: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        update: Option<String>,
        #[serde(default)]
        body: Vec<Stmt>,
    },
    Switch {
        expr: String,
        cases: Vec<Case>,
    },
    Break,
    Continue,
    /// `return`, with the returned expression if there is one.
    Return(Option<String>),
    /// `goto label`.
    Goto(String),
    /// A label that `goto` statements can jump to.
    Label(String),
}

/// One branch of a `switch`. Consecutive `case` labels that share a body
/// are merged into one branch. Without a trailing `break`, control falls
/// through into the next branch, as in C.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Case {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub default: bool,
    #[serde(default)]
    pub body: Vec<Stmt>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Program {
    pub fn to_yaml(&self) -> Result<String, serde_norway::Error> {
        serde_norway::to_string(self)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_norway::Error> {
        serde_norway::from_str(yaml)
    }
}
