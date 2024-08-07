// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// 
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use bad_red_proc_macros::auto_lua;


#[auto_lua]
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
        let unfilled_height = (percent * self.rows as f32) as u16;
        let y_row = unfilled_height.saturating_add_signed(-shift) + self.y_row;
        let rows = self.rows - y_row;

        new.y_row = y_row;
        new.rows = rows;
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
