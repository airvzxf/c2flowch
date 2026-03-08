use crate::render::routing::RoutingDrawing;
use crate::render::shapes::ShapeDrawing;

pub struct SvgRenderer {
    width: usize,
    height: usize,
    cw: f64,
    ch: f64,
    elements: Vec<String>,
    offset_x: f64,
    offset_y: f64,
}

fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

impl SvgRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cw: 10.0,
            ch: 20.0,
            elements: Vec::new(),
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    pub fn set_offset(&mut self, ox: f64, oy: f64) {
        self.offset_x = ox;
        self.offset_y = oy;
    }

    pub fn add_title(&mut self, title: &str) {
        let lines: Vec<&str> = title.trim().split('\n').collect();
        let mut y = self.offset_y * self.ch - (lines.len() as f64 * 25.0);
        if y < 30.0 {
            y = 30.0;
        }
        for line in lines {
            self.elements.push(format!(
                r#"<text x="{}" y="{}" fill="black" font-weight="bold" font-size="18" font-family="sans-serif">{}</text>"#,
                30.0, y, escape_html(line)
            ));
            y += 25.0;
        }
    }

    pub fn render(&self) -> String {
        let w = (self.width as f64) * self.cw;
        let h = (self.height as f64) * self.ch;

        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            w, h, w, h
        );
        svg.push_str(r#"<defs><marker id="arrowhead" markerWidth="10" markerHeight="7" refX="5" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="black" /></marker></defs>"#);
        svg.push_str(
            r#"<style>text { font-family: monospace; font-size: 14px; font-weight: 400; }</style>"#,
        );
        svg.push_str(r#"<rect width="100%" height="100%" fill="white" />"#);

        for el in &self.elements {
            svg.push_str(el);
            svg.push('\n');
        }

        svg.push_str("</svg>");
        svg
    }

    fn add_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, arrow: bool) {
        let mut x2_mod = x2;
        let mut y2_mod = y2;

        if arrow {
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            if len > 0.0 {
                // Pull back the end of the line by 15 user units
                // so it doesn't poke out of the arrowhead tip, and prevents
                // the tip from overlapping thick black target borders.
                let pullback = len.min(15.0);
                x2_mod = x2 - (dx / len) * pullback;
                y2_mod = y2 - (dy / len) * pullback;
            }
        }

        let marker = if arrow {
            r#" marker-end="url(#arrowhead)""#
        } else {
            ""
        };
        self.elements.push(format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"{} />"#,
            x1, y1, x2_mod, y2_mod, marker
        ));
    }

    fn add_text_aligned(&mut self, x: f64, y: f64, text: &str, anchor: &str) {
        self.elements.push(format!(
            r#"<text x="{}" y="{}" fill="black" dominant-baseline="middle" text-anchor="{}">{}</text>"#,
            x, y, anchor, escape_html(text)
        ));
    }
}

impl ShapeDrawing for SvgRenderer {
    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let rx = (x as f64 + self.offset_x) * self.cw;
        let ry = (y as f64 + self.offset_y) * self.ch;
        let rw = (w as f64) * self.cw;
        let rh = (h as f64) * self.ch;
        self.elements.push(format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="white" stroke="black" stroke-width="2" rx="4" />"#,
            rx, ry, rw, rh
        ));
        self.draw_centered_text(x, y, w, h, text);
    }

    fn draw_predefined_process(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        self.draw_box(x, y, w, h, text);
        let rx = (x as f64 + self.offset_x) * self.cw;
        let ry = (y as f64 + self.offset_y) * self.ch;
        let rh = (h as f64) * self.ch;
        let offset = 2.0 * self.cw;

        self.elements.push(format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2" />"#,
            rx + offset,
            ry,
            rx + offset,
            ry + rh
        ));
        self.elements.push(format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2" />"#,
            rx + (w as f64) * self.cw - offset,
            ry,
            rx + (w as f64) * self.cw - offset,
            ry + rh
        ));
    }

    fn draw_parallelogram(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let rx = (x as f64 + self.offset_x) * self.cw;
        let ry = (y as f64 + self.offset_y) * self.ch;
        let rw = (w as f64) * self.cw;
        let rh = (h as f64) * self.ch;
        let offset = rh / 2.0;

        self.elements.push(format!(
            r#"<polygon points="{},{} {},{} {},{} {},{}" fill="white" stroke="black" stroke-width="2" />"#,
            rx + offset, ry,
            rx + rw, ry,
            rx + rw - offset, ry + rh,
            rx, ry + rh
        ));
        self.draw_centered_text(x, y, w, h, text);
    }

    fn draw_rhombus(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let rx = (x as f64 + self.offset_x) * self.cw;
        let ry = (y as f64 + self.offset_y) * self.ch;
        let rw = (w as f64) * self.cw;
        let rh = (h as f64) * self.ch;

        self.elements.push(format!(
            r#"<polygon points="{},{} {},{} {},{} {},{}" fill="white" stroke="black" stroke-width="2" />"#,
            rx + rw / 2.0, ry,
            rx + rw, ry + rh / 2.0,
            rx + rw / 2.0, ry + rh,
            rx, ry + rh / 2.0
        ));
        self.draw_centered_text(x, y, w, h, text);
    }

    fn draw_terminator(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let rx = (x as f64 + self.offset_x) * self.cw;
        let ry = (y as f64 + self.offset_y) * self.ch;
        let rw = (w as f64) * self.cw;
        let rh = (h as f64) * self.ch;
        let r = rh / 2.0;

        self.elements.push(format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="white" stroke="black" stroke-width="2" rx="{}" ry="{}" />"#,
            rx, ry, rw, rh, r, r
        ));
        self.draw_centered_text(x, y, w, h, text);
    }

    fn draw_switch_case(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        self.draw_terminator(x, y, w, h, text);
    }

    fn draw_centered_text(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        let cx = (x as f64 + self.offset_x + w as f64 / 2.0) * self.cw;
        let cy = (y as f64 + self.offset_y + h as f64 / 2.0) * self.ch;
        let lines: Vec<&str> = text.split('\n').collect();
        let n_lines = lines.len() as f64;

        let max_chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let approx_char_width = 8.4; // width of a 14px monospace character
        let block_width = max_chars as f64 * approx_char_width;
        let start_x = cx - (block_width / 2.0);

        for (i, line) in lines.iter().enumerate() {
            let offset = (i as f64 - (n_lines - 1.0) / 2.0) * 16.0; // 16px line height approx
            self.add_text_aligned(start_x, cy + offset, line, "start");
        }
    }

    fn draw_content_text(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
        self.draw_centered_text(x, y, w, h, text);
    }
}

