use crate::ir::{Flowchart, NodeType};
use crate::render::canvas::AsciiRenderer;
use crate::render::routing::RoutingDrawing;
use crate::render::shapes::ShapeDrawing;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct NodeDim {
    w: usize,
    h: usize,
    lane: usize,
}

// Grouping traversal context to reduce arguments
struct TraversalContext<'a> {
    nodes_map: &'a HashMap<String, &'a crate::ir::FlowNode>,
    flowchart: &'a Flowchart,
    visited: &'a mut HashMap<String, usize>,
    node_dims: &'a mut HashMap<String, NodeDim>,
    lanes_width: &'a mut HashMap<usize, usize>,
    padding: usize,
}

pub fn render_flowchart(flowchart: &Flowchart) -> Result<String> {
    let padding = 3;
    let mut renderer = AsciiRenderer::new(2000, 5000, padding);

    let nodes_map: HashMap<String, &crate::ir::FlowNode> =
        flowchart.nodes.iter().map(|n| (n.id.clone(), n)).collect();

    let mut node_dims: HashMap<String, NodeDim> = HashMap::new();
    let mut lanes_width: HashMap<usize, usize> = HashMap::new();

    // 1. Assign Lanes via Recursive Traversal
    let mut visited: HashMap<String, usize> = HashMap::new();
    let start_node = flowchart.nodes.first().unwrap();

    let mut ctx = TraversalContext {
        nodes_map: &nodes_map,
        flowchart,
        visited: &mut visited,
        node_dims: &mut node_dims,
        lanes_width: &mut lanes_width,
        padding,
    };

    traverse_assign_lanes(&start_node.id, 0, &mut ctx);

    // Ensure all lane widths are even to guarantee perfect center alignment for any node width
    for lw in ctx.lanes_width.values_mut() {
        if *lw % 2 != 0 {
            *lw += 1;
        }
    }

    // 2. Calculate Lane X Offsets
    let mut lane_x: HashMap<usize, usize> = HashMap::new();
    let mut current_x = 20;
    let max_lane = *ctx.lanes_width.keys().max().unwrap_or(&0);

    for l in 0..=max_lane {
        lane_x.insert(l, current_x);
        let w = ctx.lanes_width.get(&l).unwrap_or(&0);
        current_x += w + 12;
    }

    // 3. Place Nodes (Y coordinates)
    let mut node_pos: HashMap<String, (usize, usize)> = HashMap::new();
    let mut lane_y: HashMap<usize, usize> = HashMap::new();
    let mut processed_nodes: HashMap<String, (usize, usize)> = HashMap::new();
    let mut pending_nodes: Vec<String> = flowchart.nodes.iter().map(|n| n.id.clone()).collect();

    // Initialize Start
    if let Some(dim) = ctx.node_dims.get(&start_node.id) {
        let lx = *lane_x.get(&dim.lane).unwrap();
        let lw = *ctx.lanes_width.get(&dim.lane).unwrap();
        let actual_x = lx + (lw - dim.w).div_ceil(2);
        node_pos.insert(start_node.id.clone(), (actual_x, 2));
        processed_nodes.insert(start_node.id.clone(), (actual_x, 2 + dim.h));
        *lane_y.entry(dim.lane).or_insert(0) = 2 + dim.h;
        pending_nodes.retain(|id| id != &start_node.id);
    }

    let mut progress = true;
    while progress && !pending_nodes.is_empty() {
        progress = false;
        let mut next_batch = Vec::new();

        for id in &pending_nodes {
            let parents: Vec<&crate::ir::FlowEdge> =
                flowchart.edges.iter().filter(|e| e.to == *id).collect();

            let ready_parents: Vec<_> = parents
                .iter()
                .filter(|p| {
                    if processed_nodes.contains_key(&p.from) {
                        return true;
                    }
                    // Back-edge check using ID
                    let p_id_num: usize = p.from.strip_prefix("node_").unwrap().parse().unwrap();
                    let c_id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();
                    let is_exit = flowchart
                        .nodes
                        .iter()
                        .find(|n| n.id == *id)
                        .map(|n| n.text.contains("Exit"))
                        .unwrap_or(false);

                    if p_id_num > c_id_num && !is_exit {
                        return true; // Pretend it's ready (ignore constraint)
                    }
                    false
                })
                .collect();

            let total_forward_parents = parents
                .iter()
                .filter(|p| {
                    let p_id_num: usize = p.from.strip_prefix("node_").unwrap().parse().unwrap();
                    let c_id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();
                    let is_exit = flowchart
                        .nodes
                        .iter()
                        .find(|n| n.id == *id)
                        .map(|n| n.text.contains("Exit"))
                        .unwrap_or(false);
                    p_id_num <= c_id_num || is_exit
                })
                .count();

            let ready_forward_count = ready_parents
                .iter()
                .filter(|p| {
                    let p_id_num: usize = p.from.strip_prefix("node_").unwrap().parse().unwrap();
                    let c_id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();
                    let is_exit = flowchart
                        .nodes
                        .iter()
                        .find(|n| n.id == *id)
                        .map(|n| n.text.contains("Exit"))
                        .unwrap_or(false);
                    p_id_num <= c_id_num || is_exit
                })
                .count();

            if total_forward_parents > 0 && ready_forward_count == total_forward_parents {
                let dim = ctx.node_dims.get(id).unwrap();
                let mut max_y = *lane_y.get(&dim.lane).unwrap_or(&0);
                let current_node = nodes_map.get(id).unwrap();

                for p in ready_parents {
                    let parent_node = nodes_map.get(&p.from).unwrap();
                    let node_is_merge = current_node.text == "Merge";
                    let parent_is_merge = parent_node.text == "Merge";

                    let gap = if node_is_merge {
                        2
                    } else if parent_is_merge {
                        3
                    } else {
                        5
                    };

                    if let Some((_, py)) = processed_nodes.get(&p.from)
                        && *py + gap > max_y
                    {
                        max_y = *py + gap;
                    }
                }

                *lane_y.entry(dim.lane).or_insert(0) = max_y + dim.h;

                let lx = *lane_x.get(&dim.lane).unwrap();
                let lw = *ctx.lanes_width.get(&dim.lane).unwrap();
                let actual_x = lx + (lw - dim.w).div_ceil(2);

                node_pos.insert(id.clone(), (actual_x, max_y));
                processed_nodes.insert(id.clone(), (actual_x, max_y + dim.h));
                next_batch.push(id.clone());
                progress = true;
            }
        }
        for id in next_batch {
            pending_nodes.retain(|x| x != &id);
        }
    }

    // 4. Draw Nodes
    for (id, (x, y)) in &node_pos {
        let node = nodes_map.get(id).unwrap();
        let dim = ctx.node_dims.get(id).unwrap();

        if node.text.starts_with("Merge") {
            continue;
        }

        match node.node_type {
            NodeType::Terminator => renderer.draw_terminator(*x, *y, dim.w, dim.h, &node.text),
            NodeType::Decision => renderer.draw_rhombus(*x, *y, dim.w, dim.h, &node.text),
            NodeType::InputOutput => renderer.draw_parallelogram(*x, *y, dim.w, dim.h, &node.text),
            NodeType::SwitchCase => renderer.draw_switch_case(*x, *y, dim.w, dim.h, &node.text),
            _ => renderer.draw_box(*x, *y, dim.w, dim.h, &node.text),
        }
    }

    // 5. Draw Edges
    for edge in &flowchart.edges {
        if edge.label.as_deref() == Some("Dependency") {
            continue;
        }

        if !node_pos.contains_key(&edge.from) || !node_pos.contains_key(&edge.to) {
            continue;
        }

        let n1 = nodes_map.get(&edge.from).unwrap();
        let n2 = nodes_map.get(&edge.to).unwrap();

        let (x1, y1) = *node_pos.get(&edge.from).unwrap();
        let dim1 = ctx.node_dims.get(&edge.from).unwrap();
        let (x2, y2) = *node_pos.get(&edge.to).unwrap();
        let dim2 = ctx.node_dims.get(&edge.to).unwrap();

        let is_decision =
            n1.node_type == NodeType::Decision || n1.node_type == NodeType::SwitchCase;
        let mut label = edge.label.as_deref();
        let is_target_merge = n2.text.starts_with("Merge");

        // Loop Back Detection using IDs
        let n1_id_num: usize = edge.from.strip_prefix("node_").unwrap().parse().unwrap();
        let n2_id_num: usize = edge.to.strip_prefix("node_").unwrap().parse().unwrap();

        let is_loop_head = is_decision
            && flowchart.edges.iter().any(|e| {
                if e.to == edge.from
                    && let Some(f_id_part) = e.from.strip_prefix("node_")
                    && let Ok(f_id_num) = f_id_part.parse::<usize>()
                {
                    return f_id_num > n1_id_num;
                }
                false
            });

        // Ignore ID back-edge logic if target is explicitly an Exit
        let is_back_edge = n1_id_num > n2_id_num && !n2.text.contains("Exit");

        if is_back_edge {
            // Loop Back
            let is_while_for = n2.node_type == NodeType::Decision
                || (n2.text == "Merge"
                    && flowchart.edges.iter().any(|e| {
                        e.from == edge.to
                            && nodes_map
                                .get(&e.to)
                                .is_some_and(|m| m.node_type == NodeType::Decision)
                    }));
            let use_right_side = is_while_for || dim1.lane > 0;

            // Calculate depth_inside to stagger loop lines
            let mut depth_inside = 0;
            for other_edge in &flowchart.edges {
                if let Some(m1_id_part) = other_edge.from.strip_prefix("node_")
                    && let Some(m2_id_part) = other_edge.to.strip_prefix("node_")
                    && let Ok(m1_id_num_inner) = m1_id_part.parse::<usize>()
                    && let Ok(m2_id_num_inner) = m2_id_part.parse::<usize>()
                    && m1_id_num_inner > m2_id_num_inner
                {
                    // If other loop is strictly inside this loop
                    if (m2_id_num_inner > n2_id_num && m1_id_num_inner < n1_id_num)
                        || (m2_id_num_inner == n2_id_num
                            && m1_id_num_inner == n1_id_num
                            && other_edge.from < edge.from)
                    {
                        depth_inside += 1;
                    }
                }
            }

            let is_bottom_exit = n1.node_type != NodeType::Decision;

            let join_y = if is_target_merge {
                y2
            } else {
                y2.saturating_sub(3)
            };

            if use_right_side {
                // Calculate max_x in range to avoid crossing nodes
                let mut max_x = x1 + dim1.w;
                for (id, (px, _)) in &node_pos {
                    let id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();
                    if id_num >= n2_id_num && id_num <= n1_id_num {
                        let dim = ctx.node_dims.get(id).unwrap();
                        max_x = max_x.max(px + dim.w);
                    }
                }

                let sx = if is_bottom_exit {
                    x1 + dim1.w / 2
                } else {
                    x1 + dim1.w - 1
                };
                let sy = if is_bottom_exit {
                    y1 + dim1.h
                } else {
                    y1 + dim1.h / 2
                };
                let tx = x2 + dim2.w / 2;
                let ty = y2;
                let margin_x = max_x + 4 + depth_inside * 4;
                renderer.draw_loop_back_right(
                    sx,
                    sy,
                    tx,
                    ty,
                    margin_x,
                    label,
                    is_bottom_exit,
                    !is_target_merge,
                    join_y,
                );
            } else {
                let sx = if is_bottom_exit { x1 + dim1.w / 2 } else { x1 };
                let sy = if is_bottom_exit {
                    y1 + dim1.h
                } else {
                    y1 + dim1.h / 2
                };
                let tx = x2 + dim2.w / 2;
                let ty = y2;
                let margin_x = 10u64.saturating_sub(depth_inside as u64 * 4) as usize;
                renderer.draw_loop_back(
                    sx,
                    sy,
                    tx,
                    ty,
                    margin_x.max(1),
                    label,
                    is_bottom_exit,
                    !is_target_merge,
                    join_y,
                );
            }
        } else if is_loop_head {
            if label == Some("Verdadero") {
                let sx = x1 + dim1.w - 1;
                let sy = y1 + dim1.h / 2;
                let ty = y2;
                renderer.draw_elbow_right_down(
                    sx,
                    sy,
                    x2 + dim2.w / 2,
                    ty,
                    Some("Verdadero"),
                    !is_target_merge,
                );
            } else if label == Some("Falso") {
                let sx = x1 + dim1.w / 2;
                let sy = y1 + dim1.h;
                let ty = y2;
                renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, Some("Falso"));
            } else {
                let sx = x1 + dim1.w / 2;
                let sy = y1 + dim1.h;
                let ty = y2;
                renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, label);
            }
        } else if is_decision && label == Some("Verdadero") {
            let mut sx = x1 + dim1.w - 1;
            let mut display_label = "Verdadero";

            if n1.node_type == NodeType::SwitchCase {
                sx += 2; // Leave one space after the ')'
                display_label = "Verdadero"; // The shift in sx already provides appropriate padding
            }

            let sy = y1 + dim1.h / 2;
            let _tx = x2 + dim2.w / 2; // Unused but part of logic intent
            let ty = y2;
            // Draw elbow right-down
            renderer.draw_elbow_right_down(
                sx,
                sy,
                x2 + dim2.w / 2,
                ty,
                Some(display_label),
                !is_target_merge,
            );
        } else if is_decision && label == Some("Falso") {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let _tx = x2 + dim2.w / 2;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, Some("Falso"));
        } else if label == Some("__Break__") {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let tx = x2 + dim2.w / 2;
            let ty = y2;

            let max_l = *ctx.lanes_width.keys().max().unwrap_or(&0);
            let collector_x =
                lane_x.get(&max_l).unwrap() + ctx.lanes_width.get(&max_l).unwrap() + 8;

            // Bajar un poco
            let y_turn = sy + 2;
            renderer.draw_vertical_line(sx, sy, y_turn, false, None);

            // Línea a la derecha hacia el colector
            renderer.set_char_force(sx, y_turn, '└', crate::render::CellType::LineCorner);
            for x in (sx + 1)..collector_x {
                renderer.set_char_force(x, y_turn, '─', crate::render::CellType::LineH);
            }

            // Bajar hasta la coordenada Y del colector final (ty)
            renderer.set_char_force(
                collector_x,
                y_turn,
                '┐',
                crate::render::CellType::LineCorner,
            );
            renderer.draw_vertical_line(collector_x, y_turn + 1, ty, false, None);

            // Retroceder a la izquierda para unirse a la espina
            renderer.set_char_force(collector_x, ty, '┘', crate::render::CellType::LineCorner);
            for x in (tx + 2)..collector_x {
                renderer.set_char_force(x, ty, '─', crate::render::CellType::LineH);
            }
            renderer.set_char_force(tx + 1, ty, '🭮', crate::render::CellType::Arrow);
            renderer.set_char_force(tx, ty, '├', crate::render::CellType::LineCorner);
        } else if is_target_merge {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let tx = x2 + dim2.w / 2;
            let ty = y2;
            renderer.draw_merge_left(sx, sy, tx, ty);
        } else {
            // Implicit "Falso" for loop exit
            if is_decision && label.is_none() && n1.text.contains("opcion != 2") {
                label = Some("Falso");
            }
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let _tx = x2 + dim2.w / 2;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, label);
        }
    }

    Ok(renderer.render())
}

