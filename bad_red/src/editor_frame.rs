
#[derive(Clone, Debug)]
pub struct EditorFrame {
    pub x_col: u16,
    pub y_row: u16,
    pub rows: u16,
    pub cols: u16,
}

impl EditorFrame {
    pub fn with_x_col(&self, x_col: u16) -> Self {
        let mut new = self.clone();
        new.x_col = x_col;
        new
    }

    pub fn with_y_row(&self, y_row: u16) -> Self {
        let mut new = self.clone();
        new.y_row = y_row;
        new
    }

    pub fn with_rows(&self, rows: u16) -> Self {
        let mut new = self.clone();
        new.rows = rows;
        new
    }

    pub fn with_cols(&self, cols: u16) -> Self {
        let mut new = self.clone();
        new.cols = cols;
        new
    }

    pub fn less_rows(&self, rows: u16) -> Self {
        let mut new = self.clone();
        new.rows -= rows;
        new
    }

    pub fn less_cols(&self, cols: u16) -> Self {
        let mut new = self.clone();
        new.cols -= cols;
        new
    }

    pub fn percent_rows(&self, percent: f32, shift: i16) -> Self {
        let mut new = self.clone();
        let scaled = (percent * self.rows as f32) as u16;
        new.rows = scaled.saturating_add_signed(shift);
        new
    }

    pub fn percent_cols(&self, percent: f32, shift: i16) -> Self {
        let mut new = self.clone();
        let scaled = (percent * self.cols as f32) as u16;
        new.cols = scaled.saturating_add_signed(shift);
        new
    }

    pub fn percent_rows_shift(&self, percent: f32, shift: i16) -> Self {
        let mut new = self.clone();
        let unfilled_width = (percent * self.rows as f32) as u16;
        let unfilled_width = unfilled_width.saturating_add_signed(shift);

        let filled_width = (self.rows - unfilled_width).saturating_add_signed(-2 * shift);

        new.x_col = unfilled_width.saturating_add_signed(2 * shift);
        new.rows = filled_width;
        new
    }

    pub fn percent_cols_shift(&self, percent: f32, shift: i16) -> Self {
        let mut new = self.clone();
        let unfilled_width = (percent * self.cols as f32) as u16;
        let x_col = unfilled_width.saturating_add_signed(-shift) + self.x_col;
        let cols = self.cols - x_col;

        new.x_col = x_col;
        new.cols = cols;
        new
    }
}
