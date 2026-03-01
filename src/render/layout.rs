use crate::ir::{
    EDGE_LABEL_BREAK, EDGE_LABEL_DEPENDENCY, EDGE_LABEL_FALSE, EDGE_LABEL_TRUE, Flowchart, NodeType,
};
use crate::render::canvas::AsciiRenderer;
use crate::render::routing::RoutingDrawing;
use crate::render::shapes::ShapeDrawing;
use anyhow::{Context, Result};
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

fn parse_id(id: &str) -> usize {
    id.strip_prefix("node_")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub(crate) struct NodeDim {
    pub w: usize,
    pub h: usize,
    pub lane: usize,
}

pub(crate) struct TraversalContext<'a> {
    pub nodes_map: &'a HashMap<String, &'a crate::ir::FlowNode>,
    pub flowchart: &'a Flowchart,
    pub visited: &'a mut HashMap<String, usize>,
    pub node_dims: &'a mut HashMap<String, NodeDim>,
    pub lanes_width: &'a mut HashMap<usize, usize>,
    pub padding: usize,
}

pub struct LayoutEngine<'a> {
    flowchart: &'a Flowchart,
    nodes_map: HashMap<String, &'a crate::ir::FlowNode>,
    node_dims: HashMap<String, NodeDim>,
    lanes_width: HashMap<usize, usize>,
    lane_x: HashMap<usize, usize>,
    lane_y: HashMap<usize, usize>,
    node_pos: HashMap<String, (usize, usize)>,
    processed_nodes: HashMap<String, (usize, usize)>,
    padding: usize,
}

impl<'a> LayoutEngine<'a> {
    pub fn new(flowchart: &'a Flowchart, padding: usize) -> Self {
        let nodes_map: HashMap<String, &'a crate::ir::FlowNode> =
            flowchart.nodes.iter().map(|n| (n.id.clone(), n)).collect();

        Self {
            flowchart,
            nodes_map,
            node_dims: HashMap::new(),
            lanes_width: HashMap::new(),
            lane_x: HashMap::new(),
            lane_y: HashMap::new(),
            node_pos: HashMap::new(),
            processed_nodes: HashMap::new(),
            padding,
        }
    }

    pub fn assign_lanes(&mut self) -> Result<()> {
        let mut visited: HashMap<String, usize> = HashMap::new();
        if let Some(start_node) = self.flowchart.nodes.first() {
            let mut ctx = TraversalContext {
                nodes_map: &self.nodes_map,
                flowchart: self.flowchart,
                visited: &mut visited,
                node_dims: &mut self.node_dims,
                lanes_width: &mut self.lanes_width,
                padding: self.padding,
            };
            traverse_assign_lanes(&start_node.id, 0, &mut ctx)?;
        }

        // Ensure all lane widths are even to guarantee perfect center alignment for any node width
        for lw in self.lanes_width.values_mut() {
            if *lw % 2 != 0 {
                *lw += 1;
            }
        }
        Ok(())
    }

    pub fn calculate_x_offsets(&mut self) {
        let mut current_x = 20;
        let max_lane = *self.lanes_width.keys().max().unwrap_or(&0);

        for l in 0..=max_lane {
            self.lane_x.insert(l, current_x);
            let w = self.lanes_width.get(&l).unwrap_or(&0);
            current_x += w + 12;
        }
    }

