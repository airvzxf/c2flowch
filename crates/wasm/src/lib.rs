// SPDX-License-Identifier: AGPL-3.0-or-later
//! WebAssembly bindings of c2flowch, for JavaScript in a browser or Node.js.
//!
//! Build with `cargo build -p c2flowch-wasm --target wasm32-unknown-unknown
//! --release`, then generate the JavaScript glue with `wasm-bindgen`.
//!
//! ```js
//! import init, { parse, render } from "./c2flowch_wasm.js";
//! await init();
//! const parsed = parse(code, "main.c", 3, "en");
//! console.log(parsed.warnings);
//! const svg = render(parsed.yaml, "svg");
//! ```
//!
//! Every function works on the YAML text of a program, the same format that
//! `c2flowch parse` writes, so a program can be edited between the steps.

use c2flowch_core::{Language, ParseOptions, Program, Source};
use wasm_bindgen::prelude::*;

/// A program parsed from C source code.
#[wasm_bindgen]
pub struct Parsed {
    yaml: String,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl Parsed {
    /// The program as YAML, ready for [`render`].
    #[wasm_bindgen(getter)]
    pub fn yaml(&self) -> String {
        self.yaml.clone()
    }

    /// Things in the source that could not be drawn faithfully, as
    /// `file:line N: message`.
    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }
}

/// The version of c2flowch.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Parses one C source file. `level` goes from 1 (function calls only) to 5
/// (every statement); `lang` is `en` or `es`.
#[wasm_bindgen]
pub fn parse(code: &str, title: &str, level: u8, lang: &str) -> Result<Parsed, JsError> {
    parse_files(vec![title.to_string()], vec![code.to_string()], level, lang)
}

/// Parses several C source files of one program: `names[i]` is the name of
/// `codes[i]`, used in the title and in the warnings.
#[wasm_bindgen(js_name = parseFiles)]
pub fn parse_files(
    names: Vec<String>,
    codes: Vec<String>,
    level: u8,
    lang: &str,
) -> Result<Parsed, JsError> {
    do_parse(&names, &codes, level, lang).map_err(|e| JsError::new(&e))
}

/// Draws a program given as YAML. `format` is `ascii` or `svg`.
#[wasm_bindgen]
pub fn render(yaml: &str, format: &str) -> Result<String, JsError> {
    do_render(yaml, format).map_err(|e| JsError::new(&e))
}

/// Problems in a program given as YAML that the drawing cannot show, such
/// as a `break` outside of a loop. Useful after editing the YAML by hand.
#[wasm_bindgen]
pub fn validate(yaml: &str) -> Result<Vec<String>, JsError> {
    load(yaml)
        .map(|p| c2flowch_core::validate(&p))
        .map_err(|e| JsError::new(&e))
}

fn do_parse(names: &[String], codes: &[String], level: u8, lang: &str) -> Result<Parsed, String> {
    if names.len() != codes.len() {
        return Err(format!(
            "{} names for {} source files",
            names.len(),
            codes.len()
        ));
    }
    if codes.is_empty() {
        return Err("no source file to parse".to_string());
    }
    if !(1..=5).contains(&level) {
        return Err(format!("level {level} is out of range (expected 1 to 5)"));
    }
    let lang: Language = lang.parse()?;
    let sources: Vec<Source> = names
        .iter()
        .zip(codes)
        .map(|(name, code)| Source { name, code })
        .collect();
    let opts = ParseOptions {
        title: names.join(", "),
        level,
        lang,
    };
    let parsed = c2flowch_core::parse_files(&sources, &opts).map_err(|e| format!("{e:#}"))?;
    Ok(Parsed {
        yaml: parsed.program.to_yaml().map_err(|e| e.to_string())?,
        warnings: parsed
            .warnings
            .iter()
            .map(|w| format!("{}:{w}", w.file))
            .collect(),
    })
}

fn do_render(yaml: &str, format: &str) -> Result<String, String> {
    let program = load(yaml)?;
    match format.to_lowercase().as_str() {
        "ascii" => Ok(c2flowch_core::render_ascii(&program)),
        "svg" => Ok(c2flowch_core::render_svg(&program)),
        _ => Err(format!(
            "unsupported format: {format} (expected ascii or svg)"
        )),
    }
}

fn load(yaml: &str) -> Result<Program, String> {
    Program::from_yaml(yaml).map_err(|e| format!("reading the YAML: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "int main(void) {\n  int x = 1;\n  if (x > 0) {\n    printf(\"hi\");\n  }\n  return 0;\n}\n";

    fn names(n: &[&str]) -> Vec<String> {
        n.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_then_render_ascii_and_svg() {
        let p = do_parse(&names(&["main.c"]), &names(&[CODE]), 3, "en").unwrap();
        assert!(p.yaml.contains("main"));
        let ascii = do_render(&p.yaml, "ascii").unwrap();
        assert!(ascii.contains("x > 0"));
        let svg = do_render(&p.yaml, "SVG").unwrap();
        assert!(svg.starts_with("<svg") || svg.starts_with("<?xml"));
    }

    #[test]
    fn several_files_join_the_title() {
        let other = "void f(void) {\n  g();\n}\n";
        let p = do_parse(&names(&["a.c", "b.c"]), &names(&[CODE, other]), 3, "es").unwrap();
        let program = Program::from_yaml(&p.yaml).unwrap();
        assert_eq!(program.title, "a.c, b.c");
        assert_eq!(program.functions.len(), 2);
    }

    #[test]
    fn bad_arguments_are_errors() {
        let one = names(&["main.c"]);
        let code = names(&[CODE]);
        assert!(do_parse(&one, &code, 0, "en").is_err());
        assert!(do_parse(&one, &code, 3, "fr").is_err());
        assert!(do_parse(&one, &[], 3, "en").is_err());
        assert!(do_parse(&[], &[], 3, "en").is_err());
        assert!(do_render("not: [yaml", "ascii").is_err());
        let p = do_parse(&one, &code, 3, "en").unwrap();
        assert!(do_render(&p.yaml, "png").is_err());
    }
}