impl RoutingDrawing for SvgRenderer {
    fn draw_vertical_line(
        &mut self,
        x: usize,
        y1: usize,
        y2: usize,
        arrow: bool,
        label: Option<&str>,
        from_node: bool,
    ) {
        if y1 >= y2 {
            return;
        }
        let vx = (x as f64 + self.offset_x + 0.5) * self.cw;
        let mut vy1 = (y1 as f64 + self.offset_y + 0.5) * self.ch;
        let mut vy2 = (y2 as f64 + self.offset_y + 0.5) * self.ch;

        if from_node {
            vy1 -= self.ch / 2.0; // Start exactly at the bottom boundary of the cell
        }
        if arrow {
            vy2 -= self.ch / 2.0; // Stop exactly at the top boundary of the target cell
        }

        self.add_line(vx, vy1, vx, vy2, arrow);

        if let Some(lbl) = label {
            // For vertical lines, "False" is generally to the left, and other labels to the right.
            if lbl.starts_with("Fals") || lbl == "False" {
                self.add_text_aligned(vx - 10.0, vy1 + 25.0, lbl, "end");
            } else {
                self.add_text_aligned(vx + 10.0, vy1 + 25.0, lbl, "start");
            }
        }
    }

    fn draw_horizontal_right_line(&mut self, x1: usize, x2: usize, y: usize, label: Option<&str>) {
        if x1 >= x2 {
            return;
        }
        let mut vx1 = (x1 as f64 + self.offset_x + 0.5) * self.cw;
        vx1 += self.cw / 2.0; // Start at the right edge of the cell
        let vx2 = (x2 as f64 + self.offset_x + 0.5) * self.cw;
        let vy = (y as f64 + self.offset_y + 0.5) * self.ch;
        self.add_line(vx1, vy, vx2, vy, false);

        if let Some(lbl) = label {
            self.add_text_aligned(vx1 + 15.0, vy - 10.0, lbl, "start");
        }
    }

    fn draw_elbow_right_down(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        label: Option<&str>,
        arrow: bool,
    ) {
        let mut vx1 = (x1 as f64 + self.offset_x + 0.5) * self.cw;
        vx1 += self.cw / 2.0; // Start at the right edge of the cell (node tip)
        let vx2 = (x2 as f64 + self.offset_x + 0.5) * self.cw;
        let vy1 = (y1 as f64 + self.offset_y + 0.5) * self.ch;
        let mut vy2 = (y2 as f64 + self.offset_y + 0.5) * self.ch;

        if arrow {
            vy2 -= self.ch / 2.0; // Stop exactly at the top boundary of the target cell
        }

        self.add_line(vx1, vy1, vx2, vy1, false);
        self.add_line(vx2, vy1, vx2, vy2, arrow);

        if let Some(lbl) = label {
            self.add_text_aligned(vx1 + 15.0, vy1 - 10.0, lbl, "start");
        }
    }

    fn draw_merge_left(
        &mut self,
        x_node: usize,
        y_start: usize,
        x_target: usize,
        y_end: usize,
        from_node: bool,
    ) {
        if x_node == x_target {
            self.draw_vertical_line(x_node, y_start, y_end, false, None, from_node);
            return;
        }
        let vxn = (x_node as f64 + self.offset_x + 0.5) * self.cw;
        let vxt = (x_target as f64 + self.offset_x + 0.5) * self.cw;
        let mut vys = (y_start as f64 + self.offset_y + 0.5) * self.ch;
        if from_node {
            vys -= self.ch / 2.0; // Start at the bottom of the node
        }
        let vye = (y_end as f64 + self.offset_y + 0.5) * self.ch;

        self.add_line(vxn, vys, vxn, vye, false);
        self.add_line(vxn, vye, vxt, vye, true);
    }

