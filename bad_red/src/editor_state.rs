use std::{sync::Arc, time::Duration};

use crossterm::event::KeyEvent;
use mlua::Lua;

use crate::{
    buffer::Buffer, hook_map::{Hook, HookMap, HookName}, keymap::RedKeyEvent, pane::{self, PaneTree, Split}, script_runtime::ScriptScheduler
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
    pub hook_map: HookMap<'a>,
}

impl<'a> Editor<'a> {
    pub fn new(lua: &'a Lua, init_script: String) -> Result<Self> {
        let init_function = lua.load(init_script).into_function()
            .map_err(|e| Error::Unrecoverable(format!("Failed to load init script: {}", e)))?;

        let state = EditorState::new(Duration::from_millis(10));
        Ok(Self {
            state,
            script_scheduler: ScriptScheduler::new(lua, init_function)?,
            hook_map: HookMap::new(),
        })
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let red_key_event = RedKeyEvent::from(key_event);
        let Some(function_iter) = self.hook_map.function_iter(HookName::KeyEvent) else { return Ok(()); };

        for hook_function in function_iter {
            self.script_scheduler.spawn_hook(hook_function.clone(), Hook::KeyEvent(red_key_event.clone()))?;
        }
        Ok(())
    }

    pub fn run_scripts(&mut self) -> Result<bool> {
        self.script_scheduler.run_schedule(&mut self.state, &mut self.hook_map)
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

    pub fn push_to_buffer(&mut self, content: String, index: usize) {
        let Some(ref mut buffer) = &mut self.buffers.get_mut(index) else {
            return;
        };

        buffer.content.push_str(&content);
    }

    pub fn buffer_by_id(&mut self, id: usize) -> Option<&mut Buffer> {
        self.buffers.get_mut(id)
    }

    pub fn active_buffer(&mut self) -> Option<&mut Buffer> {
        let pane = self.pane_tree.pane_by_index(self.active_pane_index)?;

        self.buffers.get_mut(pane.buffer_id)
    }

    pub fn clear_dirty(&mut self) {
        for buffer in &mut self.buffers {
            buffer.is_dirty = false;
        }
    }
}

impl EditorState {
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

    pub fn hsplit(&mut self, index: usize) -> Result<()> {
        let active_pane = self.pane_tree.pane_by_index(index).ok_or_else(|| {
            Error::Unrecoverable(format!(
                "Attempted to split pane but could not find pane at index: {}",
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
