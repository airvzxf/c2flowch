use std::sync::LazyLock;
use tree_sitter::Node;

static IO_FUNCTIONS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "printf(", "scanf(", "puts(", "gets(", "fputs(", "fgets(", "putchar(", "getchar(",
    ]
});

static KEYWORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "short unsigned ",
        "unsigned short ",
        "short ",
        "unsigned ",
        "int ",
        "char ",
        "double ",
        "const ",
        "float ",
        "long ",
        "void ",
        "static ",
    ]
});

pub(crate) fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut paren_count = 0;
    let mut in_quote = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c == '"' && (i == 0 || chars[i - 1] != '\\') {
            in_quote = !in_quote;
        }

        if !in_quote {
            if c == '(' {
                paren_count += 1;
            } else if c == ')' {
                paren_count -= 1;
            } else if c == ',' && paren_count == 0 {
                args.push(current.trim().to_string());
                current = String::new();
                i += 1;
                continue;
            }
        }
        current.push(c);
        i += 1;
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

pub(crate) fn process_printf(inner: &str) -> String {
    let args = split_args(inner);
    if args.is_empty() {
        return "Imprimir".to_string();
    }

    let fmt = &args[0];
    let mut parts = Vec::new();

    if fmt.starts_with('"') && fmt.ends_with('"') {
        let fmt_str = &fmt[1..fmt.len() - 1];
        let mut last_pos = 0;
        let mut arg_idx = 1;

        for (i, _) in fmt_str.match_indices('%') {
            if i > last_pos {
                parts.push(format!("\"{}\"", &fmt_str[last_pos..i]));
            }
            if arg_idx < args.len() {
                parts.push(args[arg_idx].clone());
                arg_idx += 1;
            }

            // Skip the format specifier (e.g., %d, %s, %02d)
            let mut skip = 1;
            let sub = &fmt_str[i + 1..];
            for c in sub.chars() {
                if c.is_alphabetic() {
                    break;
                }
                skip += 1;
            }
            last_pos = i + 1 + skip;
        }

        if last_pos < fmt_str.len() {
            parts.push(format!("\"{}\"", &fmt_str[last_pos..]));
        }
    } else {
        for arg in args {
            parts.push(arg);
        }
    }

    if parts.is_empty() {
        parts.push("\"\"".to_string());
    }

    format!("Imprimir {}", parts.join(", "))
}

pub(crate) fn remove_pointer_syntax(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut in_quote = false;

    while i < chars.len() {
        let c = chars[i];

        if c == '"' && (i == 0 || chars[i - 1] != '\\') {
            in_quote = !in_quote;
        }

        if !in_quote {
            if c == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                result.push('.');
                i += 2;
                continue;
            }

            if c == '&' && i + 1 < chars.len() && chars[i + 1] == '&' {
                result.push('&');
                result.push('&');
                i += 2;
                continue;
            }

            if (c == '*' || c == '&') && i + 1 < chars.len() && chars[i + 1] == '=' {
                result.push(c);
                result.push('=');
                i += 2;
                continue;
            }

            if c == '&' || c == '*' {
                let mut is_unary = false;
                if i == 0 {
                    is_unary = true;
                } else {
                    let mut j = i.saturating_sub(1);
                    while j > 0 && chars[j].is_whitespace() {
                        j -= 1;
                    }
                    let prev = chars[j];
                    if prev == '('
                        || prev == '['
                        || prev == ','
                        || prev == '='
                        || prev == '+'
                        || prev == '-'
                        || prev == '*'
                        || prev == '/'
                        || prev == '<'
                        || prev == '>'
                        || prev == '!'
                        || prev == '&'
                        || prev == '|'
                        || prev == ':'
                        || prev == '?'
                        || prev == '{'
                        || prev == ';'
                    {
                        is_unary = true;
                    } else if prev.is_alphabetic() {
                        let mut k = j;
                        while k > 0 && chars[k - 1].is_alphanumeric() {
                            k -= 1;
                        }
                        let word: String = chars[k..=j].iter().collect();
                        if matches!(
                            word.as_str(),
                            "return"
                                | "case"
                                | "if"
                                | "while"
                                | "int"
                                | "char"
                                | "float"
                                | "double"
                                | "void"
                                | "struct"
                                | "long"
                                | "short"
                                | "unsigned"
                                | "signed"
                        ) {
                            is_unary = true;
                        }
                    }
                }

                if is_unary {
                    i += 1;
                    continue;
                }
            }
        }

        result.push(c);
        i += 1;
    }
    result
}

