use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

use crossterm::event::Event;
use mlua::Lua;

use crate::{
    buffer::{Buffer, BufferUpdate},
    pane::{self, PaneTree, Split},
    script_handler::ScriptHandler,
    script_runtime::ScriptScheduler,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Unrecoverable(String),
    Recoverable(String),
    Script(String),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Error> for mlua::Error {
    fn from(value: Error) -> Self {
        mlua::Error::ExternalError(Arc::new(value))
    }
}

impl Error {
    pub fn into_lua(self) -> mlua::Error {
        mlua::Error::ExternalError(Arc::new(self))
    }
}

pub struct Editor<'a> {
    pub state: EditorState,
    pub script_scheduler: ScriptScheduler<'a>,
}

impl<'a> Editor<'a> {
    pub fn new(lua: &'a Lua) -> Result<Self> {
        Ok(Self {
            state: EditorState::new(Duration::from_millis(50)),
            script_scheduler: ScriptScheduler::new(lua),
        })
    }

    pub fn handle_event(&mut self, input_event: Event) -> Result<()> {
        let event_result = self.state.handle_event(input_event)?;

        match event_result {
            BufferUpdate::None => Ok(()),
            BufferUpdate::Raw => Ok(()),
            BufferUpdate::Command(command) => self.script_scheduler.spawn_script(command),
        }
    }

    pub fn run_scripts(&mut self) -> Result<()> {
        self.script_scheduler.run_schedule(&mut self.state)
    }
}

fn lua_to_editor_result(lua_result: mlua::Result<()>) -> Result<()> {
    match lua_result {
        Ok(_) => Ok(()),
        Err(error) => lua_error_to_editor_result(error),
    }
}

fn lua_error_to_editor_result(lua_error: mlua::Error) -> Result<()> {
    match lua_error {
        mlua::Error::CallbackError {
            traceback: _,
            cause: e,
        } => callback_error_to_editor_result(e),
        _ => Err(Error::Script(format!("{}", lua_error))),
    }
}

fn callback_error_to_editor_result(callback_cause: Arc<mlua::Error>) -> Result<()> {
    match (*callback_cause).clone() {
        mlua::Error::ExternalError(editor_error) => {
            if let Some(editor_error) = editor_error.downcast_ref::<Error>() {
                match editor_error {
                    Error::Unrecoverable(_) => Err((*editor_error).clone()),
                    Error::Recoverable(_) => Ok(()),
                    Error::Script(_) => Ok(()),
                }
            } else {
                Err(Error::Script(format!("{}", editor_error)))
            }
        }
        other => Err(Error::Script(format!("{}", other))),
    }
}

pub struct EditorState {
    pub active_pane_index: usize,
    pub input_poll_rate: Duration,
    pub buffers: Vec<Buffer>,
    pub pane_tree: PaneTree,
}

impl EditorState {
    pub fn new(input_poll_rate: Duration) -> Self {
        Self {
            active_pane_index: 0,
            input_poll_rate,
            buffers: vec![Buffer::new("root".to_string())],
            pane_tree: PaneTree::new(0),
        }
    }

    pub fn handle_event(&mut self, input_event: Event) -> Result<BufferUpdate> {
        let Some(pane) = self.pane_tree.pane_by_index(self.active_pane_index) else {
            return Err(Error::Unrecoverable(format!(
                "Invalid active pane index. No pane at index {}",
                self.active_pane_index
            )));
        };
        let Some(buffer) = self.buffers.get_mut(pane.buffer_id) else {
            return Err(Error::Unrecoverable(format!(
                "Pane at index {} with invalid buffer id: {}",
                self.active_pane_index, pane.buffer_id
            )));
        };

        Ok(buffer.handle_event(input_event))
    }

    pub fn push_to_buffer(&mut self, content: String, index: usize) {
        let Some(ref mut buffer) = &mut self.buffers.get_mut(index) else {
            return;
        };

        buffer.content.push_str(&content);
    }

    pub fn clear_dirty(&mut self) {
        for buffer in &mut self.buffers {
            buffer.is_dirty = false;
        }
    }
}

