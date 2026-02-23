use crate::ir::{FlowNode, Flowchart, NodeType};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tree_sitter::{Node, Parser};

pub struct C2Flowch {
    flowchart: Flowchart,
    node_count: usize,
}

impl C2Flowch {
    pub fn new(title: String, _level: u8) -> Self {
        Self {
            flowchart: Flowchart {
                title,
                nodes: Vec::new(),
                edges: Vec::new(),
            },
            node_count: 0,
        }
    }

    fn next_id(&mut self) -> String {
        self.node_count += 1;
        format!("node_{}", self.node_count)
    }

    fn split_args(s: &str) -> Vec<String> {
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

    fn process_printf(inner: &str) -> String {
        let args = Self::split_args(inner);
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
                if let Some(&(_, next_c)) = chars.peek() {
                    if next_c == '%' {
                        chars.next();
                        continue;
                    }
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

        format!("Imprimir {}", parts.join(", "))
    }

    pub fn parse_file(path: &Path, level: u8) -> Result<Flowchart> {
        let code = fs::read_to_string(path).context("Could not read C file")?;
        let mut parser = Parser::new();
        let language = tree_sitter_c::LANGUAGE.into();
        parser
            .set_language(&language)
            .context("Error setting language")?;

        let tree = parser.parse(&code, None).context("Error parsing C code")?;
        let mut converter = Self::new(
            path.file_name().unwrap().to_string_lossy().into_owned(),
            level,
        );

        let start_id = converter.next_id();
        converter.flowchart.add_node(FlowNode {
            id: start_id.clone(),
            node_type: NodeType::Terminator,
            text: "Inicio".to_string(),
            source_line: None,
        });

        converter.traverse(tree.root_node(), &code, Some(start_id), None)?;

        Ok(converter.flowchart)
    }

    fn clean_text(text: &str) -> String {
        let mut cleaned = text.trim().to_string();

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

        cleaned = cleaned.replace(";", "").trim().to_string();

        if cleaned.contains("printf(") {
            let start = cleaned.find("printf(").unwrap();
            // Ensure we don't panic if string slice is invalid or empty
            if cleaned.len() > start + 7 {
                let inner = &cleaned[start + 7..cleaned.len() - 1];
                return Self::process_printf(inner);
            }
            return "Imprimir".to_string();
        } else if cleaned.contains("scanf(") {
            let start = cleaned.find("scanf(").unwrap();
            let inner = &cleaned[start + 6..cleaned.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let var = inner[comma_pos + 1..].trim().replace("&", "");
                return format!("Leer {}", var);
            }
            return "Leer".to_string();
        }

        if cleaned.contains("=") && !cleaned.contains("==") {
            return cleaned.replace(" = ", " ← ").replace("=", " ← ");
        }

        cleaned
    }

    fn traverse(
        &mut self,
        node: Node,
        code: &str,
        last_node_id: Option<String>,
        label: Option<String>,
    ) -> Result<Option<String>> {
        let mut current_last = last_node_id;
        let mut current_label = label;

        match node.kind() {
            "translation_unit" | "function_definition" | "compound_statement" | "else_clause" => {
                let mut cursor = node.walk();
                let children: Vec<Node> = node.children(&mut cursor).collect();
                let mut i = 0;
                while i < children.len() {
                    let child = children[i];

                    if child.kind() == "comment" {
                        i += 1;
                        continue;
                    }

                    if self.is_io(child, code) {
                        let mut group_text = Vec::new();
                        let mut j = i;
                        while j < children.len() {
                            if children[j].kind() == "comment" {
                                j += 1;
                                continue;
                            }
                            if self.is_io(children[j], code) {
                                let text = Self::clean_text(
                                    &code[children[j].start_byte()..children[j].end_byte()],
                                );
                                if !text.is_empty() {
                                    group_text.push(text);
                                }
                                j += 1;
                            } else {
                                break;
                            }
                        }

                        if !group_text.is_empty() {
                            let node_id = self.next_id();
                            self.flowchart.add_node(FlowNode {
                                id: node_id.clone(),
                                node_type: NodeType::InputOutput,
                                text: group_text.join("\n"),
                                source_line: Some(child.start_position().row + 1),
                            });
                            if let Some(prev) = current_last {
                                self.flowchart
                                    .add_edge(&prev, &node_id, current_label.take()); // Consume label here
                            }
                            current_last = Some(node_id);
                        }
                        i = j;
                        continue;
                    } else if self.is_process(child, code) {
                        let mut group_text = Vec::new();
                        let mut j = i;
                        while j < children.len() {
                            if children[j].kind() == "comment" {
                                j += 1;
                                continue;
                            }
                            if self.is_process(children[j], code) {
                                let text = Self::clean_text(
                                    &code[children[j].start_byte()..children[j].end_byte()],
                                );
                                if !text.is_empty() {
                                    group_text.push(text);
                                }
                                j += 1;
                            } else {
                                break;
                            }
                        }

                        if !group_text.is_empty() {
                            let node_id = self.next_id();
                            self.flowchart.add_node(FlowNode {
                                id: node_id.clone(),
                                node_type: NodeType::Process,
                                text: group_text.join("\n"),
                                source_line: Some(child.start_position().row + 1),
                            });
                            if let Some(prev) = current_last {
                                self.flowchart
                                    .add_edge(&prev, &node_id, current_label.take()); // Consume label here
                            }
                            current_last = Some(node_id);
                        }
                        i = j;
                        continue;
                    }

                    // For non-leaf nodes, they might or might not generate a node.
                    // If traverse returns a node, we should consume the label.
                    let prev_last = current_last.clone();
                    current_last =
                        self.traverse(child, code, current_last, current_label.clone())?;
                    if current_last != prev_last {
                        current_label = None; // A node was added, consume label
                    }
                    i += 1;
                }
            }
            "if_statement" => {
                let condition = node.child_by_field_name("condition").unwrap();
                let cond_text = &code[condition.start_byte()..condition.end_byte()];
                let if_id = self.next_id();

                self.flowchart.add_node(FlowNode {
                    id: if_id.clone(),
                    node_type: NodeType::Decision,
                    text: cond_text
                        .to_string()
                        .replace("(", "")
                        .replace(")", "")
                        .replace("==", "="),
                    source_line: Some(node.start_position().row + 1),
                });

                if let Some(prev) = current_last {
                    self.flowchart.add_edge(&prev, &if_id, current_label.take());
                }

                let consequence = node.child_by_field_name("consequence").unwrap();
                let true_last = self.traverse(
                    consequence,
                    code,
                    Some(if_id.clone()),
                    Some("Verdadero".to_string()),
                )?;

                let mut false_last = None;
                if let Some(alternative) = node.child_by_field_name("alternative") {
                    false_last = self.traverse(
                        alternative,
                        code,
                        Some(if_id.clone()),
                        Some("Falso".to_string()),
                    )?;
                }

                let join_id = self.next_id();
                self.flowchart.add_node(FlowNode {
                    id: join_id.clone(),
                    node_type: NodeType::Process,
                    text: "Merge".to_string(),
                    source_line: None,
                });

                if let Some(t) = true_last {
                    self.flowchart.add_edge(&t, &join_id, None);
                }
                if let Some(f) = false_last {
                    self.flowchart.add_edge(&f, &join_id, None);
                } else {
                    self.flowchart
                        .add_edge(&if_id, &join_id, Some("Falso".to_string()));
                }
                current_last = Some(join_id);
            }
            "do_statement" => {
                let body = node.child_by_field_name("body").unwrap();
                let condition = node.child_by_field_name("condition").unwrap();
                let cond_text = &code[condition.start_byte()..condition.end_byte()];

                // Capture index of next node to be added
                let pre_body_node_idx = self.flowchart.nodes.len();
                let loop_predecessor = current_last.clone();

                let body_last = self.traverse(body, code, current_last, current_label.take())?;

                let dec_id = self.next_id();
                self.flowchart.add_node(FlowNode {
                    id: dec_id.clone(),
                    node_type: NodeType::Decision,
                    text: cond_text
                        .to_string()
                        .replace("(", "")
                        .replace(")", "")
                        .replace("==", "="),
                    source_line: Some(condition.start_position().row + 1),
                });

                if let Some(bl) = body_last {
                    self.flowchart.add_edge(&bl, &dec_id, None);
                }

                // Link loop back to body start
                if self.flowchart.nodes.len() > pre_body_node_idx {
                    let body_start_id = self.flowchart.nodes[pre_body_node_idx].id.clone();
                    self.flowchart
                        .add_edge(&dec_id, &body_start_id, Some("Verdadero".to_string()));
                } else {
                    // Fallback if body was empty
                    if let Some(sl) = loop_predecessor {
                        self.flowchart
                            .add_edge(&dec_id, &sl, Some("Verdadero".to_string()));
                    } else {
                        // Fallback self-loop
                        self.flowchart
                            .add_edge(&dec_id, &dec_id, Some("Verdadero".to_string()));
                    }
                }
                current_last = Some(dec_id);
            }
            "return_statement" => {
                let node_id = self.next_id();
                self.flowchart.add_node(FlowNode {
                    id: node_id.clone(),
                    node_type: NodeType::Terminator,
                    text: "Fin".to_string(),
                    source_line: Some(node.start_position().row + 1),
                });
                if let Some(prev) = current_last {
                    self.flowchart
                        .add_edge(&prev, &node_id, current_label.take());
                }
                current_last = None;
            }
            _ => {
                if node.child_count() > 0 {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        let prev_last = current_last.clone();
                        current_last =
                            self.traverse(child, code, current_last, current_label.clone())?;
                        if current_last != prev_last {
                            current_label = None;
                        }
                    }
                }
            }
        }
        Ok(current_last)
    }

    fn is_io(&self, node: Node, code: &str) -> bool {
        if node.kind() == "expression_statement" {
            let text = &code[node.start_byte()..node.end_byte()];
            text.contains("printf(") || text.contains("scanf(")
        } else {
            false
        }
    }

    fn is_process(&self, node: Node, code: &str) -> bool {
        match node.kind() {
            "expression_statement" => {
                let text = &code[node.start_byte()..node.end_byte()];
                !text.contains("printf(") && !text.contains("scanf(")
            }
            "declaration" => {
                let text = &code[node.start_byte()..node.end_byte()];
                text.contains("=")
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_args_basic() {
        let args = C2Flowch::split_args("\"fmt\", var1, var2");
        assert_eq!(args, vec!["\"fmt\"", "var1", "var2"]);
    }

    #[test]
    fn test_process_printf_basic() {
        let res = C2Flowch::process_printf("\"Value: %d\", x");
        assert_eq!(res, "Imprimir \"Value: \", x");
    }
}