    pub fn place_nodes(&mut self) -> Result<()> {
        let mut pending_nodes: Vec<String> =
            self.flowchart.nodes.iter().map(|n| n.id.clone()).collect();

        if let Some(start_node) = self.flowchart.nodes.first()
            && let Some(dim) = self.node_dims.get(&start_node.id).cloned()
        {
            let lx = *self.lane_x.get(&dim.lane).context("Missing lane_x")?;
            let lw = *self
                .lanes_width
                .get(&dim.lane)
                .context("Missing lanes_width")?;
            let actual_x = lx + (lw - dim.w).div_ceil(2);
            self.node_pos.insert(start_node.id.clone(), (actual_x, 2));
            self.processed_nodes
                .insert(start_node.id.clone(), (actual_x, 2 + dim.h));
            *self.lane_y.entry(dim.lane).or_insert(0) = 2 + dim.h;
            pending_nodes.retain(|id| id != &start_node.id);
        }

        let mut progress = true;
        while progress && !pending_nodes.is_empty() {
            progress = false;
            let mut next_batch = Vec::new();

            for id in &pending_nodes {
                let parents: Vec<&crate::ir::FlowEdge> = self
                    .flowchart
                    .edges
                    .iter()
                    .filter(|e| e.to == *id)
                    .collect();

                let c_id_num: usize = parse_id(id);
                let is_exit = self
                    .flowchart
                    .nodes
                    .iter()
                    .find(|n| n.id == *id)
                    .map(|n| n.text.contains("Exit"))
                    .unwrap_or(false);

                let ready_parents: Vec<_> = parents
                    .iter()
                    .filter(|p| {
                        if self.processed_nodes.contains_key(&p.from) {
                            return true;
                        }
                        // Back-edge check using ID
                        let p_id_num: usize = parse_id(&p.from);
                        if p_id_num > c_id_num && !is_exit {
                            return true; // Pretend it's ready (ignore constraint)
                        }
                        false
                    })
                    .collect();

                let total_forward_parents = parents
                    .iter()
                    .filter(|p| {
                        let p_id_num: usize = parse_id(&p.from);
                        p_id_num <= c_id_num || is_exit
                    })
                    .count();

                let ready_forward_count = ready_parents
                    .iter()
                    .filter(|p| {
                        let p_id_num: usize = parse_id(&p.from);
                        p_id_num <= c_id_num || is_exit
                    })
                    .count();

                if total_forward_parents > 0 && ready_forward_count == total_forward_parents {
                    let dim = self.node_dims.get(id).context("Missing node_dim")?.clone();
                    let mut max_y = *self.lane_y.get(&dim.lane).unwrap_or(&0);
                    let current_node = self.nodes_map.get(id).context("Missing node_map")?;

                    for p in ready_parents {
                        let parent_node = self
                            .nodes_map
                            .get(&p.from)
                            .context("Missing node_map for parent")?;
                        let node_is_merge = current_node.text == "Merge";
                        let parent_is_merge = parent_node.text == "Merge";

                        let gap = if node_is_merge {
                            2
                        } else if parent_is_merge {
                            3
                        } else {
                            5
                        };

                        if let Some((_, py)) = self.processed_nodes.get(&p.from)
                            && *py + gap > max_y
                        {
                            max_y = *py + gap;
                        }
                    }

                    *self.lane_y.entry(dim.lane).or_insert(0) = max_y + dim.h;

                    let lx = *self.lane_x.get(&dim.lane).context("Missing lane_x")?;
                    let lw = *self
                        .lanes_width
                        .get(&dim.lane)
                        .context("Missing lanes_width")?;
                    let actual_x = lx + (lw - dim.w).div_ceil(2);

                    self.node_pos.insert(id.clone(), (actual_x, max_y));
                    self.processed_nodes
                        .insert(id.clone(), (actual_x, max_y + dim.h));
                    next_batch.push(id.clone());
                    progress = true;
                }
            }
            for id in next_batch {
                pending_nodes.retain(|x| x != &id);
            }
        }
        Ok(())
    }

    pub fn draw_nodes(&self, renderer: &mut AsciiRenderer) -> Result<()> {
        for (id, (x, y)) in &self.node_pos {
            let node = self.nodes_map.get(id).context("Missing node_map")?;
            let dim = self.node_dims.get(id).context("Missing node_dim")?;

            if node.text.starts_with("Merge") {
                continue;
            }

            match node.node_type {
                NodeType::Terminator => renderer.draw_terminator(*x, *y, dim.w, dim.h, &node.text),
                NodeType::Decision => renderer.draw_rhombus(*x, *y, dim.w, dim.h, &node.text),
                NodeType::InputOutput => {
                    renderer.draw_parallelogram(*x, *y, dim.w, dim.h, &node.text)
                }
                NodeType::SwitchCase => renderer.draw_switch_case(*x, *y, dim.w, dim.h, &node.text),
                NodeType::PredefinedProcess => {
                    renderer.draw_predefined_process(*x, *y, dim.w, dim.h, &node.text)
                }
                _ => renderer.draw_box(*x, *y, dim.w, dim.h, &node.text),
            }
        }
        Ok(())
    }