#[allow(clippy::too_many_arguments)] // Preserving logic structure
fn traverse_assign_lanes(id: &str, lane: usize, ctx: &mut TraversalContext) -> usize {
    if let Some(&l) = ctx.visited.get(id) {
        return l;
    }
    ctx.visited.insert(id.to_string(), lane);

    let node = ctx.nodes_map.get(id).unwrap();
    let (w, h) = calculate_node_size(node, ctx.padding);
    ctx.node_dims.insert(id.to_string(), NodeDim { w, h, lane });

    let lw = ctx.lanes_width.entry(lane).or_insert(0);
    if w > *lw {
        *lw = w;
    }

    let mut max_lane = lane;

    let mut edges: Vec<&crate::ir::FlowEdge> = ctx
        .flowchart
        .edges
        .iter()
        .filter(|e| e.from == *id)
        .collect();

    let n_id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();
    let is_loop_head = ctx.flowchart.edges.iter().any(|e| {
        if e.to == id
            && let Some(f_id_part) = e.from.strip_prefix("node_")
            && let Ok(f_id_num) = f_id_part.parse::<usize>()
        {
            return f_id_num > n_id_num;
        }
        false
    });

    edges.sort_by(|a, b| {
        let la = a.label.as_deref().unwrap_or("");
        let lb = b.label.as_deref().unwrap_or("");

        if is_loop_head {
            if la == "Falso" {
                return std::cmp::Ordering::Less;
            }
            if lb == "Falso" {
                return std::cmp::Ordering::Greater;
            }
        }

        // For all decisions: Falso goes down (stay in lane), Verdadero goes right (new lane)
        if la == "Falso" {
            std::cmp::Ordering::Less
        } else if lb == "Falso" || la == "Verdadero" {
            std::cmp::Ordering::Greater
        } else if lb == "Verdadero" {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });

    for edge in edges {
        // ID-based back-edge detection for traversal
        let t_id_num: usize = edge.to.strip_prefix("node_").unwrap().parse().unwrap();
        let target_node = ctx.nodes_map.get(&edge.to).unwrap();
        let is_exit = target_node.text.contains("Exit");

        // Prevent breaks or fallthroughs from placing the Exit node in a lateral lane
        if is_exit && edge.label.as_deref() != Some("Falso") {
            continue;
        }

        if t_id_num < n_id_num && !is_exit {
            continue;
        }

        let next_lane_start = if node.text == "Merge" {
            lane
        } else if is_loop_head {
            if edge.label.as_deref() == Some("Falso") || edge.label.is_none() {
                lane
            } else {
                max_lane + 1
            }
        } else if edge.label.as_deref() == Some("Verdadero") {
            // For do-while back-edges, we stay in lane 0 if we can?
            // Actually, if it's a back-edge, traverse_assign_lanes skips it anyway.
            max_lane + 1
        } else {
            lane
        };

        let m = traverse_assign_lanes(&edge.to, next_lane_start, ctx);
        if m > max_lane {
            max_lane = m;
        }
    }

    max_lane
}

#[allow(clippy::manual_is_multiple_of)]
fn calculate_node_size(node: &crate::ir::FlowNode, padding: usize) -> (usize, usize) {
    if node.text.starts_with("Merge") {
        return (0, 0);
    }
    let lines: Vec<&str> = node.text.split('\n').collect();
    let content_h = lines.len();
    let content_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    match node.node_type {
        NodeType::Decision => {
            // Rhombus sizing
            // Inner width available = W - 2 (corners take 1 each)
            // W = 4*half_h.
            // Inner W = 4*half_h - 2.
            // Requirement: 4*half_h - 2 >= content_w + padding (2).
            // 4*half_h >= content_w + 4.
            // half_h >= (content_w + 4)/4.
            let min_half_h = (content_w + 4).div_ceil(4);
            let min_h_for_h = content_h + 4;
            let mut h = std::cmp::max(2 * min_half_h + 1, min_h_for_h);
            if h % 2 == 0 {
                h += 1;
            }
            let half_h = h / 2;
            let w = 4 * half_h;
            (w, h)
        }
        NodeType::Terminator => {
            let h = content_h;
            let w = content_w + 6; // Space for "(  ...  )"
            (w, h)
        }
        NodeType::SwitchCase => {
            let h = 1;
            let w = content_w + 4; // Extra space for "(  )"
            (w, h)
        }
        NodeType::InputOutput => {
            let h = content_h + 2;
            let offset = h.saturating_sub(1);
            let w = content_w + offset + padding * 2 + 2;
            // Force even width for perfect center alignment
            let mut final_w = w;
            if final_w % 2 != 0 {
                final_w += 1;
            }
            (final_w, h)
        }
        _ => {
            let h = content_h + 2;
            let w = content_w + padding * 2 + 2;
            // Force even width
            let mut final_w = w;
            if final_w % 2 != 0 {
                final_w += 1;
            }
            (final_w, h)
        }
    }
}
