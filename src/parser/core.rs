use crate::ir::{FlowNode, Flowchart, NodeType};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tree_sitter::{Node, Parser};

pub struct C2Flowch {
    pub(crate) flowchart: Flowchart,
    pub(crate) node_count: usize,
    pub(crate) loop_exits: Vec<String>,
    pub(crate) code: String,
}

impl C2Flowch {
    pub fn new(title: String, _level: u8, code: String) -> Self {
        Self {
            flowchart: Flowchart {
                title,
                nodes: Vec::new(),
                edges: Vec::new(),
            },
            node_count: 0,
            loop_exits: Vec::new(),
            code,
        }
    }

    pub(crate) fn next_id(&mut self) -> String {
        self.node_count += 1;
        format!("node_{}", self.node_count)
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
            code.clone(),
        );

        let start_id = converter.next_id();
        converter.flowchart.add_node(FlowNode {
            id: start_id.clone(),
            node_type: NodeType::Terminator,
            text: "Inicio".to_string(),
            source_line: None,
        });

        converter.traverse(tree.root_node(), Some(start_id), None)?;

        Ok(converter.flowchart)
    }

    pub(crate) fn traverse(
        &mut self,
        node: Node,
        last_node_id: Option<String>,
        label: Option<String>,
    ) -> Result<Option<String>> {
        let mut current_last = last_node_id;
        let mut current_label = label;

        match node.kind() {
            "translation_unit"
            | "function_definition"
            | "compound_statement"
            | "else_clause"
            | "case_statement"
            | "default_statement" => {
                return crate::parser::statements::parse_translation_unit(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "if_statement" => {
                return crate::parser::statements::parse_if_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "while_statement" => {
                return crate::parser::statements::parse_while_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "for_statement" => {
                return crate::parser::statements::parse_for_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "do_statement" => {
                return crate::parser::statements::parse_do_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "break_statement" => {
                return crate::parser::statements::parse_break_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "switch_statement" => {
                return crate::parser::statements::parse_switch_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            "return_statement" => {
                return crate::parser::statements::parse_return_statement(
                    self,
                    node,
                    current_last,
                    current_label,
                );
            }
            _ => {
                if node.child_count() > 0 {
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        let prev_last = current_last.clone();
                        current_last = self.traverse(child, current_last, current_label.clone())?;
                        if current_last != prev_last {
                            current_label = None;
                        }
                    }
                }
            }
        }
        Ok(current_last)
    }
}
