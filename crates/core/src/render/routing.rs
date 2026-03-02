use crate::render::canvas::{AsciiRenderer, CellType};

pub trait RoutingDrawing {
    fn draw_vertical_line(
        &mut self,
        x: usize,
        y1: usize,
        y2: usize,
        arrow: bool,
        label: Option<&str>,
    );
    fn draw_horizontal_right_line(&mut self, x1: usize, x2: usize, y: usize, label: Option<&str>);
    fn draw_elbow_right_down(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        label: Option<&str>,
        arrow: bool,
    );
    fn draw_merge_left(&mut self, x_node: usize, y_start: usize, x_target: usize, y_end: usize);
    #[allow(clippy::too_many_arguments)]
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
    );
    #[allow(clippy::too_many_arguments)]
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
    );
}

impl RoutingDrawing for AsciiRenderer {
    fn draw_vertical_line(
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
            if lbl == "Falso" {
                let lbl_len = lbl.chars().count();
                let tx = x.saturating_sub(lbl_len + 2);
                self.draw_text(tx, y1, lbl);
            } else {
                self.draw_text(x + 3, y1, lbl);
            }
        }
    }

    fn draw_horizontal_right_line(&mut self, x1: usize, x2: usize, y: usize, label: Option<&str>) {
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

    fn draw_elbow_right_down(
        &mut self,
        x1: usize,
        y1: usize,
        x2: usize,
        y2: usize,
        label: Option<&str>,
        arrow: bool,
    ) {
        self.draw_horizontal_right_line(x1, x2, y1, label);
        self.set_char_force(x2, y1, '┐', CellType::LineCorner);
        self.draw_vertical_line(x2, y1 + 1, y2, arrow, None);
    }

    fn draw_merge_left(&mut self, x_node: usize, y_start: usize, x_target: usize, y_end: usize) {
        let turn_y = y_end;
        if x_node == x_target {
            self.draw_vertical_line(x_node, y_start, turn_y, false, None);
            return;
        }
        self.draw_vertical_line(x_node, y_start, turn_y, false, None);
        if x_target < x_node {
            self.set_char_force(x_node, turn_y, '┘', CellType::LineCorner);
            for x in (x_target + 2..x_node).rev() {
                self.set_char(x, turn_y, '─', CellType::LineH);
            }
            self.set_char_force(x_target + 1, turn_y, '🭮', CellType::Arrow);
            self.set_char_force(x_target, turn_y, '├', CellType::LineCorner);
        } else {
            self.set_char_force(x_node, turn_y, '└', CellType::LineCorner);
            for x in x_node + 1..x_target - 1 {
                self.set_char(x, turn_y, '─', CellType::LineH);
            }
            self.set_char_force(x_target - 1, turn_y, '🭬', CellType::Arrow);
            self.set_char_force(x_target, turn_y, '┤', CellType::LineCorner);
        }
    }

    #[allow(clippy::too_many_arguments)]
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
        let turn_y = if is_bottom_exit { y_start + 1 } else { y_start };

        if is_bottom_exit {
            self.set_char(x_start, y_start, '│', CellType::LineV);
            self.set_char_force(x_start, turn_y, '┘', CellType::LineCorner);
        }

        let h_end_x = if is_bottom_exit {
            x_start.saturating_sub(1)
        } else {
            x_start
        };
        for x in (margin_x..=h_end_x).rev() {
            self.set_char(x, turn_y, '─', CellType::LineH);
        }
        if let Some(lbl) = label {
            let len = lbl.chars().count();
            if x_start > margin_x + len + 2 {
                self.draw_text(x_start - 2 - len, turn_y - 1, lbl);
            }
        }
        self.set_char_force(margin_x, turn_y, '└', CellType::LineCorner);

        for y in (join_y..turn_y).rev() {
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
        if draw_arrow {
            self.set_char_force(x_target, y_target - 1, '🭭', CellType::Arrow);
        }
        self.set_char(x_target, join_y - 1, '│', CellType::LineV);
    }

    #[allow(clippy::too_many_arguments)]
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
        let turn_y = if is_bottom_exit { y_start + 1 } else { y_start };

        if is_bottom_exit {
            self.set_char(x_start, y_start, '│', CellType::LineV);
            self.set_char_force(x_start, turn_y, '└', CellType::LineCorner);
        }

        let h_start_x = if is_bottom_exit { x_start + 1 } else { x_start };
        for x in h_start_x..=margin_x {
            self.set_char(x, turn_y, '─', CellType::LineH);
        }
        if let Some(lbl) = label {
            self.draw_text(x_start + 2, turn_y - 1, lbl);
        }
        self.set_char_force(margin_x, turn_y, '┘', CellType::LineCorner);

        for y in (join_y..turn_y).rev() {
            self.set_char(margin_x, y, '│', CellType::LineV);
        }
        self.set_char_force(margin_x, join_y, '┐', CellType::LineCorner);

        for x in (x_target + 2..margin_x).rev() {
            self.set_char(x, join_y, '─', CellType::LineH);
        }

        self.set_char_force(x_target + 1, join_y, '🭮', CellType::LineCorner);
        self.set_char_force(x_target, join_y, '│', CellType::LineV);

        for y in join_y + 1..y_target {
            self.set_char(x_target, y, '│', CellType::LineV);
        }
        if draw_arrow {
            self.set_char_force(x_target, y_target - 1, '🭭', CellType::Arrow);
        }
        self.set_char(x_target, join_y - 1, '│', CellType::LineV);
    }
}