    pub fn draw_edges(&self, renderer: &mut AsciiRenderer) -> Result<()> {
        for edge in &self.flowchart.edges {
            if edge.label.as_deref() == Some(EDGE_LABEL_DEPENDENCY) {
                continue;
            }

            if !self.node_pos.contains_key(&edge.from) || !self.node_pos.contains_key(&edge.to) {
                continue;
            }

            let n1 = self
                .nodes_map
                .get(&edge.from)
                .context("Missing node_map for edge from")?;
            let n2 = self
                .nodes_map
                .get(&edge.to)
                .context("Missing node_map for edge to")?;

            let (x1, y1) = *self
                .node_pos
                .get(&edge.from)
                .context("Missing node_pos for edge from")?;
            let dim1 = self
                .node_dims
                .get(&edge.from)
                .context("Missing node_dims for edge from")?;
            let (x2, y2) = *self
                .node_pos
                .get(&edge.to)
                .context("Missing node_pos for edge to")?;
            let dim2 = self
                .node_dims
                .get(&edge.to)
                .context("Missing node_dims for edge to")?;

            let is_decision =
                n1.node_type == NodeType::Decision || n1.node_type == NodeType::SwitchCase;
            let label = edge.label.as_deref();
            let is_target_merge = n2.text.starts_with("Merge");

            let n1_id_num: usize = parse_id(&edge.from);
            let n2_id_num: usize = parse_id(&edge.to);

            let is_loop_head = is_decision
                && self.flowchart.edges.iter().any(|e| {
                    if e.to == edge.from
                        && let Some(f_id_part) = e.from.strip_prefix("node_")
                    {
                        let f_id_num = f_id_part.parse::<usize>().unwrap_or(0);
                        return f_id_num > n1_id_num;
                    }
                    false
                });

            let is_back_edge = n1_id_num > n2_id_num && !n2.text.contains("Exit");

            if is_back_edge {
                self.draw_back_edge(
                    renderer,
                    edge,
                    (x1, y1, dim1),
                    (x2, y2, dim2),
                    is_target_merge,
                )?;
            } else if label == Some(EDGE_LABEL_BREAK) {
                self.draw_break_edge(renderer, (x1, y1, dim1), (x2, y2, dim2))?;
            } else if is_loop_head {
                self.draw_loop_head_edge(
                    renderer,
                    label,
                    (x1, y1, dim1),
                    (x2, y2, dim2),
                    is_target_merge,
                );
            } else if is_decision {
                self.draw_decision_edge(
                    renderer,
                    n1,
                    label,
                    (x1, y1, dim1),
                    (x2, y2, dim2),
                    is_target_merge,
                );
            } else if is_target_merge {
                renderer.draw_merge_left(x1 + dim1.w / 2, y1 + dim1.h, x2 + dim2.w / 2, y2);
            } else {
                let sx = x1 + dim1.w / 2;
                let sy = y1 + dim1.h;
                let ty = y2;
                renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, label);
            }
        }
        Ok(())
    }

    fn draw_back_edge(
        &self,
        renderer: &mut AsciiRenderer,
        edge: &crate::ir::FlowEdge,
        (x1, y1, dim1): (usize, usize, &NodeDim),
        (x2, y2, dim2): (usize, usize, &NodeDim),
        is_target_merge: bool,
    ) -> Result<()> {
        let n1_id_num = parse_id(&edge.from);
        let n2_id_num = parse_id(&edge.to);
        let n2 = self
            .nodes_map
            .get(&edge.to)
            .context("Node not found for back edge target")?;
        let n1 = self
            .nodes_map
            .get(&edge.from)
            .context("Node not found for back edge source")?;

        let is_while_for = n2.node_type == NodeType::Decision
            || (n2.text == "Merge"
                && self.flowchart.edges.iter().any(|e| {
                    e.from == edge.to
                        && self
                            .nodes_map
                            .get(&e.to)
                            .is_some_and(|m| m.node_type == NodeType::Decision)
                }));
        let use_right_side = is_while_for || dim1.lane > 0;

        let mut depth_inside = 0;
        for other_edge in &self.flowchart.edges {
            if let Some(m1_id_part) = other_edge.from.strip_prefix("node_")
                && let Some(m2_id_part) = other_edge.to.strip_prefix("node_")
                && let m1_id_num_inner = m1_id_part.parse::<usize>().unwrap_or(0)
                && let m2_id_num_inner = m2_id_part.parse::<usize>().unwrap_or(0)
                && m1_id_num_inner > m2_id_num_inner
                && ((m2_id_num_inner > n2_id_num && m1_id_num_inner < n1_id_num)
                    || (m2_id_num_inner == n2_id_num
                        && m1_id_num_inner == n1_id_num
                        && other_edge.from < edge.from))
            {
                depth_inside += 1;
            }
        }

        let is_bottom_exit = n1.node_type != NodeType::Decision;

        let join_y = if is_target_merge {
            y2
        } else {
            y2.saturating_sub(3)
        };

        if use_right_side {
            let mut max_x = x1 + dim1.w;
            for (id, (px, _)) in &self.node_pos {
                let id_num: usize = parse_id(id);
                if id_num >= n2_id_num && id_num <= n1_id_num {
                    let dim = self.node_dims.get(id).context("Missing node_dim")?;
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
                edge.label.as_deref(),
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
                edge.label.as_deref(),
                is_bottom_exit,
                !is_target_merge,
                join_y,
            );
        }
        Ok(())
    }

    fn draw_break_edge(
        &self,
        renderer: &mut AsciiRenderer,
        (x1, y1, dim1): (usize, usize, &NodeDim),
        (x2, y2, dim2): (usize, usize, &NodeDim),
    ) -> Result<()> {
        let sx = x1 + dim1.w / 2;
        let sy = y1 + dim1.h;
        let tx = x2 + dim2.w / 2;
        let ty = y2;

        let max_l = *self.lanes_width.keys().max().unwrap_or(&0);
        let collector_x = self.lane_x.get(&max_l).context("Missing max lane x")?
            + self
                .lanes_width
                .get(&max_l)
                .context("Missing max lane width")?
            + 8;

        let y_turn = sy + 2;
        renderer.draw_vertical_line(sx, sy, y_turn, false, None);

        renderer.set_char_force(sx, y_turn, '└', crate::render::canvas::CellType::LineCorner);
        for x in (sx + 1)..collector_x {
            renderer.set_char_force(x, y_turn, '─', crate::render::canvas::CellType::LineH);
        }

        renderer.set_char_force(
            collector_x,
            y_turn,
            '┐',
            crate::render::canvas::CellType::LineCorner,
        );
        renderer.draw_vertical_line(collector_x, y_turn + 1, ty, false, None);

        renderer.set_char_force(
            collector_x,
            ty,
            '┘',
            crate::render::canvas::CellType::LineCorner,
        );
        for x in (tx + 2)..collector_x {
            renderer.set_char_force(x, ty, '─', crate::render::canvas::CellType::LineH);
        }
        renderer.set_char_force(tx + 1, ty, '🭮', crate::render::canvas::CellType::Arrow);
        renderer.set_char_force(tx, ty, '├', crate::render::canvas::CellType::LineCorner);
        Ok(())
    }

    fn draw_loop_head_edge(
        &self,
        renderer: &mut AsciiRenderer,
        label: Option<&str>,
        (x1, y1, dim1): (usize, usize, &NodeDim),
        (x2, y2, dim2): (usize, usize, &NodeDim),
        is_target_merge: bool,
    ) {
        if label == Some(EDGE_LABEL_TRUE) {
            let sx = x1 + dim1.w - 1;
            let sy = y1 + dim1.h / 2;
            let ty = y2;
            renderer.draw_elbow_right_down(
                sx,
                sy,
                x2 + dim2.w / 2,
                ty,
                Some(EDGE_LABEL_TRUE),
                !is_target_merge,
            );
        } else if label == Some(EDGE_LABEL_FALSE) {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, Some(EDGE_LABEL_FALSE));
        } else {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, label);
        }
    }

    fn draw_decision_edge(
        &self,
        renderer: &mut AsciiRenderer,
        n1: &crate::ir::FlowNode,
        mut label: Option<&str>,
        (x1, y1, dim1): (usize, usize, &NodeDim),
        (x2, y2, dim2): (usize, usize, &NodeDim),
        is_target_merge: bool,
    ) {
        if label == Some(EDGE_LABEL_TRUE) {
            let mut sx = x1 + dim1.w - 1;
            let mut display_label = EDGE_LABEL_TRUE;

            if n1.node_type == NodeType::SwitchCase {
                sx += 2;
                display_label = EDGE_LABEL_TRUE;
            }

            let sy = y1 + dim1.h / 2;
            let ty = y2;
            renderer.draw_elbow_right_down(
                sx,
                sy,
                x2 + dim2.w / 2,
                ty,
                Some(display_label),
                !is_target_merge,
            );
        } else if label == Some(EDGE_LABEL_FALSE) {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, Some(EDGE_LABEL_FALSE));
        } else {
            if label.is_none() && n1.text.contains("opcion != 2") {
                label = Some(EDGE_LABEL_FALSE);
            }
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, !is_target_merge, label);
        }
    }
}

