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
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

pub struct I18n;

impl I18n {
    pub fn t_true(lang: Language) -> &'static str {
        match lang {
            Language::En => "True",
            Language::Es => "Verdadero",
        }
    }

    pub fn t_false(lang: Language) -> &'static str {
        match lang {
            Language::En => "False",
            Language::Es => "Falso",
        }
    }

    pub fn t_start(lang: Language) -> &'static str {
        match lang {
            Language::En => "Start",
            Language::Es => "Inicio",
        }
    }

    pub fn t_end(lang: Language) -> &'static str {
        match lang {
            Language::En => "End",
            Language::Es => "Fin",
        }
    }

    pub fn t_return(lang: Language) -> &'static str {
        match lang {
            Language::En => "Return",
            Language::Es => "Retornar",
        }
    }

    pub fn t_print(lang: Language) -> &'static str {
        match lang {
            Language::En => "Print",
            Language::Es => "Imprimir",
        }
    }

    pub fn t_read(lang: Language) -> &'static str {
        match lang {
            Language::En => "Read",
            Language::Es => "Leer",
        }
    }

    pub fn t_default(lang: Language) -> &'static str {
        match lang {
            Language::En => "default",
            Language::Es => "por defecto",
        }
    }

    pub fn t_subroutine(lang: Language) -> &'static str {
        match lang {
            Language::En => "Subroutine",
            Language::Es => "Subrutina",
        }
    }
}
