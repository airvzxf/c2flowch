//! Labels printed in the flowcharts, in every supported language.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
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
    pub yes: &'static str,
    pub no: &'static str,
    pub start: &'static str,
    pub end: &'static str,
    pub ret: &'static str,
    pub print: &'static str,
    pub read: &'static str,
    pub default: &'static str,
    pub and: &'static str,
    pub or: &'static str,
    pub not: &'static str,
    pub always: &'static str,
}

impl Language {
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
