
#[derive(Clone)]
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
}
