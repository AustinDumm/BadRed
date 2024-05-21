use crate::EditorFrame;

pub struct Panes {
    pub tree: PaneTree,
    pub panes: Vec<Pane>,
}

impl Panes {
    pub fn new(initial_buffer_id: usize) -> Self {
        Self {
            tree: PaneTree::LeafIndex(0),
            panes: vec![Pane::new(initial_buffer_id)],
        }
    }

    pub fn pane_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a Pane> {
        self.panes.get(pane_index)
    }
}

pub enum PaneTree {
    LeafIndex(usize),
    VSplit(Split),
    HSplit(Split),
}

pub struct Split {
    pub first: Box<PaneTree>,
    pub second: Box<PaneTree>,
    pub first_char_size: u16,
}

pub struct Pane {
    pub top_line: u16,
    pub buffer_id: usize,
}

impl Pane {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            top_line: 0,
            buffer_id,
        }
    }
}
