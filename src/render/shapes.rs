use crate::render::canvas::{AsciiRenderer, CellType};

pub trait ShapeDrawing {
    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
    fn draw_parallelogram(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
    fn draw_rhombus(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
    fn draw_terminator(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
    fn draw_switch_case(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
    fn draw_centered_text(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
    fn draw_content_text(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str);
}

impl ShapeDrawing for AsciiRenderer {
    fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
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

    fn draw_parallelogram(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
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

    fn draw_rhombus(&mut self, x: usize, y: usize, w: usize, h: usize, text: &str) {
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

    fn draw_terminator(&mut self, mut x: usize, y: usize, w: usize, h: usize, text: &str) {
        // Shift 1 space right if text length is even to improve alignment with the center arrow
        let content_w = text
            .split('\n')
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);
        if content_w.is_multiple_of(2) {
            x += 1;
        }

        // En lugar de una caja, dibuja (  texto  )
        // Rellena de espacios el centro
        for j in 0..h {
            for i in 0..w {
                self.set_char_force(x + i, y + j, ' ', CellType::Node);
            }
            // Los paréntesis solo en los extremos de cada línea si es multilínea,
            // pero normalmente es solo una.
            self.set_char_force(x, y + j, '(', CellType::Node);
            self.set_char_force(x + w - 1, y + j, ')', CellType::Node);
        }

        self.draw_centered_text(x, y, w, h, text);
    }

    fn draw_switch_case(&mut self, mut x: usize, y: usize, w: usize, _h: usize, text: &str) {
        // Shift 1 space right if text length is even to improve alignment with the center arrow
        let content_w = text.chars().count();
        if content_w.is_multiple_of(2) {
            x += 1;
        }

        // En lugar de una caja, dibuja ( texto )
        // Rellena de espacios el centro para borrar líneas que puedan pasar
        for i in 0..w {
            self.set_char_force(x + i, y, ' ', CellType::Node);
        }

        let text_with_parens = format!("( {} )", text);
        let len = text_with_parens.chars().count();
        let start_x = x + (w - len) / 2;
        self.draw_text(start_x, y, &text_with_parens);
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
}
