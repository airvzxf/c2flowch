use crate::ir::{Flowchart, NodeType};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellType {
    Empty,
    Node,
    LineH,
    LineV,
    LineCorner,
    Arrow,
    Text,
}

pub struct AsciiRenderer {
    canvas: Vec<Vec<char>>,
    occupancy: Vec<Vec<CellType>>,
    width: usize,
    height: usize,
    padding: usize, // Internal padding for nodes
}

impl AsciiRenderer {
    pub fn new(width: usize, height: usize, padding: usize) -> Self {
        Self {
            canvas: vec![vec![' '; width]; height],
            occupancy: vec![vec![CellType::Empty; width]; height],
            width,
            height,
            padding,
        }
    }

    fn set_char(&mut self, x: usize, y: usize, c: char, cell_type: CellType) {
        if x < self.width && y < self.height {
            self.canvas[y][x] = c;
            self.occupancy[y][x] = cell_type;
        }
    }

    fn set_char_force(&mut self, x: usize, y: usize, c: char, cell_type: CellType) {
        if x < self.width && y < self.height {
            self.canvas[y][x] = c;
            self.occupancy[y][x] = cell_type;
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        let lines: Vec<&str> = text.split('\n').collect();
        for (j, line) in lines.iter().enumerate() {
            for (i, c) in line.chars().enumerate() {
                self.set_char_force(x + i, y + j, c, CellType::Text);
            }
        }
    }

    pub fn draw_rounded_box(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        if w < 2 || h < 2 {
            return;
        }
        // Draw corners
        self.set_char_force(x, y, '╭', CellType::Node);
        self.set_char_force(x + w - 1, y, '╮', CellType::Node);
        self.set_char_force(x, y + h - 1, '╰', CellType::Node);
        self.set_char_force(x + w - 1, y + h - 1, '╯', CellType::Node);

        // Draw edges
        for i in 1..w - 1 {
            self.set_char_force(x + i, y, '─', CellType::Node);
            self.set_char_force(x + i, y + h - 1, '─', CellType::Node);
        }
        for j in 1..h - 1 {
            self.set_char_force(x, y + j, '│', CellType::Node);
            self.set_char_force(x + w - 1, y + j, '│', CellType::Node);
        }

        // Fill center
        for j in 1..h - 1 {
            for i in 1..w - 1 {
                self.set_char_force(x + i, y + j, ' ', CellType::Node);
            }
        }

        self.draw_centered_text(x, y, w, h, text);
    }

    pub fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        if w < 2 || h < 2 {
            return;
        }
        // Standard Box Drawing
        self.set_char_force(x, y, '┌', CellType::Node);
        self.set_char_force(x + w - 1, y, '┐', CellType::Node);
        self.set_char_force(x, y + h - 1, '└', CellType::Node);
        self.set_char_force(x + w - 1, y + h - 1, '┘', CellType::Node);

        for i in 1..w - 1 {
            self.set_char_force(x + i, y, '─', CellType::Node);
            self.set_char_force(x + i, y + h - 1, '─', CellType::Node);
        }
        for j in 1..h - 1 {
            self.set_char_force(x, y + j, '│', CellType::Node);
            self.set_char_force(x + w - 1, y + j, '│', CellType::Node);
        }

        for j in 1..h - 1 {
            for i in 1..w - 1 {
                self.set_char_force(x + i, y + j, ' ', CellType::Node);
            }
        }

        self.draw_content_text(x, y, w, h, text);
    }

    pub fn draw_parallelogram(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let offset = h.saturating_sub(1);
        if w <= offset {
            return;
        }

        for j in 0..h {
            let shift = offset.saturating_sub(j);
            self.set_char_force(
                x + shift,
                y + j,
                if j == 0 || j == h - 1 { '*' } else { '╱' },
                CellType::Node,
            );
            self.set_char_force(
                x + shift + (w - offset) - 1,
                y + j,
                if j == 0 || j == h - 1 { '*' } else { '╱' },
                CellType::Node,
            );

            for i in 1..(w - offset - 1) {
                self.set_char_force(
                    x + shift + i,
                    y + j,
                    if j == 0 || j == h - 1 { '─' } else { ' ' },
                    CellType::Node,
                );
            }
        }

        let lines: Vec<&str> = text.split('\n').collect();
        let content_w_avail = w.saturating_sub(offset + 2 + self.padding * 2);

        for (j, line) in lines.iter().enumerate() {
            if j >= h - 2 {
                break;
            }
            let line_len = line.chars().count();
            let safe_len = line_len.min(content_w_avail + 20);
            let display_text: String = line.chars().take(safe_len).collect();

            let row_offset = offset.saturating_sub(j + 1);
            let start_x = x + row_offset + 1 + self.padding;

            self.draw_text(start_x, y + 1 + j, &display_text);
        }
    }

