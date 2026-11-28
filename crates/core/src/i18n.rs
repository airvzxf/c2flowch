// SPDX-License-Identifier: AGPL-3.0-or-later
//! Labels printed in the flowcharts, in every supported language.

use serde::{Deserialize, Serialize};

/// Language of the labels that c2flowch writes by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English.
    #[default]
    En,
    /// Spanish.
    Es,
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::En),
            "es" | "spanish" | "español" => Ok(Language::Es),
            _ => Err(format!("unsupported language: {s} (expected en or es)")),
        }
    }
}

/// Every word that c2flowch writes by itself (not taken from the source).
#[derive(Debug, Clone, Copy)]
pub struct Words {
    /// Label of the true branch of a decision.
    pub yes: &'static str,
    /// Label of the false branch of a decision.
    pub no: &'static str,
    /// Start terminator text; subroutines add `: name(params)`.
    pub start: &'static str,
    /// End terminator text of the entry point.
    pub end: &'static str,
    /// End terminator text of a subroutine, and prefix of a `return` with a value.
    pub ret: &'static str,
    /// Verb of an output symbol (`printf` and similar).
    pub print: &'static str,
    /// Verb of an input symbol (`scanf` and similar).
    pub read: &'static str,
    /// Label of the `default` branch of a `switch`.
    pub default: &'static str,
    /// Logical AND, as written in a condition.
    pub and: &'static str,
    /// Logical OR, as written in a condition.
    pub or: &'static str,
    /// Logical NOT, as written in a condition.
    pub not: &'static str,
    /// Condition shown for a `for` loop that has none.
    pub always: &'static str,
}

impl Language {
    /// Labels of this language.
    pub fn words(self) -> &'static Words {
        match self {
            Language::En => &EN,
            Language::Es => &ES,
        }
    }
}

static EN: Words = Words {
    yes: "True",
    no: "False",
    start: "Start",
    end: "End",
    ret: "Return",
    print: "Print",
    read: "Read",
    default: "default",
    and: "and",
    or: "or",
    not: "not",
    always: "true",
};

static ES: Words = Words {
    yes: "Verdadero",
    no: "Falso",
    start: "Inicio",
    end: "Fin",
    ret: "Retornar",
    print: "Imprimir",
    read: "Leer",
    default: "por defecto",
    and: "y",
    or: "o",
    not: "no",
    always: "verdadero",
};
