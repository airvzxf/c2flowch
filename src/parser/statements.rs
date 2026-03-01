use crate::ir::{
    EDGE_LABEL_BREAK, EDGE_LABEL_DEPENDENCY, EDGE_LABEL_FALSE, EDGE_LABEL_TRUE, FlowNode, NodeType,
};
use crate::parser::core::C2Flowch;
use anyhow::{Context, Result};
use tree_sitter::Node;

pub(crate) fn parse_translation_unit(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();

    let keys: Vec<String> = parser.known_functions.keys().cloned().collect();
    let check_prep = |n: Node, c: &str| crate::parser::utils::is_preparation(n, c);
    let check_io = |n: Node, c: &str| crate::parser::utils::is_io(n, c);

    let mut i = 0;
    while i < children.len() {
        let child = children[i];

        if child.kind() == "comment" {
            i += 1;
            continue;
        }

        let keys_for_predef = keys.clone();
        let check_predef = move |n: Node, c: &str| {
            if !crate::parser::utils::is_process(n, c) {
                return false;
            }
            let text = &c[n.start_byte()..n.end_byte()];
            keys_for_predef
                .iter()
                .any(|k| text.contains(&format!("{k}(")))
        };

        let keys_for_proc = keys.clone();
        let check_proc = move |n: Node, c: &str| {
            if !crate::parser::utils::is_process(n, c) {
                return false;
            }
            let text = &c[n.start_byte()..n.end_byte()];
            !keys_for_proc
                .iter()
                .any(|k| text.contains(&format!("{k}(")))
        };

        type RuleFn<'a> = &'a dyn Fn(Node, &str) -> bool;
        let rules: [(NodeType, RuleFn, bool); 4] = [
            (NodeType::Preparation, &check_prep, parser.level >= 4),
            (NodeType::InputOutput, &check_io, true),
            (NodeType::PredefinedProcess, &check_predef, true),
            (NodeType::Process, &check_proc, true),
        ];

        let mut grouped = false;
        for (node_type, check_fn, condition) in rules {
            if condition
                && check_fn(child, parser.code)
                && let Some(next_idx) = try_parse_group(
                    parser,
                    &children,
                    i,
                    node_type,
                    check_fn,
                    &mut current_last,
                    &mut current_label,
                )
            {
                i = next_idx;
                grouped = true;
                break;
            }
        }

        if grouped {
            continue;
        }

        // For non-leaf nodes, they might or might not generate a node.
        // If traverse returns a node, we should consume the label.
        let prev_last = current_last.clone();
        current_last = parser.traverse(child, current_last, current_label.clone())?;
        if current_last != prev_last {
            current_label = None; // A node was added, consume label
        }
        i += 1;
    }

    Ok(current_last)
}

fn try_parse_group<F>(
    parser: &mut C2Flowch,
    children: &[Node],
    start_index: usize,
    node_type: NodeType,
    check_fn: F,
    current_last: &mut Option<String>,
    current_label: &mut Option<String>,
) -> Option<usize>
where
    F: Fn(Node, &str) -> bool,
{
    let child = children[start_index];
    let mut group_text = Vec::new();
    let mut j = start_index;

    while j < children.len() {
        let c = children[j];
        if c.kind() == "comment" {
            j += 1;
            continue;
        }
        if check_fn(c, parser.code) {
            let text = crate::parser::utils::clean_text(&parser.code[c.start_byte()..c.end_byte()]);
            if !text.is_empty() {
                group_text.push(text);
            }
            j += 1;
        } else {
            break;
        }
    }

    if group_text.is_empty() {
        None
    } else {
        let full_text = group_text.join("\n");

        parser.check_and_enqueue_subroutines(&full_text);

        let node_id = parser.next_id();
        parser.flowchart.add_node(FlowNode {
            id: node_id.clone(),
            node_type,
            text: full_text,
            source_line: Some(child.start_position().row + 1),
        });
        if let Some(prev) = current_last.take() {
            parser
                .flowchart
                .add_edge(&prev, &node_id, current_label.take());
        }
        *current_last = Some(node_id);
        Some(j)
    }
}