    pub fn draw_rhombus(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let cx = x + w / 2;
        let half_h = h / 2;

        // Top Half + Center
        for j in 0..=half_h {
            // Slope is 2 normally (j*2).
            // But for center (j=half_h), we want it to be "pulled in" by 1 relative to the slope.
            // Effectively: offset = j*2 for j < half_h.
            // offset = j*2 - 1 for j == half_h.

            let offset = if j == half_h { j * 2 - 1 } else { j * 2 };

            let left_x = cx.saturating_sub(1).saturating_sub(offset);
            let right_x = cx + offset;

            if j == half_h {
                self.set_char_force(left_x, y + j, '🯝', CellType::Node);
                self.set_char_force(right_x, y + j, '🯟', CellType::Node);
            } else {
                self.set_char_force(left_x, y + j, '🯐', CellType::Node);
                self.set_char_force(right_x, y + j, '🯓', CellType::Node);
            }

            for k in (left_x + 1)..right_x {
                self.set_char_force(k, y + j, ' ', CellType::Node);
            }
        }

        // Bottom Half
        for j in 0..half_h {
            let r = half_h - 1 - j; // r goes from half_h-1 down to 0.
            let offset = r * 2;

            let left_x = cx.saturating_sub(1).saturating_sub(offset);
            let right_x = cx + offset;
            let draw_y = y + half_h + 1 + j;

            self.set_char_force(left_x, draw_y, '🯒', CellType::Node);
            self.set_char_force(right_x, draw_y, '🯑', CellType::Node);

            for k in (left_x + 1)..right_x {
                self.set_char_force(k, draw_y, ' ', CellType::Node);
            }
        }

        self.draw_centered_text(x, y, w, h, text);
    }

    fn draw_centered_text(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let lines: Vec<&str> = text.split('\n').collect();
        let content_h = lines.len();
        let start_y = y + (h - content_h) / 2;

        for (j, line) in lines.iter().enumerate() {
            let line_len = line.chars().count();
            let start_x = x + (w - line_len) / 2;
            self.draw_text(start_x, start_y + j, line);
        }
    }

    fn draw_content_text(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let lines: Vec<&str> = text.split('\n').collect();
        for (j, line) in lines.iter().enumerate() {
            if j >= h - 2 {
                break;
            }
            let max_w = w.saturating_sub(2 + self.padding * 2);
            let truncated: String = if line.chars().count() > max_w {
                line.chars().take(max_w).collect()
            } else {
                line.to_string()
            };
            let tx = x + 1 + self.padding;
            let ty = y + 1 + j;
            self.draw_text(tx, ty, &truncated);
        }
    }

    pub fn draw_vertical_line(
        &mut self,
        x: usize,
        y1: usize,
        y2: usize,
        arrow: bool,
        label: Option<&str>,
    ) {
        if y1 >= y2 {
            return;
        }
        for y in y1..y2 {
            self.set_char(x, y, '│', CellType::LineV);
        }
        if arrow {
            self.set_char_force(x, y2 - 1, '🭭', CellType::Arrow);
        }
        if let Some(lbl) = label {
            self.draw_text(x + 3, y1, lbl);
        }
    }

    pub fn draw_horizontal_right_line(
        &mut self,
        x1: usize,
        x2: usize,
        y: usize,
        label: Option<&str>,
    ) {
        if x1 >= x2 {
            return;
        }
        for x in x1..=x2 {
            self.set_char(x, y, '─', CellType::LineH);
        }
        if let Some(lbl) = label {
            self.draw_text(x1 + 2, y - 1, lbl);
        }
    }

    pub fn draw_elbow_right_down(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        label: Option<&str>,
    ) {
        self.draw_horizontal_right_line(x1, x2, y1, label);
        self.set_char_force(x2, y1, '┐', CellType::LineCorner);
        self.draw_vertical_line(x2, y1 + 1, y2, true, None);
    }

