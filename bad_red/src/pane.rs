use crate::EditorSize;

pub enum Pane {
    Leaf(LeafPane),
    VSplit(Split),
    HSplit(Split),
}

pub struct Split {
    first: Box<Pane>,
    second: Box<Pane>,
    first_char_size: u16,
}

impl Pane {
    pub fn leaf_and_size_at_path<'a>(
        &'a self,
        editor_size: &EditorSize,
        pane_path: u16,
    ) -> Option<(&LeafPane, EditorSize)> {
        match self {
            Pane::Leaf(leaf) => {
                if pane_path == 0 {
                    Some((leaf, editor_size.clone()))
                } else {
                    None
                }
            }
            Pane::VSplit(split) => {
                if (pane_path & 1) == 0 {
                    let child_pane = &split.first;
                    let child_size = editor_size.with_rows(split.first_char_size);
                    child_pane.leaf_and_size_at_path(&child_size, pane_path >> 1)
                } else {
                    let child_pane = &split.second;
                    let child_size = editor_size
                        .with_x_row(split.first_char_size + 2)
                        .less_rows(split.first_char_size + 1);
                    child_pane.leaf_and_size_at_path(&child_size, pane_path >> 1)
                }
            }
            Pane::HSplit(split) => {
                if (pane_path & 1) == 0 {
                    let child_pane = &split.first;
                    let child_size = editor_size.with_cols(split.first_char_size);
                    child_pane.leaf_and_size_at_path(&child_size, pane_path >> 1)
                } else {
                    let child_pane = &split.second;
                    let child_size = editor_size
                        .with_y_col(split.first_char_size + 2)
                        .less_cols(split.first_char_size + 1);
                    child_pane.leaf_and_size_at_path(&child_size, pane_path >> 1)
                }
            }
        }
    }
}

pub struct LeafPane {
    pub top_line: u16,
}

impl LeafPane {
    pub fn new() -> Self {
        Self {
            top_line: 0,
        }
    }
}

