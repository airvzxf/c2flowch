use crate::ir::{FlowNode, Flowchart, NodeType};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use tree_sitter::{Node, Parser};

pub struct C2Flowch<'a> {
    pub(crate) flowchart: Flowchart,
    pub(crate) node_count: usize,
    pub(crate) loop_exits: Vec<String>,
    pub(crate) code: String,
    pub(crate) level: u8,
    pub(crate) known_functions: HashMap<String, Node<'a>>,
    pub(crate) queue: VecDeque<String>,
    pub(crate) processed_functions: HashSet<String>,
    pub(crate) subroutines: Vec<Flowchart>,
    pub(crate) is_main: bool,
}

impl<'a> C2Flowch<'a> {
    pub fn new(
        title: String,
        level: u8,
        code: String,
        known_functions: HashMap<String, Node<'a>>,
    ) -> Self {
        Self {
            flowchart: Flowchart {
                title,
                nodes: Vec::new(),
                edges: Vec::new(),
                subroutines: Vec::new(),
            },
            node_count: 0,
            loop_exits: Vec::new(),
            code,
            level,
            known_functions,
            queue: VecDeque::new(),
            processed_functions: HashSet::new(),
            subroutines: Vec::new(),
            is_main: true,
        }
    }

    pub(crate) fn next_id(&mut self) -> String {
        self.node_count += 1;
        format!("node_{}", self.node_count)
    }

    pub fn check_and_enqueue_subroutines(&mut self, text: &str) {
        let mut keys: Vec<String> = self.known_functions.keys().cloned().collect();
        keys.sort_by_key(|b| std::cmp::Reverse(b.len())); // Check longest first
        for k in keys {
            if text.contains(&format!("{}(", k))
                && !self.processed_functions.contains(&k)
                && !self.queue.contains(&k)
            {
                self.queue.push_back(k.clone());
            }
        }
    }

    fn parse_function(&mut self, name: &str, node: Node, is_main: bool) -> Result<()> {
        self.is_main = is_main;
        let start_id = self.next_id();

        let start_text = if is_main {
            "Inicio".to_string()
        } else {
            let mut params_text = String::new();
            if let Some(decl) = node.child_by_field_name("declarator") {
                let mut current = decl;
                while current.kind() != "function_declarator" {
                    if let Some(n) = current.child_by_field_name("declarator") {
                        current = n;
                    } else if current.child_count() > 0 {
                        current = current.child(0).unwrap();
                    } else {
                        break;
                    }
                }
                if current.kind() == "function_declarator"
                    && let Some(params) = current.child_by_field_name("parameters")
                {
                    let mut param_names = Vec::new();
                    let mut p_cursor = params.walk();
                    for param in params.children(&mut p_cursor) {
                        if param.kind() == "parameter_declaration"
                            && let Some(decl) = param.child_by_field_name("declarator")
                        {
                            let mut name_node = decl;
                            loop {
                                if name_node.kind() == "identifier" {
                                    break;
                                } else if let Some(n) = name_node.child_by_field_name("declarator")
                                {
                                    name_node = n;
                                } else if name_node.child_count() > 0 {
                                    name_node = name_node.child(0).unwrap();
                                } else {
                                    break;
                                }
                            }
                            if name_node.kind() == "identifier" {
                                param_names.push(
                                    self.code[name_node.start_byte()..name_node.end_byte()]
                                        .to_string(),
                                );
                            }
                        }
                    }
                    params_text = format!("({})", param_names.join(", "));
                }
            }
            if params_text.is_empty() || params_text == "()" {
                format!("Inicio: {}()", name)
            } else {
                format!("Inicio: {}{}", name, params_text)
            }
        };

        self.flowchart.add_node(FlowNode {
            id: start_id.clone(),
            node_type: NodeType::Terminator,
            text: start_text,
            source_line: None,
        });

        self.processed_functions.insert(name.to_string());

        if let Some(body) = node.child_by_field_name("body") {
            let last_id = self.traverse(body, Some(start_id.clone()), None)?;
            if let Some(last) = last_id {
                let end_id = self.next_id();
                self.flowchart.add_node(FlowNode {
                    id: end_id.clone(),
                    node_type: NodeType::Terminator,
                    text: if is_main {
                        "Fin".to_string()
                    } else {
                        "Retornar".to_string()
                    },
                    source_line: None,
                });
                self.flowchart.add_edge(&last, &end_id, None);
            }
        }

        Ok(())
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
            "function_definition" => {
                return Ok(current_last);
            }
            "translation_unit" | "compound_statement" | "else_clause" | "case_statement"
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

pub fn parse_file(path: &Path, level: u8) -> Result<Flowchart> {
    let code = fs::read_to_string(path).context("Could not read C file")?;
    let mut parser = Parser::new();
    let language = tree_sitter_c::LANGUAGE.into();
    parser
        .set_language(&language)
        .context("Error setting language")?;

    let tree = parser.parse(&code, None).context("Error parsing C code")?;
    let root = tree.root_node();

    // 1. Find all functions
    let mut known_functions = HashMap::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "function_definition"
            && let Some(decl) = child.child_by_field_name("declarator")
        {
            let mut name_node = decl;
            while name_node.kind() != "identifier" {
                if let Some(n) = name_node.child_by_field_name("declarator") {
                    name_node = n;
                } else if name_node.child_count() > 0 {
                    name_node = name_node.child(0).unwrap();
                } else {
                    break;
                }
            }
            if name_node.kind() == "identifier" {
                let name = &code[name_node.start_byte()..name_node.end_byte()];
                known_functions.insert(name.to_string(), child);
            }
        }
    }

    let file_name = path
        .file_name()
        .context("Could not get file name")?
        .to_string_lossy()
        .into_owned();

    let mut converter = C2Flowch::new(file_name.clone(), level, code.clone(), known_functions);

    // 2. Parse main (if exists) or the first function
    let start_node = if let Some(&n) = converter.known_functions.get("main") {
        Some(("main".to_string(), n))
    } else if let Some((name, &n)) = converter.known_functions.iter().next() {
        Some((name.clone(), n))
    } else {
        None
    };

    if let Some((name, node)) = start_node {
        converter.parse_function(&name, node, true)?;
    }

    // 3. Process subroutines
    while let Some(func_name) = converter.queue.pop_front() {
        if let Some(&node) = converter.known_functions.get(&func_name) {
            let old_flowchart = std::mem::replace(
                &mut converter.flowchart,
                Flowchart {
                    title: func_name.clone(),
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    subroutines: Vec::new(),
                },
            );

            converter.node_count = 0;
            converter.parse_function(&func_name, node, false)?;

            let finished_sub = std::mem::replace(&mut converter.flowchart, old_flowchart);
            converter.subroutines.push(finished_sub);
        }
    }

    converter.flowchart.subroutines = converter.subroutines;
    Ok(converter.flowchart)
}