    pub fn draw_merge_left(
        &mut self,
        x_node: usize,
        y_start: usize,
        x_target: usize,
        y_end: usize,
    ) {
        let turn_y = y_end;
        self.draw_vertical_line(x_node, y_start, turn_y, false, None);
        self.set_char_force(x_node, turn_y, '┘', CellType::LineCorner);
        for x in (x_target..x_node).rev() {
            self.set_char(x, turn_y, '─', CellType::LineH);
        }
        self.set_char_force(x_target, turn_y, '┤', CellType::LineCorner);
    }

    pub fn draw_loop_back(
        &mut self,
        x_start: usize,
        y_start: usize,
        x_target: usize,
        y_target: usize,
        margin_x: usize,
        label: Option<&str>,
    ) {
        let join_y = y_target.saturating_sub(3);

        for x in (margin_x..x_start).rev() {
            self.set_char(x, y_start, '─', CellType::LineH);
        }
        if let Some(lbl) = label {
            let len = lbl.chars().count();
            if x_start > margin_x + len + 2 {
                self.draw_text(x_start - 2 - len, y_start - 1, lbl);
            }
        }
        self.set_char_force(margin_x, y_start, '└', CellType::LineCorner);

        for y in (join_y..y_start).rev() {
            self.set_char(margin_x, y, '│', CellType::LineV);
        }
        self.set_char_force(margin_x, join_y, '┌', CellType::LineCorner);

        for x in margin_x + 1..x_target - 1 {
            self.set_char(x, join_y, '─', CellType::LineH);
        }

        self.set_char_force(x_target - 1, join_y, '🭬', CellType::LineCorner);
        self.set_char_force(x_target, join_y, '│', CellType::LineV);

        for y in join_y + 1..y_target {
            self.set_char(x_target, y, '│', CellType::LineV);
        }
        self.set_char_force(x_target, y_target - 1, '🭭', CellType::Arrow);
        self.set_char(x_target, join_y - 1, '│', CellType::LineV);
    }

    pub fn fix_junctions(&mut self) {
        let width = self.width;
        let height = self.height;

        for y in 0..height {
            for x in 0..width {
                let c = self.canvas[y][x];
                let is_line = matches!(
                    c,
                    '│' | '─' | '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' | '🭬'
                );
                if !is_line {
                    continue;
                }

                let mut up = false;
                let mut down = false;
                let mut left = false;
                let mut right = false;

                if y > 0 {
                    let cu = self.canvas[y - 1][x];
                    if matches!(cu, '│' | '┌' | '┐' | '├' | '┤' | '┬' | '┼' | '🭬') {
                        up = true;
                    }
                }
                if y < height - 1 {
                    let cd = self.canvas[y + 1][x];
                    if matches!(cd, '│' | '└' | '┘' | '├' | '┤' | '┴' | '┼' | '🭬' | '🭭') {
                        down = true;
                    }
                }
                if x > 0 {
                    let cl = self.canvas[y][x - 1];
                    if matches!(cl, '─' | '┌' | '└' | '├' | '┬' | '┴' | '┼' | '🭬') {
                        left = true;
                    }
                }
                if x < width - 1 {
                    let cr = self.canvas[y][x + 1];
                    if matches!(cr, '─' | '┐' | '┘' | '┤' | '┬' | '┴' | '┼' | '🭬') {
                        right = true;
                    }
                }

                let new_char = match (up, down, left, right) {
                    (true, true, true, true) => '┼',
                    (true, true, true, false) => '┤',
                    (true, true, false, true) => '├',
                    (false, true, true, true) => '┬',
                    (true, false, true, true) => '┴',
                    (true, false, false, true) => '└',
                    (true, false, true, false) => '┘',
                    (false, true, false, true) => '┌',
                    (false, true, true, false) => '┐',
                    (true, true, false, false) => '│',
                    (false, false, true, true) => '─',
                    _ => c,
                };

                if new_char != c {
                    self.canvas[y][x] = new_char;
                }
            }
        }
    }

