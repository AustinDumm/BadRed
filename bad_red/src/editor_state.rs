use crate::{
    buffer::Buffer,
    pane::{LeafPane, Pane}, script_handler::BuiltIn,
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
            buffers: vec![Buffer::new("root".to_string())],
            buffer_panes: vec![Some(0)],
            root_pane: Pane::Leaf(LeafPane::new()),
        }
    }

    pub fn execute_commands(&mut self, commands: Vec<BuiltIn>) {
        for command in commands {
            self.execute_command(command)
        }
    }

    pub fn execute_command(&mut self, command: BuiltIn) {
        match command {
            BuiltIn::VSplit => {
                let Some(pane_path) = self.buffer_panes[self.active_buffer] else { return; };
                let Some(pane) = self.root_pane.leaf_at_path(pane_path) else { return; };
            },
            BuiltIn::HSplit => todo!(),
        }
    }
}
