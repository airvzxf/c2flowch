use crate::ir::{FlowNode, NodeType};
use crate::parser::core::C2Flowch;
use anyhow::Result;
use tree_sitter::Node;

pub(crate) fn parse_translation_unit(
    parser: &mut C2Flowch,
    node: Node,
    code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    let mut i = 0;
    while i < children.len() {
        let child = children[i];

        if child.kind() == "comment" {
            i += 1;
            continue;
        }

        if crate::parser::utils::is_io(child, code) {
            let mut group_text = Vec::new();
            let mut j = i;
            while j < children.len() {
                if children[j].kind() == "comment" {
                    j += 1;
                    continue;
                }
                if crate::parser::utils::is_io(children[j], code) {
                    let text = crate::parser::utils::clean_text(
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
                let node_id = parser.next_id();
                parser.flowchart.add_node(FlowNode {
                    id: node_id.clone(),
                    node_type: NodeType::InputOutput,
                    text: group_text.join("\n"),
                    source_line: Some(child.start_position().row + 1),
                });
                if let Some(prev) = current_last {
                    parser
                        .flowchart
                        .add_edge(&prev, &node_id, current_label.take()); // Consume label here
                }
                current_last = Some(node_id);
            }
            i = j;
            continue;
        } else if crate::parser::utils::is_process(child, code) {
            let mut group_text = Vec::new();
            let mut j = i;
            while j < children.len() {
                if children[j].kind() == "comment" {
                    j += 1;
                    continue;
                }
                if crate::parser::utils::is_process(children[j], code) {
                    let text = crate::parser::utils::clean_text(
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
                let node_id = parser.next_id();
                parser.flowchart.add_node(FlowNode {
                    id: node_id.clone(),
                    node_type: NodeType::Process,
                    text: group_text.join("\n"),
                    source_line: Some(child.start_position().row + 1),
                });
                if let Some(prev) = current_last {
                    parser
                        .flowchart
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
        current_last = parser.traverse(child, code, current_last, current_label.clone())?;
        if current_last != prev_last {
            current_label = None; // A node was added, consume label
        }
        i += 1;
    }

    Ok(current_last)
}

pub(crate) fn parse_if_statement(
    parser: &mut C2Flowch,
    node: Node,
    code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let condition = node.child_by_field_name("condition").unwrap();
    let cond_text = &code[condition.start_byte()..condition.end_byte()];
    let if_id = parser.next_id();

    parser.flowchart.add_node(FlowNode {
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
        parser
            .flowchart
            .add_edge(&prev, &if_id, current_label.take());
    }

    let consequence = node.child_by_field_name("consequence").unwrap();
    let true_last = parser.traverse(
        consequence,
        code,
        Some(if_id.clone()),
        Some("Verdadero".to_string()),
    )?;

    let mut false_last = None;
    if let Some(alternative) = node.child_by_field_name("alternative") {
        false_last = parser.traverse(
            alternative,
            code,
            Some(if_id.clone()),
            Some("Falso".to_string()),
        )?;
    }

    let join_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: join_id.clone(),
        node_type: NodeType::Process,
        text: "Merge".to_string(),
        source_line: None,
    });

    if let Some(t) = true_last {
        parser.flowchart.add_edge(&t, &join_id, None);
    }
    if let Some(f) = false_last {
        parser.flowchart.add_edge(&f, &join_id, None);
    } else {
        parser
            .flowchart
            .add_edge(&if_id, &join_id, Some("Falso".to_string()));
    }
    current_last = Some(join_id);

    Ok(current_last)
}

pub(crate) fn parse_while_statement(
    parser: &mut C2Flowch,
    node: Node,
    code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let condition = node.child_by_field_name("condition").unwrap();
    let body = node.child_by_field_name("body").unwrap();
    let cond_text = &code[condition.start_byte()..condition.end_byte()];

    // 1. Pre-condition Merge (Join point for back-edge)
    let join_cond_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: join_cond_id.clone(),
        node_type: NodeType::Process,
        text: "Merge".to_string(),
        source_line: None,
    });

    if let Some(prev) = current_last {
        parser
            .flowchart
            .add_edge(&prev, &join_cond_id, current_label.take());
    }

    // 2. Decision (Condition)
    let dec_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: dec_id.clone(),
        node_type: NodeType::Decision,
        text: cond_text
            .to_string()
            .replace("(", "")
            .replace(")", "")
            .replace("==", "="),
        source_line: Some(condition.start_position().row + 1),
    });

    parser.flowchart.add_edge(&join_cond_id, &dec_id, None);

    // 3. Create Exit early for breaks
    let join_exit_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: join_exit_id.clone(),
        node_type: NodeType::Process,
        text: "MergeExit".to_string(),
        source_line: None,
    });
    parser.loop_exits.push(join_exit_id.clone());

    // 4. Body
    let body_last = parser.traverse(
        body,
        code,
        Some(dec_id.clone()),
        Some("Verdadero".to_string()),
    )?;

    parser.loop_exits.pop();

    // Back-edge to Pre-condition Merge
    if let Some(bl) = body_last {
        parser.flowchart.add_edge(&bl, &join_cond_id, None);
        // Add dependency to ensure exit is below body
        parser
            .flowchart
            .add_edge(&bl, &join_exit_id, Some("Dependency".to_string()));
    }

    parser
        .flowchart
        .add_edge(&dec_id, &join_exit_id, Some("Falso".to_string()));

    current_last = Some(join_exit_id);

    Ok(current_last)
}