    pub fn render(&mut self) -> String {
        self.fix_junctions();
        self.canvas
            .iter()
            .map(|row| row.iter().collect::<String>())
            .map(|s| s.trim_end().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

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
        let actual_x = lx + (lw - dim.w) / 2;
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
                    if p_id_num > c_id_num {
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
                    p_id_num <= c_id_num
                })
                .count();

            let ready_forward_count = ready_parents
                .iter()
                .filter(|p| {
                    let p_id_num: usize = p.from.strip_prefix("node_").unwrap().parse().unwrap();
                    let c_id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();
                    p_id_num <= c_id_num
                })
                .count();

            if total_forward_parents > 0 && ready_forward_count == total_forward_parents {
                let dim = ctx.node_dims.get(id).unwrap();
                let mut max_y = *lane_y.get(&dim.lane).unwrap_or(&0);

                for p in ready_parents {
                    if let Some((_, py)) = processed_nodes.get(&p.from) {
                        if *py + 8 > max_y {
                            max_y = *py + 8;
                        }
                    }
                }

                *lane_y.entry(dim.lane).or_insert(0) = max_y + dim.h;

                let lx = *lane_x.get(&dim.lane).unwrap();
                let lw = *ctx.lanes_width.get(&dim.lane).unwrap();
                let actual_x = lx + (lw - dim.w) / 2;

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

        if node.text == "Merge" {
            continue;
        }

        match node.node_type {
            NodeType::Terminator => renderer.draw_rounded_box(*x, *y, dim.w, dim.h, &node.text),
            NodeType::Decision => renderer.draw_rhombus(*x, *y, dim.w, dim.h, &node.text),
            NodeType::InputOutput => {
                renderer.draw_parallelogram(*x, *y, dim.w, dim.h, &node.text)
            }
            _ => renderer.draw_box(*x, *y, dim.w, dim.h, &node.text),
        }
    }

    // 5. Draw Edges
    for edge in &flowchart.edges {
        if !node_pos.contains_key(&edge.from) || !node_pos.contains_key(&edge.to) {
            continue;
        }

        let n1 = nodes_map.get(&edge.from).unwrap();
        let n2 = nodes_map.get(&edge.to).unwrap();

        let (x1, y1) = *node_pos.get(&edge.from).unwrap();
        let dim1 = ctx.node_dims.get(&edge.from).unwrap();
        let (x2, y2) = *node_pos.get(&edge.to).unwrap();
        let dim2 = ctx.node_dims.get(&edge.to).unwrap();

        let is_decision = n1.node_type == NodeType::Decision;
        let mut label = edge.label.as_deref();

        // Loop Back Detection using IDs
        let n1_id_num: usize = edge.from.strip_prefix("node_").unwrap().parse().unwrap();
        let n2_id_num: usize = edge.to.strip_prefix("node_").unwrap().parse().unwrap();

        if n1_id_num > n2_id_num {
            // Loop Back
            let sx = x1;
            let sy = y1 + dim1.h / 2;
            let tx = x2 + dim2.w / 2;
            let ty = y2;
            renderer.draw_loop_back(sx, sy, tx, ty, 4, label);
        } else if is_decision && label == Some("Falso") {
            let sx = x1 + dim1.w;
            let sy = y1 + dim1.h / 2;
            let _tx = x2 + dim2.w / 2; // Unused but part of logic intent
            let ty = y2;
            // Draw elbow right-down
            // We need to target x2, but draw_elbow_right_down expects x2.
            renderer.draw_elbow_right_down(sx, sy, x2 + dim2.w / 2, ty, Some("Falso"));
        } else if is_decision && label == Some("Verdadero") {
            let sx = x1 + dim1.w / 2;
            let sy = y1 + dim1.h;
            let _tx = x2 + dim2.w / 2;
            let ty = y2;
            renderer.draw_vertical_line(sx, sy, ty, true, Some("Verdadero"));
        } else if n2.text == "Merge" {
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
            renderer.draw_vertical_line(sx, sy, ty, true, label);
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

    edges.sort_by(|a, b| {
        let la = a.label.as_deref().unwrap_or("");
        let lb = b.label.as_deref().unwrap_or("");
        if la == "Verdadero" {
            std::cmp::Ordering::Less
        } else if lb == "Verdadero" {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    let n_id_num: usize = id.strip_prefix("node_").unwrap().parse().unwrap();

    for edge in edges {
        // ID-based back-edge detection for traversal
        let t_id_num: usize = edge.to.strip_prefix("node_").unwrap().parse().unwrap();
        if t_id_num < n_id_num {
            continue;
        }

        let next_lane_start = if edge.label.as_deref() == Some("Falso") {
            // Heuristic: If False branch goes to End (last node), usually Main Lane.
            // If False branch goes to side logic, Lane + 1.
            // We can check if `edge.to` is the Terminator?
            let target = ctx.nodes_map.get(&edge.to).unwrap();
            if target.node_type == NodeType::Terminator {
                0 // End is on Lane 0
            } else {
                max_lane + 1
            }
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
    if node.text == "Merge" {
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