pub(crate) fn parse_if_statement(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let condition = node
        .child_by_field_name("condition")
        .context("Missing condition in if statement")?;
    let cond_text = parser.code[condition.start_byte()..condition.end_byte()].to_string();

    parser.check_and_enqueue_subroutines(&cond_text);

    let if_id = parser.next_id();

    parser.flowchart.add_node(FlowNode {
        id: if_id.clone(),
        node_type: NodeType::Decision,
        text: crate::parser::utils::clean_condition(&cond_text),
        source_line: Some(node.start_position().row + 1),
    });

    if let Some(prev) = current_last {
        parser
            .flowchart
            .add_edge(&prev, &if_id, current_label.take());
    }

    let consequence = node
        .child_by_field_name("consequence")
        .context("Missing consequence in if statement")?;
    let true_last = parser.traverse(
        consequence,
        Some(if_id.clone()),
        Some(EDGE_LABEL_TRUE.to_string()),
    )?;

    let mut false_last = None;
    if let Some(alternative) = node.child_by_field_name("alternative") {
        false_last = parser.traverse(
            alternative,
            Some(if_id.clone()),
            Some(EDGE_LABEL_FALSE.to_string()),
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
            .add_edge(&if_id, &join_id, Some(EDGE_LABEL_FALSE.to_string()));
    }
    current_last = Some(join_id);

    Ok(current_last)
}

pub(crate) fn parse_while_statement(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let condition = node
        .child_by_field_name("condition")
        .context("Missing condition in while statement")?;
    let body = node
        .child_by_field_name("body")
        .context("Missing body in while statement")?;
    let cond_text = parser.code[condition.start_byte()..condition.end_byte()].to_string();

    parser.check_and_enqueue_subroutines(&cond_text);

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
        text: crate::parser::utils::clean_condition(&cond_text),
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
        Some(dec_id.clone()),
        Some(EDGE_LABEL_TRUE.to_string()),
    )?;

    parser.loop_exits.pop();

    // Back-edge to Pre-condition Merge
    if let Some(bl) = body_last {
        parser.flowchart.add_edge(&bl, &join_cond_id, None);
        // Add dependency to ensure exit is below body
        parser
            .flowchart
            .add_edge(&bl, &join_exit_id, Some(EDGE_LABEL_DEPENDENCY.to_string()));
    }

    parser
        .flowchart
        .add_edge(&dec_id, &join_exit_id, Some(EDGE_LABEL_FALSE.to_string()));

    current_last = Some(join_exit_id);

    Ok(current_last)
}

