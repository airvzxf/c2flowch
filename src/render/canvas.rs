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

    pub(crate) fn set_char(&mut self, x: usize, y: usize, c: char, cell_type: CellType) {
        if x < self.width && y < self.height && self.occupancy[y][x] != CellType::Node {
            self.canvas[y][x] = c;
            self.occupancy[y][x] = cell_type;
        }
    }

    pub(crate) fn set_char_force(&mut self, x: usize, y: usize, c: char, cell_type: CellType) {
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

    pub fn fix_junctions(&mut self) {
        let width = self.width;
        let height = self.height;

        for y in 0..height {
            for x in 0..width {
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
