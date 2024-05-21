use crossterm::event::Event;

use crate::{
    buffer::Buffer,
    pane::{Pane, PaneTree, Panes}, script_handler::BuiltIn,
};

type Result<T> = std::result::Result<T, String>;

pub struct EditorState {
    pub active_pane_index: usize,
    pub buffers: Vec<Buffer>,
    pub panes: Panes,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_pane_index: 0,
            buffers: vec![Buffer::new("root".to_string())],
            panes: Panes::new(0),
        }
    }

    pub fn dispatch_input(&mut self, input_event: Event) -> Result<()> {
        let Some(pane) = self.panes.pane_by_index(self.active_pane_index) else {
            return Err(format!("Invalid active pane index. No pane at index {}", self.active_pane_index));
        };
        let Some(buffer) = self.buffers.get_mut(pane.buffer_id) else {
            return Err(format!("Pane at index {} with invalid buffer id: {}", self.active_pane_index, pane.buffer_id));
        };

        buffer.handle_event(input_event);

        Ok(())
    }
}
