use crate::EditorSize;

pub struct Panes {
    pub tree: PaneTree,
    pub panes: Vec<Pane>,
}

impl Panes {
    pub fn new() -> Self {
        Self {
            tree: PaneTree::LeafIndex(0),
            panes: vec![Pane::new()],
        }
    }
}

pub enum PaneTree {
    LeafIndex(usize),
    VSplit(Split),
    HSplit(Split),
}

pub struct Split {
    first: Box<PaneTree>,
    second: Box<PaneTree>,
    first_char_size: u16,
}

pub struct Pane {
    pub top_line: u16,
    pub buffer_id: Option<u16>,
    pub is_dirty: bool
}

impl Pane {
    pub fn new() -> Self {
        Self {
            top_line: 0,
            buffer_id: None,
            is_dirty: false,
        }
    }
}