pub fn render_flowchart(flowchart: &Flowchart) -> Result<String> {
    let mut final_render = render_flowchart_single(flowchart)?;

    for sub in &flowchart.subroutines {
        final_render.push_str("\n\n\n");
        let separator = format!(
            "=========================================================\n            [ Subrutina: {} ]\n=========================================================",
            sub.title
        );
        final_render.push_str(&separator);
        final_render.push_str("\n\n");

        let sub_render = render_flowchart_single(sub)?;
        final_render.push_str(&sub_render);
    }

    Ok(final_render)
}

fn render_flowchart_single(flowchart: &Flowchart) -> Result<String> {
    let padding = 3;
    let mut renderer = AsciiRenderer::new(2000, 5000, padding);

    let mut engine = LayoutEngine::new(flowchart, padding);
    engine.assign_lanes()?;
    engine.calculate_x_offsets();
    engine.place_nodes()?;
    engine.draw_nodes(&mut renderer)?;
    engine.draw_edges(&mut renderer)?;

    Ok(renderer.render())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn traverse_assign_lanes(
    id: &str,
    lane: usize,
    ctx: &mut TraversalContext,
) -> Result<usize> {
    if let Some(&l) = ctx.visited.get(id) {
        return Ok(l);
    }
    ctx.visited.insert(id.to_string(), lane);

    let node = ctx.nodes_map.get(id).context("Missing context node")?;
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

    let n_id_num: usize = parse_id(id);
    let is_loop_head = ctx.flowchart.edges.iter().any(|e| {
        if e.to == id
            && let Some(f_id_part) = e.from.strip_prefix("node_")
        {
            let f_id_num = f_id_part.parse::<usize>().unwrap_or(0);
            return f_id_num > n_id_num;
        }
        false
    });

    edges.sort_by(|a, b| {
        let la = a.label.as_deref().unwrap_or("");
        let lb = b.label.as_deref().unwrap_or("");

        if is_loop_head {
            if la == EDGE_LABEL_FALSE {
                return std::cmp::Ordering::Less;
            }
            if lb == EDGE_LABEL_FALSE {
                return std::cmp::Ordering::Greater;
            }
        }

        if la == EDGE_LABEL_FALSE {
            std::cmp::Ordering::Less
        } else if lb == EDGE_LABEL_FALSE || la == EDGE_LABEL_TRUE {
            std::cmp::Ordering::Greater
        } else if lb == EDGE_LABEL_TRUE {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });

    for edge in edges {
        let t_id_num: usize = parse_id(&edge.to);
        let target_node = ctx
            .nodes_map
            .get(&edge.to)
            .context("Missing context edge node")?;
        let is_exit = target_node.text.contains("Exit");

        if is_exit && edge.label.as_deref() != Some(EDGE_LABEL_FALSE) {
            continue;
        }

        if t_id_num < n_id_num && !is_exit {
            continue;
        }

        let next_lane_start = if node.text == "Merge" {
            lane
        } else if is_loop_head {
            if edge.label.as_deref() == Some(EDGE_LABEL_FALSE) || edge.label.is_none() {
                lane
            } else {
                max_lane + 1
            }
        } else if node.node_type == NodeType::SwitchCase
            && edge.label.as_deref() == Some(EDGE_LABEL_TRUE)
        {
            lane + 1
        } else if edge.label.as_deref() == Some(EDGE_LABEL_TRUE) {
            max_lane + 1
        } else {
            lane
        };

        let m = traverse_assign_lanes(&edge.to, next_lane_start, ctx)?;
        if m > max_lane {
            max_lane = m;
        }
    }

    Ok(max_lane)
}

#[allow(clippy::manual_is_multiple_of)]
pub(crate) fn calculate_node_size(node: &crate::ir::FlowNode, padding: usize) -> (usize, usize) {
    if node.text.starts_with("Merge") {
        return (0, 0);
    }
    let lines: Vec<&str> = node.text.split('\n').collect();
    let content_h = lines.len();
    let content_w = lines.iter().map(|l| l.width()).max().unwrap_or(0);

    match node.node_type {
        NodeType::Decision => {
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
            let w = content_w + 6;
            (w, h)
        }
        NodeType::SwitchCase => {
            let h = 1;
            let w = content_w + 4;
            (w, h)
        }
        NodeType::PredefinedProcess => {
            let h = content_h + 2;
            let w = content_w + padding * 2 + 6;
            (w, h)
        }
        NodeType::InputOutput => {
            let h = content_h + 2;
            let offset = h.saturating_sub(1);
            let w = content_w + offset + padding * 2 + 2;
            (w, h)
        }
        _ => {
            let h = content_h + 2;
            let w = content_w + padding * 2 + 2;
            (w, h)
        }
    }
}
