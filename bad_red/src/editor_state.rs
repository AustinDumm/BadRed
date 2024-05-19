use crate::{
    buffer::Buffer,
    pane::{LeafPane, Pane},
};

pub struct EditorState {
    pub active_buffer: usize,
    pub buffers: Vec<Buffer>,
    pub buffer_panes: Vec<Option<u16>>,
    pub root_pane: Pane,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_buffer: 0,
            buffers: vec![Buffer {
                title: "root".to_string(),
            }],
            buffer_panes: vec![Some(0)],
            root_pane: Pane::Leaf(LeafPane {}),
        }
    }
}
