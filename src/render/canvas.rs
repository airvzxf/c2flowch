use std::collections::HashSet;
use unicode_width::UnicodeWidthChar;

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
    pub(crate) canvas: Vec<Vec<char>>,
    pub(crate) occupancy: Vec<Vec<CellType>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) padding: usize, // Internal padding for nodes
    pub(crate) lines_drawn: HashSet<(usize, usize)>,
}

impl AsciiRenderer {
    pub fn new(width: usize, height: usize, padding: usize) -> Self {
        Self {
            canvas: vec![vec![' '; width]; height],
            occupancy: vec![vec![CellType::Empty; width]; height],
            width,
            height,
            padding,
            lines_drawn: HashSet::new(),
        }
    }

    pub(crate) fn set_char(&mut self, x: usize, y: usize, c: char, cell_type: CellType) {
        if x < self.width && y < self.height && self.occupancy[y][x] != CellType::Node {
            self.canvas[y][x] = c;
            self.occupancy[y][x] = cell_type;
            if matches!(
                cell_type,
                CellType::LineH | CellType::LineV | CellType::LineCorner | CellType::Arrow
            ) {
                self.mark_as_line(x, y);
            }
        }
    }

    pub(crate) fn set_char_force(&mut self, x: usize, y: usize, c: char, cell_type: CellType) {
        if x < self.width && y < self.height {
            self.canvas[y][x] = c;
            self.occupancy[y][x] = cell_type;
            if matches!(
                cell_type,
                CellType::LineH | CellType::LineV | CellType::LineCorner | CellType::Arrow
            ) {
                self.mark_as_line(x, y);
            }
        }
    }

    fn mark_as_line(&mut self, x: usize, y: usize) {
        self.lines_drawn.insert((x, y));
        if x > 0 {
            self.lines_drawn.insert((x - 1, y));
        }
        if x + 1 < self.width {
            self.lines_drawn.insert((x + 1, y));
        }
        if y > 0 {
            self.lines_drawn.insert((x, y - 1));
        }
        if y + 1 < self.height {
            self.lines_drawn.insert((x, y + 1));
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        let lines: Vec<&str> = text.split('\n').collect();
        for (j, line) in lines.iter().enumerate() {
            let mut current_x = x;
            for c in line.chars() {
                let width = c.width().unwrap_or(0);
                if width > 0 {
                    self.set_char_force(current_x, y + j, c, CellType::Text);
                    current_x += width;
                }
            }
        }
    }

    pub fn fix_junctions(&mut self) {
        let width = self.width;
        let height = self.height;

        let coords_to_check = std::mem::take(&mut self.lines_drawn);

        for (x, y) in coords_to_check {
            if x >= width || y >= height {
                continue;
            }

            let c = self.canvas[y][x];
            let is_line = matches!(
                c,
                '│' | '─'
                    | '┌'
                    | '┐'
                    | '└'
                    | '┘'
                    | '├'
                    | '┤'
                    | '┬'
                    | '┴'
                    | '┼'
                    | '🭬'
                    | '🭮'
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
                if matches!(cu, '│' | '┌' | '┐' | '├' | '┤' | '┬' | '┼' | '🭬' | '🭮')
                {
                    up = true;
                }
            }
            if y < height - 1 {
                let cd = self.canvas[y + 1][x];
                if matches!(
                    cd,
                    '│' | '└' | '┘' | '├' | '┤' | '┴' | '┼' | '🭬' | '🭮' | '🭭'
                ) {
                    down = true;
                }
            }
            if x > 0 {
                let cl = self.canvas[y][x - 1];
                if matches!(cl, '─' | '┌' | '└' | '├' | '┬' | '┴' | '┼' | '🭬' | '🭮')
                {
                    left = true;
                }
            }
            if x < width - 1 {
                let cr = self.canvas[y][x + 1];
                if matches!(cr, '─' | '┐' | '┘' | '┤' | '┬' | '┴' | '┼' | '🭬' | '🭮')
                {
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
                (false, false, true, true) => {
                    if matches!(c, '🭬' | '🭮') {
                        c
                    } else {
                        '─'
                    }
                }
                _ => c,
            };

            if new_char != c {
                self.canvas[y][x] = new_char;
            }
        }
    }

    pub fn render(&mut self) -> String {
        self.fix_junctions();
        let lines: Vec<String> = self
            .canvas
            .iter()
            .map(|row| row.iter().collect::<String>())
            .map(|s| s.trim_end().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if lines.is_empty() {
            return String::new();
        }

        let min_leading_spaces = lines
            .iter()
            .map(|line| line.chars().take_while(|&c| c == ' ').count())
            .min()
            .unwrap_or(0);

        lines
            .into_iter()
            .map(|line| {
                if line.len() >= min_leading_spaces {
                    format!("  {}", &line[min_leading_spaces..])
                } else {
                    line
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_junctions_plus() {
        let mut r = AsciiRenderer::new(3, 3, 0);
        // Create a + shape with lines
        r.set_char(1, 0, '│', CellType::LineV);
        r.set_char(1, 1, '┼', CellType::LineCorner); // This will be fixed anyway
        r.set_char(1, 2, '│', CellType::LineV);
        r.set_char(0, 1, '─', CellType::LineH);
        r.set_char(2, 1, '─', CellType::LineH);

        r.fix_junctions();
        assert_eq!(r.canvas[1][1], '┼');
    }

    #[test]
    fn test_fix_junctions_corners() {
        let mut r = AsciiRenderer::new(3, 3, 0);
        // Top-left corner
        r.set_char(1, 1, '┌', CellType::LineCorner);
        r.set_char(2, 1, '─', CellType::LineH);
        r.set_char(1, 2, '│', CellType::LineV);

        r.fix_junctions();
        assert_eq!(r.canvas[1][1], '┌');

        // Bottom-right corner
        let mut r = AsciiRenderer::new(3, 3, 0);
        r.set_char(1, 1, '┘', CellType::LineCorner);
        r.set_char(0, 1, '─', CellType::LineH);
        r.set_char(1, 0, '│', CellType::LineV);
        r.fix_junctions();
        assert_eq!(r.canvas[1][1], '┘');
    }
}