pub(crate) fn parse_do_statement(
    parser: &mut C2Flowch,
    node: Node,
    code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let body = node.child_by_field_name("body").unwrap();
    let condition = node.child_by_field_name("condition").unwrap();
    let cond_text = &code[condition.start_byte()..condition.end_byte()];

    // Create exit node for breaks
    let exit_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: exit_id.clone(),
        node_type: NodeType::Process,
        text: "MergeExit".to_string(),
        source_line: None,
    });
    parser.loop_exits.push(exit_id.clone());

    // Create an explicit entry Merge node for the loop back edge
    let entry_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: entry_id.clone(),
        node_type: NodeType::Process,
        text: "Merge".to_string(),
        source_line: None,
    });

    if let Some(prev) = current_last {
        parser
            .flowchart
            .add_edge(&prev, &entry_id, current_label.take());
    }

    let body_last = parser.traverse(body, code, Some(entry_id.clone()), None)?;

    parser.loop_exits.pop();

    let dec_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
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
        parser.flowchart.add_edge(&bl, &dec_id, None);
    } else {
        parser.flowchart.add_edge(&entry_id, &dec_id, None);
    }

    // Link loop back to entry
    parser
        .flowchart
        .add_edge(&dec_id, &entry_id, Some("Verdadero".to_string()));
    parser
        .flowchart
        .add_edge(&dec_id, &exit_id, Some("Falso".to_string()));

    current_last = Some(exit_id);

    Ok(current_last)
}

pub(crate) fn parse_for_statement(
    parser: &mut C2Flowch,
    node: Node,
    code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let init = node.child_by_field_name("initializer");
    let cond = node.child_by_field_name("condition");
    let update = node.child_by_field_name("update");
    let body = node.child_by_field_name("body").unwrap();

    // 1. Initializer
    if let Some(i) = init {
        let i_text = &code[i.start_byte()..i.end_byte()];
        let clean_i = crate::parser::utils::clean_text(i_text);
        let init_id = parser.next_id();
        parser.flowchart.add_node(FlowNode {
            id: init_id.clone(),
            node_type: NodeType::Process,
            text: clean_i,
            source_line: Some(i.start_position().row + 1),
        });
        if let Some(prev) = current_last {
            parser
                .flowchart
                .add_edge(&prev, &init_id, current_label.take());
        }
        current_last = Some(init_id);
    }

    // 2. Pre-condition Merge (Join point for back-edge)
    let join_cond_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: join_cond_id.clone(),
        node_type: NodeType::Process,
        text: "Merge".to_string(),
        source_line: None,
    });

    if let Some(prev) = current_last {
        parser
            .flowchart
            .add_edge(&prev, &join_cond_id, current_label.take());
    }

    // 3. Decision (Condition)
    let dec_id = parser.next_id();
    let cond_text = if let Some(c) = cond {
        code[c.start_byte()..c.end_byte()]
            .to_string()
            .replace("(", "")
            .replace(")", "")
            .replace("==", "=")
    } else {
        "true".to_string()
    };

    parser.flowchart.add_node(FlowNode {
        id: dec_id.clone(),
        node_type: NodeType::Decision,
        text: cond_text,
        source_line: cond.map(|c| c.start_position().row + 1),
    });

    parser.flowchart.add_edge(&join_cond_id, &dec_id, None);

    // 3.5 Exit (Falso) - Created early for breaks
    let join_exit_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: join_exit_id.clone(),
        node_type: NodeType::Process,
        text: "MergeExit".to_string(),
        source_line: None,
    });
    parser.loop_exits.push(join_exit_id.clone());

    // 4. Body
    let body_last = parser.traverse(
        body,
        code,
        Some(dec_id.clone()),
        Some("Verdadero".to_string()),
    )?;

    parser.loop_exits.pop();

    // 5. Update
    let mut last_before_back = body_last.clone();
    if let Some(u) = update {
        // Force update to be its own node if it's an expression
        let u_text = &code[u.start_byte()..u.end_byte()];
        let clean_u = crate::parser::utils::clean_text(u_text);
        let u_id = parser.next_id();
        parser.flowchart.add_node(FlowNode {
            id: u_id.clone(),
            node_type: NodeType::Process,
            text: clean_u,
            source_line: Some(u.start_position().row + 1),
        });
        if let Some(bl) = body_last {
            parser.flowchart.add_edge(&bl, &u_id, None);
        }
        last_before_back = Some(u_id);
    }

    // Back-edge to Pre-condition Merge
    if let Some(lbl) = last_before_back {
        parser.flowchart.add_edge(&lbl, &join_cond_id, None);
        // Add dependency to ensure exit is below body
        parser
            .flowchart
            .add_edge(&lbl, &join_exit_id, Some("Dependency".to_string()));
    }

    parser
        .flowchart
        .add_edge(&dec_id, &join_exit_id, Some("Falso".to_string()));

    current_last = Some(join_exit_id);

    Ok(current_last)
}