impl EditorState {
    pub fn vsplit_active(&mut self) -> Result<()> {
        self.vsplit(self.active_pane_index)
    }

    pub fn vsplit(&mut self, index: usize) -> Result<()> {
        let active_pane = self.pane_tree.pane_node_by_index(index).ok_or_else(|| {
            Error::Unrecoverable(format!(
                "Attempted to split pane but could not find pane at index: {}",
                self.active_pane_index
            ))
        })?;

        let mut current_pane = active_pane;

        let buffer_id = loop {
            match &current_pane.node_type {
                pane::PaneNodeType::Leaf(pane) => break pane.buffer_id,
                pane::PaneNodeType::VSplit(split) | pane::PaneNodeType::HSplit(split) => {
                    current_pane = self
                        .pane_tree
                        .pane_node_by_index(split.first)
                        .ok_or_else(|| {
                            Error::Unrecoverable(format!(
                                "Attemped to find leaf for split pane but pane does not exist at index: {}",
                                split.first
                            ))
                        })?;
                }
            };
        };

        let new_active_index = self
            .pane_tree
            .vsplit(self.active_pane_index, buffer_id)
            .map_err(|e| Error::Recoverable(e))?;

        self.active_pane_index = new_active_index;

        Ok(())
    }

    pub fn hsplit_active(&mut self) -> Result<()> {
        let active_pane = self
            .pane_tree
            .pane_by_index(self.active_pane_index)
            .ok_or_else(|| {
                Error::Unrecoverable(format!(
                    "Attempted to split active pane but could not find active pane at index: {}",
                    self.active_pane_index
                ))
            })?;

        let new_active_index = self
            .pane_tree
            .hsplit(self.active_pane_index, active_pane.buffer_id)
            .map_err(|e| Error::Recoverable(e))?;

        self.active_pane_index = new_active_index;

        Ok(())
    }

    pub fn move_active_up(&mut self) -> Result<()> {
        let active_pane = self
            .pane_tree
            .pane_node_by_index(self.active_pane_index)
            .ok_or_else(|| {
                Error::Unrecoverable(format!(
                    "Attempted to move up with no active pane at index: {}",
                    self.active_pane_index
                ))
            })?;
        let Some(parent_index) = active_pane.parent_index else {
            return Ok(());
        };

        self.active_pane_index = parent_index;
        Ok(())
    }

    pub fn move_down_child(&mut self, to_first: bool) -> Result<()> {
        self.move_down(|split| if to_first { split.first } else { split.second })
    }

    pub fn move_down(&mut self, get_index: impl FnOnce(&Split) -> usize) -> Result<()> {
        let active_pane = self
            .pane_tree
            .pane_node_by_index(self.active_pane_index)
            .ok_or_else(|| {
                Error::Unrecoverable(format!(
                    "Attempted to move down first with no active pane at index: {}",
                    self.active_pane_index
                ))
            })?;
        match &active_pane.node_type {
            pane::PaneNodeType::Leaf(_) => (),
            pane::PaneNodeType::VSplit(split) | pane::PaneNodeType::HSplit(split) => {
                self.active_pane_index = get_index(split)
            }
        }

        Ok(())
    }

    pub fn is_first_child(&self) -> Result<Option<bool>> {
        let Some(parent_index) = self
            .pane_tree
            .pane_node_by_index(self.active_pane_index)
            .ok_or_else(|| {
                Error::Unrecoverable(format!(
                    "Attempted to get child parity with no active pane at index: {}",
                    self.active_pane_index
                ))
            })?
            .parent_index
        else {
            return Ok(None);
        };

        let parity = self
            .pane_tree
            .pane_node_by_index(parent_index)
            .map(|parent| match &parent.node_type {
                pane::PaneNodeType::Leaf(_) => None,
                pane::PaneNodeType::VSplit(split) | pane::PaneNodeType::HSplit(split) => {
                    Some(split)
                }
            })
            .flatten()
            .map(|split| {
                if split.first == self.active_pane_index {
                    Some(true)
                } else if split.second == self.active_pane_index {
                    Some(false)
                } else {
                    None
                }
            })
            .flatten();

        Ok(parity)
    }
}
