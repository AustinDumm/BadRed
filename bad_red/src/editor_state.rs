use std::{cell::RefCell, rc::Rc};

use crossterm::event::Event;

use crate::{
    buffer::{Buffer, BufferUpdate},
    pane::{self, PaneTree},
    script_handler::ScriptHandler,
};

type Result<T> = std::result::Result<T, String>;

pub struct Editor {
    pub state: Rc<RefCell<EditorState>>,
    pub script_handler: ScriptHandler,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let state = Rc::new(RefCell::new(EditorState::new()));

        Ok(Self {
            state: state.clone(),
            script_handler: ScriptHandler::new(&state)
                .map_err(|e| format!("Failed to initialize script handler: {}", e))?,
        })
    }

    pub fn handle_event(&mut self, input_event: Event) -> Result<()> {
        let event_result = self.state
            .try_borrow_mut()
            .map_err(|e| format!("Attempted to handle editor event without unique mutable access to editor state: {:#?}", e))?
            .handle_event(input_event)?;

        match event_result {
            BufferUpdate::None => (),
            BufferUpdate::Raw => (),
            BufferUpdate::Command(command) => {
                self.script_handler
                    .run(command)
                    .map_err(|e| format!("Lua script error: {}", e))?;
            }
        }

        Ok(())
    }
}

pub struct EditorState {
    pub active_pane_index: usize,
    pub buffers: Vec<Buffer>,
    pub pane_tree: PaneTree,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_pane_index: 0,
            buffers: vec![Buffer::new("root".to_string())],
            pane_tree: PaneTree::new(0),
        }
    }

    pub fn handle_event(&mut self, input_event: Event) -> Result<BufferUpdate> {
        let Some(pane) = self.pane_tree.pane_by_index(self.active_pane_index) else {
            return Err(format!(
                "Invalid active pane index. No pane at index {}",
                self.active_pane_index
            ));
        };
        let Some(buffer) = self.buffers.get_mut(pane.buffer_id) else {
            return Err(format!(
                "Pane at index {} with invalid buffer id: {}",
                self.active_pane_index, pane.buffer_id
            ));
        };

        Ok(buffer.handle_event(input_event))
    }
}

impl EditorState {
    pub fn vsplit_active(&mut self) -> Result<()> {
        let active_pane = self
            .pane_tree
            .pane_by_index(self.active_pane_index)
            .ok_or_else(|| {
                format!(
                    "Attempted to split active pane but could not find active pane at index: {}",
                    self.active_pane_index
                )
            })?;

        let new_active_index = self.pane_tree
            .vsplit(self.active_pane_index, active_pane.buffer_id)?;

        self.active_pane_index = new_active_index;

        Ok(())
    }

    pub fn hsplit_active(&mut self) -> Result<()> {
        let active_pane = self
            .pane_tree
            .pane_by_index(self.active_pane_index)
            .ok_or_else(|| {
                format!(
                    "Attempted to split active pane but could not find active pane at index: {}",
                    self.active_pane_index
                )
            })?;

        let new_active_index = self.pane_tree
            .vsplit(self.active_pane_index, active_pane.buffer_id)?;

        self.active_pane_index = new_active_index;

        Ok(())
    }
}