pub(crate) fn parse_do_statement(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let body = node
        .child_by_field_name("body")
        .context("Missing body in do statement")?;
    let condition = node
        .child_by_field_name("condition")
        .context("Missing condition in do statement")?;
    let cond_text = parser.code[condition.start_byte()..condition.end_byte()].to_string();

    parser.check_and_enqueue_subroutines(&cond_text);

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

    let body_last = parser.traverse(body, Some(entry_id.clone()), None)?;

    parser.loop_exits.pop();

    let dec_id = parser.next_id();
    parser.flowchart.add_node(FlowNode {
        id: dec_id.clone(),
        node_type: NodeType::Decision,
        text: crate::parser::utils::clean_condition(&cond_text),
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
        .add_edge(&dec_id, &entry_id, Some(EDGE_LABEL_TRUE.to_string()));
    parser
        .flowchart
        .add_edge(&dec_id, &exit_id, Some(EDGE_LABEL_FALSE.to_string()));

    current_last = Some(exit_id);

    Ok(current_last)
}

pub(crate) fn parse_for_statement(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let init = node.child_by_field_name("initializer");
    let cond = node.child_by_field_name("condition");
    let update = node.child_by_field_name("update");
    let body = node
        .child_by_field_name("body")
        .context("Missing body in for statement")?;

    // 1. Initializer
    if let Some(i) = init {
        let i_text = parser.code[i.start_byte()..i.end_byte()].to_string();
        let clean_i = crate::parser::utils::clean_text(&i_text);
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
        let text = parser.code[c.start_byte()..c.end_byte()].to_string();
        parser.check_and_enqueue_subroutines(&text);
        crate::parser::utils::clean_condition(&text)
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
        Some(dec_id.clone()),
        Some(EDGE_LABEL_TRUE.to_string()),
    )?;

    parser.loop_exits.pop();

    // 5. Update
    let mut last_before_back = body_last.clone();
    if let Some(u) = update {
        // Force update to be its own node if it's an expression
        let u_text = parser.code[u.start_byte()..u.end_byte()].to_string();
        let clean_u = crate::parser::utils::clean_text(&u_text);
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
            .add_edge(&lbl, &join_exit_id, Some(EDGE_LABEL_DEPENDENCY.to_string()));
    }

    parser
        .flowchart
        .add_edge(&dec_id, &join_exit_id, Some(EDGE_LABEL_FALSE.to_string()));

    current_last = Some(join_exit_id);

    Ok(current_last)
}

pub(crate) fn parse_switch_statement(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let condition = node
        .child_by_field_name("condition")
        .context("Missing condition in switch statement")?;
    let cond_text = parser.code[condition.start_byte()..condition.end_byte()].to_string();

    parser.check_and_enqueue_subroutines(&cond_text);

    let cond_clean = crate::parser::utils::clean_condition(&cond_text);

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

    let body = node
        .child_by_field_name("body")
        .context("Missing body in switch statement")?;
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
                let val_node = child
                    .child_by_field_name("value")
                    .context("Missing value in case statement")?;
                parser.code[val_node.start_byte()..val_node.end_byte()].to_string()
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
                parser.flowchart.add_edge(
                    prev_node,
                    &case_id,
                    Some(EDGE_LABEL_DEPENDENCY.to_string()),
                );
            }

            let spine_label = if spine_last == switch_id {
                None
            } else {
                Some(EDGE_LABEL_FALSE.to_string())
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
                .add_edge(&case_id, &case_entry_id, Some(EDGE_LABEL_TRUE.to_string()));
            if let Some(ft) = fallthrough_id {
                parser.flowchart.add_edge(&ft, &case_entry_id, None);
            }

            let start_idx = parser.flowchart.nodes.len();
            let case_last = parser.traverse(child, Some(case_entry_id), None)?;
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
            .add_edge(prev_node, &exit_id, Some(EDGE_LABEL_DEPENDENCY.to_string()));
    }

    parser
        .flowchart
        .add_edge(&spine_last, &exit_id, Some(EDGE_LABEL_FALSE.to_string()));

    parser.loop_exits.pop();
    current_last = Some(exit_id);

    Ok(current_last)
}

pub(crate) fn parse_break_statement(
    parser: &mut C2Flowch,
    _node: Node,
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
            .add_edge(&prev, exit_id, Some(EDGE_LABEL_BREAK.to_string()));
    }
    current_last = None;

    Ok(current_last)
}

pub(crate) fn parse_return_statement(
    parser: &mut C2Flowch,
    node: Node,
    last_node_id: Option<String>,
    label: Option<String>,
) -> Result<Option<String>> {
    let mut current_last = last_node_id;
    let mut current_label = label;

    let node_id = parser.next_id();

    let raw_text = parser.code[node.start_byte()..node.end_byte()].to_string();
    let text = if parser.is_main {
        "Fin".to_string()
    } else {
        let clean = raw_text
            .replace("return", "")
            .replace(';', "")
            .trim()
            .to_string();
        let clean = crate::parser::utils::remove_pointer_syntax(&clean);
        if clean.trim().is_empty() {
            "Retornar".to_string()
        } else {
            format!("Retornar: {}", clean.trim())
        }
    };

    parser.flowchart.add_node(FlowNode {
        id: node_id.clone(),
        node_type: NodeType::Terminator,
        text,
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