pub(crate) fn parse_switch_statement(
    parser: &mut C2Flowch,
    node: Node,
    code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let condition = node.child_by_field_name("condition").unwrap();
    let cond_text = &code[condition.start_byte()..condition.end_byte()];
    let cond_clean = cond_text.to_string().replace("(", "").replace(")", "");

    let switch_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: switch_id.clone(),
        node_type: NodeType::Decision, // Esto dibujará el Rombo/Diamante
        text: cond_clean,
        source_line: Some(node.start_position().row + 1),
    });

    if let Some(prev) = current_last {
        parser
            .flowchart
            .add_edge(&prev, &switch_id, current_label.take());
    }

    let exit_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: exit_id.clone(),
        node_type: NodeType::Process,
        text: "MergeExit".to_string(),
        source_line: None,
    });

    parser.loop_exits.push(exit_id.clone());

    let body = node.child_by_field_name("body").unwrap();
    let mut spine_last = switch_id.clone();
    let mut fallthrough_id: Option<String> = None;
    let mut prev_case_nodes: Vec<String> = Vec::new();

    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if child.kind() == "case_statement" || child.kind() == "default_statement" {
            let is_default =
                child.kind() == "default_statement" || child.child_by_field_name("value").is_none();

            let case_text = if is_default {
                "default".to_string()
            } else {
                let val_node = child.child_by_field_name("value").unwrap();
                code[val_node.start_byte()..val_node.end_byte()].to_string()
            };

            let case_id = parser.next_id();
            parser.flowchart.add_node(FlowNode {
                id: case_id.clone(),
                node_type: NodeType::SwitchCase,
                text: case_text,
                source_line: Some(child.start_position().row + 1),
            });

            // Make this spine node wait for all nodes of the previous case
            for prev_node in &prev_case_nodes {
                parser
                    .flowchart
                    .add_edge(prev_node, &case_id, Some("Dependency".to_string()));
            }

            let spine_label = if spine_last == switch_id {
                None
            } else {
                Some("Falso".to_string())
            };
            parser
                .flowchart
                .add_edge(&spine_last, &case_id, spine_label);
            spine_last = case_id.clone();

            let case_entry_id = parser.next_id();
            parser.flowchart.add_node(FlowNode {
                id: case_entry_id.clone(),
                node_type: NodeType::Process,
                text: "MergeEntry".to_string(),
                source_line: None,
            });

            parser
                .flowchart
                .add_edge(&case_id, &case_entry_id, Some("Verdadero".to_string()));
            if let Some(ft) = fallthrough_id {
                parser.flowchart.add_edge(&ft, &case_entry_id, None);
            }

            let start_idx = parser.flowchart.nodes.len();
            let case_last = parser.traverse(child, code, Some(case_entry_id), None)?;
            fallthrough_id = case_last;
            let end_idx = parser.flowchart.nodes.len();

            prev_case_nodes = parser.flowchart.nodes[start_idx..end_idx]
                .iter()
                .map(|n| n.id.clone())
                .collect();
        }
    }

    if let Some(ft) = fallthrough_id {
        parser.flowchart.add_edge(&ft, &exit_id, None);
    }

    // Exit node waits for the last case's nodes too
    for prev_node in &prev_case_nodes {
        parser
            .flowchart
            .add_edge(prev_node, &exit_id, Some("Dependency".to_string()));
    }

    parser
        .flowchart
        .add_edge(&spine_last, &exit_id, Some("Falso".to_string()));

    parser.loop_exits.pop();
    current_last = Some(exit_id);

    Ok(current_last)
}

pub(crate) fn parse_break_statement(
    parser: &mut C2Flowch,
    _node: Node,
    _code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let _current_label = label;

    if let Some(prev) = current_last
        && let Some(exit_id) = parser.loop_exits.last()
    {
        parser
            .flowchart
            .add_edge(&prev, exit_id, Some("__Break__".to_string()));
    }
    current_last = None;

    Ok(current_last)
}

pub(crate) fn parse_return_statement(
    parser: &mut C2Flowch,
    node: Node,
    _code: &str,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let node_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: node_id.clone(),
        node_type: NodeType::Terminator,
        text: "Fin".to_string(),
        source_line: Some(node.start_position().row + 1),
    });
    if let Some(prev) = current_last {
        parser
            .flowchart
            .add_edge(&prev, &node_id, current_label.take());
    }
    current_last = None;

    Ok(current_last)
}
