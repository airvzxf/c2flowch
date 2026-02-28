use tree_sitter::Node;

pub(crate) fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut paren_count = 0;
    let mut in_quote = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quote {
            if c == '\\' {
                current.push(c);
                if let Some(next_c) = chars.next() {
                    current.push(next_c);
                }
            } else if c == '"' {
                in_quote = false;
                current.push(c);
            } else {
                current.push(c);
            }
        } else {
            match c {
                '"' => {
                    in_quote = true;
                    current.push(c);
                }
                '(' => {
                    paren_count += 1;
                    current.push(c);
                }
                ')' => {
                    paren_count -= 1;
                    current.push(c);
                }
                ',' => {
                    if paren_count == 0 {
                        args.push(current.trim().to_string());
                        current.clear();
                    } else {
                        current.push(c);
                    }
                }
                _ => current.push(c),
            }
        }
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

pub(crate) fn process_printf(inner: &str) -> String {
    let args = split_args(inner);
    if args.is_empty() {
        return format!("Imprimir {}", inner);
    }

    let fmt_str = &args[0];
    if !fmt_str.starts_with('"') || !fmt_str.ends_with('"') {
        return format!("Imprimir {}", inner);
    }

    let fmt_content = &fmt_str[1..fmt_str.len() - 1];
    let vars = &args[1..];

    let mut parts = Vec::new();
    let mut last_pos = 0;
    let mut var_idx = 0;
    let mut chars = fmt_content.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '%' {
            if let Some(&(_, next_c)) = chars.peek()
                && next_c == '%'
            {
                chars.next();
                continue;
            }

            // Found specifier
            if i > last_pos {
                let s = &fmt_content[last_pos..i];
                let s_clean = s.replace("\\n", "").replace("%%", "%");
                if !s_clean.is_empty() {
                    parts.push(format!("\"{}\"", s_clean));
                }
            }

            let mut spec_end = i + 1;
            while let Some((j, sc)) = chars.peek() {
                spec_end = *j + 1;
                if "diuoxXfFeEgGaAcspn".contains(*sc) {
                    chars.next();
                    break;
                }
                if !"-+ #0123456789.*hljztL".contains(*sc) {
                    break;
                }
                chars.next();
            }

            if var_idx < vars.len() {
                parts.push(vars[var_idx].clone());
                var_idx += 1;
            }

            last_pos = spec_end;
        }
    }

    if last_pos < fmt_content.len() {
        let s = &fmt_content[last_pos..];
        let s_clean = s.replace("\\n", "").replace("%%", "%");
        if !s_clean.is_empty() {
            parts.push(format!("\"{}\"", s_clean));
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

const IO_FUNCTIONS: &[&str] = &[
    "printf(", "scanf(", "puts(", "gets(", "fputs(", "fgets(", "putchar(", "getchar(",
];

pub(crate) fn clean_text(text: &str) -> String {
    let mut cleaned = remove_pointer_syntax(text.trim());

    let keywords = [
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
    ];

    for kw in keywords {
        if cleaned.starts_with(kw) {
            cleaned = cleaned[kw.len()..].to_string();
        }
        cleaned = cleaned.replace(&format!(" {}", kw), " ");
    }

    // Strip "struct XYZ " if it is at the beginning
    if cleaned.starts_with("struct ")
        && let Some(space_idx) = cleaned[7..].find(' ')
    {
        let next_word_end = 7 + space_idx;
        cleaned = cleaned[next_word_end..].trim().to_string();
    }

    cleaned = cleaned.replace(";", "").trim().to_string();

    let io_output = ["printf(", "puts(", "fputs(", "putchar("];
    let io_input = ["scanf(", "gets(", "fgets(", "getchar("];

    for &call in &io_output {
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

    for &call in &io_input {
        if let Some(start) = cleaned.find(call) {
            if cleaned.len() > start + call.len() {
                let inner = &cleaned[start + call.len()..cleaned.len() - 1];
                if call == "scanf(" {
                    if let Some(comma_pos) = inner.find(',') {
                        let var = inner[comma_pos + 1..].trim().replace("&", "");
                        return format!("Leer {}", var);
                    }
                    return "Leer".to_string();
                }
                let args = split_args(inner);
                if !args.is_empty() {
                    return format!("Leer {}", args[0].replace("&", ""));
                }
            }
            return "Leer".to_string();
        }
    }

    for &op in &["+=", "-=", "*=", "/=", "%="] {
        if let Some(pos) = cleaned.find(op) {
            let var = cleaned[..pos].trim();
            let expr = cleaned[pos + op.len()..].trim().replace(";", "");
            let symbol = match op {
                "+=" => "+",
                "-=" => "-",
                "*=" => "*",
                "/=" => "/",
                "%=" => "%",
                _ => unreachable!(),
            };

            // Only use parentheses if expr contains spaces or operators (excluding simple identifiers or numbers)
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

    if cleaned.contains("=") && !cleaned.contains("==") {
        return cleaned.replace(" = ", " ← ").replace("=", " ← ");
    }

    if (cleaned.contains("++") || cleaned.contains("--")) && !cleaned.contains("(") {
        let var = cleaned
            .replace("++", "")
            .replace("--", "")
            .replace(";", "")
            .trim()
            .to_string();
        if cleaned.contains("++") {
            return format!("{} ← {} + 1", var, var);
        } else {
            return format!("{} ← {} - 1", var, var);
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
        !text.contains("=") && !text.contains("++") && !text.contains("--")
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
            text.contains("=") || text.contains("++") || text.contains("--")
        }
        "update_expression" => true,
        _ => false,
    }
}

pub(crate) fn clean_condition(text: &str) -> String {
    let mut cleaned = remove_pointer_syntax(text.trim());
    if cleaned.starts_with('(') && cleaned.ends_with(')') {
        cleaned = cleaned[1..cleaned.len() - 1].to_string();
    }
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
        assert_eq!(clean_text("puts(\"Hello\");"), "Imprimir \"Hello\"");
        assert_eq!(
            clean_text("fputs(\"World\", stdout);"),
            "Imprimir \"World\""
        );
        assert_eq!(clean_text("putchar('A');"), "Imprimir 'A'");
        assert_eq!(clean_text("gets(buf);"), "Leer buf");
        assert_eq!(clean_text("fgets(buf, 10, stdin);"), "Leer buf");
    }
}
