use crate::{
    buffer::Buffer,
    pane::{Pane, PaneTree, Panes}, script_handler::BuiltIn,
};

pub struct EditorState {
    pub buffers: Vec<Buffer>,
    pub panes: Panes,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            buffers: vec![Buffer::new("root".to_string())],
            panes: Panes::new(),
        }
    }

    pub fn execute_commands(&mut self, commands: Vec<BuiltIn>) {
        for command in commands {
            self.execute_command(command)
        }
    }

    pub fn execute_command(&mut self, command: BuiltIn) {
    }
}