    fn draw_loop_back(
        &mut self,
        x_start: usize,
        y_start: usize,
        x_target: usize,
        y_target: usize,
        margin_x: usize,
        label: Option<&str>,
        is_bottom_exit: bool,
        draw_arrow: bool,
        join_y: usize,
    ) {
        let mut vxs = (x_start as f64 + self.offset_x + 0.5) * self.cw;
        let vxt = (x_target as f64 + self.offset_x + 0.5) * self.cw;
        let vxm = (margin_x as f64 + self.offset_x + 0.5) * self.cw;
        let mut vys = (y_start as f64 + self.offset_y + 0.5) * self.ch;
        let mut vyt = (y_target as f64 + self.offset_y + 0.5) * self.ch;
        let vyj = (join_y as f64 + self.offset_y + 0.5) * self.ch;

        if is_bottom_exit {
            vys -= self.ch / 2.0; // Attach perfectly to bottom
        } else {
            vxs -= self.cw / 2.0; // Attach perfectly to left
        }

        let turn_y = if is_bottom_exit { vys + self.ch } else { vys };

        if is_bottom_exit {
            self.add_line(vxs, vys, vxs, turn_y, false);
        }

        self.add_line(vxs, turn_y, vxm, turn_y, false);
        if let Some(lbl) = label {
            if vxm < vxs {
                self.add_text_aligned(vxs - 15.0, turn_y - 10.0, lbl, "end");
            } else {
                self.add_text_aligned(vxs + 15.0, turn_y - 10.0, lbl, "start");
            }
        }

        self.add_line(vxm, turn_y, vxm, vyj, false);
        self.add_line(vxm, vyj, vxt, vyj, true); // Arrow pointing back to main flow

        if draw_arrow {
            vyt -= self.ch / 2.0;
        }
        self.add_line(vxt, vyj, vxt, vyt, draw_arrow);
    }

    fn draw_loop_back_right(
        &mut self,
        x_start: usize,
        y_start: usize,
        x_target: usize,
        y_target: usize,
        margin_x: usize,
        label: Option<&str>,
        is_bottom_exit: bool,
        draw_arrow: bool,
        join_y: usize,
    ) {
        let mut vxs = (x_start as f64 + self.offset_x + 0.5) * self.cw;
        let vxt = (x_target as f64 + self.offset_x + 0.5) * self.cw;
        let vxm = (margin_x as f64 + self.offset_x + 0.5) * self.cw;
        let mut vys = (y_start as f64 + self.offset_y + 0.5) * self.ch;
        let mut vyt = (y_target as f64 + self.offset_y + 0.5) * self.ch;
        let vyj = (join_y as f64 + self.offset_y + 0.5) * self.ch;

        if is_bottom_exit {
            vys -= self.ch / 2.0; // Attach perfectly to bottom
        } else {
            vxs += self.cw / 2.0; // Attach perfectly to right
        }

        let turn_y = if is_bottom_exit { vys + self.ch } else { vys };

        if is_bottom_exit {
            self.add_line(vxs, vys, vxs, turn_y, false);
        }

        self.add_line(vxs, turn_y, vxm, turn_y, false);
        if let Some(lbl) = label {
            if vxm < vxs {
                self.add_text_aligned(vxs - 15.0, turn_y - 10.0, lbl, "end");
            } else {
                self.add_text_aligned(vxs + 15.0, turn_y - 10.0, lbl, "start");
            }
        }

        self.add_line(vxm, turn_y, vxm, vyj, false);
        self.add_line(vxm, vyj, vxt, vyj, true); // Arrow pointing back to main flow

        if draw_arrow {
            vyt -= self.ch / 2.0;
        }
        self.add_line(vxt, vyj, vxt, vyt, draw_arrow);
    }

    fn draw_break_line(&mut self, sx: usize, sy: usize, tx: usize, ty: usize, collector_x: usize) {
        let vsx = (sx as f64 + self.offset_x + 0.5) * self.cw;
        let vtx = (tx as f64 + self.offset_x + 0.5) * self.cw;
        let vcx = (collector_x as f64 + self.offset_x + 0.5) * self.cw;
        let mut vsy = (sy as f64 + self.offset_y + 0.5) * self.ch;
        vsy -= self.ch / 2.0; // Attach to bottom of the node
        let vty = (ty as f64 + self.offset_y + 0.5) * self.ch;
        let y_turn = (sy as f64 + 2.0 + self.offset_y + 0.5) * self.ch;

        self.add_line(vsx, vsy, vsx, y_turn, false);
        self.add_line(vsx, y_turn, vcx, y_turn, false);
        self.add_line(vcx, y_turn, vcx, vty, false);
        self.add_line(vcx, vty, vtx, vty, true);
    }
}