pub(crate) fn clean_text(text: &str) -> String {
    let mut cleaned = remove_pointer_syntax(text.trim()).replace("\\n", "");

    for kw in KEYWORDS.iter() {
        if cleaned.starts_with(kw) {
            cleaned = cleaned[kw.len()..].to_string();
        }
        if cleaned.contains(kw) {
            cleaned = cleaned.replace(&format!(" {}", kw), " ");
        }
    }

    // Strip "struct XYZ " if it is at the beginning
    if cleaned.starts_with("struct ")
        && let Some(space_idx) = cleaned[7..].find(' ')
    {
        let next_word_end = 7 + space_idx;
        cleaned = cleaned[next_word_end..].trim().to_string();
    }

    cleaned = cleaned.replace(';', "").trim().to_string();

    for &call in &["printf(", "puts(", "fputs(", "putchar("] {
        if let Some(start) = cleaned.find(call) {
            if cleaned.len() > start + call.len() {
                let inner = &cleaned[start + call.len()..cleaned.len() - 1];
                if call == "printf(" {
                    return process_printf(inner);
                }
                let args = split_args(inner);
                if !args.is_empty() {
                    return format!("Imprimir {}", args[0]);
                }
            }
            return "Imprimir".to_string();
        }
    }

    for &call in &["scanf(", "gets(", "fgets(", "getchar("] {
        if let Some(start) = cleaned.find(call) {
            if cleaned.len() > start + call.len() {
                let inner = &cleaned[start + call.len()..cleaned.len() - 1];
                if call == "scanf(" {
                    if let Some(comma_pos) = inner.find(',') {
                        let var = inner[comma_pos + 1..].trim().replace('&', "");
                        return format!("Leer {}", var);
                    }
                    return "Leer".to_string();
                }
                let args = split_args(inner);
                if !args.is_empty() {
                    return format!("Leer {}", args[0].replace('&', ""));
                }
            }
            return "Leer".to_string();
        }
    }

    for &op in &["+=", "-=", "*=", "/=", "%="] {
        if let Some(pos) = cleaned.find(op) {
            let var = cleaned[..pos].trim();
            let expr = cleaned[pos + op.len()..].trim().replace(';', "");
            let symbol = match op {
                "+=" => "+",
                "-=" => "-",
                "*=" => "*",
                "/=" => "/",
                "%=" => "%",
                _ => unreachable!(),
            };

            let is_complex = expr.contains(' ')
                || expr.contains('+')
                || expr.contains('-')
                || expr.contains('*')
                || expr.contains('/')
                || expr.contains('%');

            if is_complex {
                return format!("{} ← {} {} ({})", var, var, symbol, expr);
            } else {
                return format!("{} ← {} {} {}", var, var, symbol, expr);
            }
        }
    }

    if cleaned.contains('=') && !cleaned.contains("==") {
        return cleaned.replace(" = ", " ← ").replace('=', " ← ");
    }

    if (cleaned.starts_with("strcpy(") || cleaned.starts_with("strncpy("))
        && let Some(inner_end) = cleaned.rfind(')')
    {
        let func_len = if cleaned.starts_with("strcpy(") { 7 } else { 8 };
        let inner = &cleaned[func_len..inner_end];
        let args = split_args(inner);
        if args.len() >= 2 {
            return format!("{} ← {}", args[0], args[1]);
        }
    }

    if cleaned.contains("++") || cleaned.contains("--") {
        let is_inc = cleaned.contains("++");
        let op = if is_inc { "++" } else { "--" };
        let mut var = cleaned.replace(op, "").replace(';', "").trim().to_string();

        if var.starts_with('(') && var.ends_with(')') {
            var = var[1..var.len() - 1].trim().to_string();
        }

        if !var.contains('(') {
            let symbol = if is_inc { "+" } else { "-" };
            return format!("{} ← {} {} 1", var, var, symbol);
        }
    }

    cleaned
}

pub(crate) fn is_io(node: Node, code: &str) -> bool {
    if node.kind() == "expression_statement" {
        let text = &code[node.start_byte()..node.end_byte()];
        IO_FUNCTIONS.iter().any(|&f| text.contains(f))
    } else {
        false
    }
}

pub(crate) fn is_preparation(node: Node, code: &str) -> bool {
    if node.kind() == "declaration" {
        let text = &code[node.start_byte()..node.end_byte()];
        !text.contains('=') && !text.contains("++") && !text.contains("--")
    } else {
        false
    }
}

pub(crate) fn is_process(node: Node, code: &str) -> bool {
    match node.kind() {
        "expression_statement" => {
            let text = &code[node.start_byte()..node.end_byte()];
            !IO_FUNCTIONS.iter().any(|&f| text.contains(f))
        }
        "declaration" => {
            let text = &code[node.start_byte()..node.end_byte()];
            text.contains('=') || text.contains("++") || text.contains("--")
        }
        "update_expression" => true,
        _ => false,
    }
}

pub(crate) fn simplify_strcmp(mut text: String) -> String {
    while let Some(start) = text.find("strcmp(") {
        let mut paren_count = 0;
        let mut end_idx = start + 7;
        let chars = text.as_bytes();
        let mut found_end = false;
        while end_idx < chars.len() {
            if chars[end_idx] == b'(' {
                paren_count += 1;
            } else if chars[end_idx] == b')' {
                if paren_count == 0 {
                    found_end = true;
                    break;
                }
                paren_count -= 1;
            }
            end_idx += 1;
        }

        if found_end {
            let inner = &text[start + 7..end_idx];
            let args = split_args(inner);
            if args.len() == 2 {
                let rest = text[end_idx + 1..].to_string();
                let mut op = "";
                let mut val = "";

                let operators = [
                    (" == 0", " = "),
                    (" != 0", " ≠ "),
                    (" > 0", " > "),
                    (" < 0", " < "),
                    (" >= 0", " ≥ "),
                    (" <= 0", " ≤ "),
                ];

                for (o, replacement) in operators {
                    if let Some(stripped) = rest.strip_prefix(o) {
                        op = replacement;
                        val = stripped;
                        break;
                    }
                }

                if !op.is_empty() {
                    text = format!("{}{} {}{}", args[0], op, args[1], val);
                    continue;
                } else {
                    text = format!("{} = {}{}", args[0], args[1], rest);
                    continue;
                }
            }
        }
        break;
    }
    text
}

pub(crate) fn clean_condition(text: &str) -> String {
    let mut cleaned = remove_pointer_syntax(text.trim());
    if cleaned.starts_with('(') && cleaned.ends_with(')') {
        cleaned = cleaned[1..cleaned.len() - 1].to_string();
    }
    cleaned = simplify_strcmp(cleaned);
    cleaned.replace("==", "=").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_args_basic() {
        let args = split_args("\"fmt\", var1, var2");
        assert_eq!(args, vec!["\"fmt\"", "var1", "var2"]);
    }

    #[test]
    fn test_process_printf_basic() {
        let res = process_printf("\"Value: %d\", x");
        assert_eq!(res, "Imprimir \"Value: \", x");
    }

    #[test]
    fn test_clean_text_io() {
        assert_eq!(clean_text("puts(\"Hello\\n\");"), "Imprimir \"Hello\"");
        assert_eq!(clean_text("puts(\"Hello\");"), "Imprimir \"Hello\"");
        assert_eq!(
            clean_text("fputs(\"World\", stdout);"),
            "Imprimir \"World\""
        );
        assert_eq!(clean_text("putchar('A');"), "Imprimir 'A'");
        assert_eq!(clean_text("gets(buf);"), "Leer buf");
        assert_eq!(clean_text("fgets(buf, 10, stdin);"), "Leer buf");
    }

    #[test]
    fn test_unify_member_access() {
        assert_eq!(clean_text("linea->curp"), "linea.curp");
        assert_eq!(clean_text("linea.curp"), "linea.curp");
        assert_eq!(clean_condition("linea->id > 0"), "linea.id > 0");
    }
}
