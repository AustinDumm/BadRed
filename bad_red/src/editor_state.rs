// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{str::FromStr, sync::Arc, time::Duration};

use bad_red_proc_macros::auto_lua;
use bimap::BiMap;
use crossterm::event::KeyEvent;
use mlua::{FromLua, IntoLua, Lua};

use crate::{
    buffer::{ContentBuffer, EditorBuffer},
    file_handle::FileHandle,
    hook_map::{HookMap, HookType, HookTypeName},
    keymap::RedKeyEvent,
    pane::{self, PaneTree, Split},
    script_runtime::{SchedulerYield, ScriptScheduler},
    styling::TextStyleMap,
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
    pub fn new(
        lua: &'a Lua,
        preload_script: String,
        init_script: String,
        starting_file_paths: Vec<String>,
    ) -> Result<Self> {
        let preload_function = lua
            .load(preload_script)
            .into_function()
            .map_err(|e| Error::Unrecoverable(format!("Failed to load preload script: {}", e)))?;

        let init_function = lua
            .load(init_script)
            .into_function()
            .map_err(|e| Error::Unrecoverable(format!("Failed to load init script: {}", e)))?;

        let mut state = EditorState::new(Duration::from_millis(10));

        let has_files = !starting_file_paths.is_empty();
        for path in starting_file_paths.into_iter() {
            state.open_file(path)?;
        }
        let initial_file_id = if has_files {
            Some(0)
        } else {
            None
        };

        Ok(Self {
            state,
            script_scheduler: ScriptScheduler::new(lua, preload_function, init_function, initial_file_id)?,
            hook_map: HookMap::new(),
        })
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let red_key_event = RedKeyEvent::from(key_event);
        let Some(function_iter) = self.hook_map.function_iter(HookTypeName::KeyEvent, None) else {
            return Ok(());
        };

        for hook_function in function_iter {
            self.script_scheduler.spawn_hook(
                hook_function.clone(),
                HookType::KeyEvent(red_key_event.clone()),
            )?;
        }
        Ok(())
    }

    pub fn handle_error(&mut self, error_description: String) -> Result<()> {
        let function_iter = self
            .hook_map
            .function_iter(HookTypeName::Error, None)
            .ok_or_else(|| {
                Error::Recoverable(format!("No error hook set for {}", error_description))
            })?;

        for hook_function in function_iter {
            self.script_scheduler.spawn_hook(
                hook_function.clone(),
                HookType::Error(error_description.clone()),
            )?;
        }
        Ok(())
    }

    pub fn handle_secondary_error(&mut self, error_description: String) -> Result<()> {
        let function_iter = self
            .hook_map
            .function_iter(HookTypeName::SecondaryError, None)
            .ok_or_else(|| {
                Error::Unrecoverable(format!(
                    "No secondary error hook set for {}",
                    error_description
                ))
            })?;

        for hook_function in function_iter {
            self.script_scheduler.spawn_hook(
                hook_function.clone(),
                HookType::Error(error_description.clone()),
            )?;
        }
        Ok(())
    }

    pub fn run_scripts(&mut self) -> Result<SchedulerYield> {
        self.script_scheduler
            .run_schedule(&mut self.state, &mut self.hook_map)
    }
}

pub struct EditorState {
    pub active_pane_index: usize,
    pub input_poll_rate: Duration,
    pub buffers: Vec<Option<EditorBuffer>>,
    pub files: Vec<Option<FileHandle>>,
    pub pane_tree: PaneTree,
    pub options: EditorOptions,

    pub style_map: TextStyleMap,

    pub buffer_file_map: BiMap<usize, usize>,
}

impl EditorState {
    pub fn new(input_poll_rate: Duration) -> Self {
        Self {
            active_pane_index: 0,
            input_poll_rate,
            buffers: vec![Some(EditorBuffer::new())],
            files: vec![],
            pane_tree: PaneTree::new(0),

            buffer_file_map: BiMap::new(),
            options: EditorOptions { tab_width: 8 },

            style_map: TextStyleMap::new(),
        }
    }

    pub fn buffer_by_id(&self, id: usize) -> Option<&EditorBuffer> {
        self.buffers.get(id).map(|b| b.as_ref()).flatten()
    }

    pub fn mut_buffer_by_id(&mut self, id: usize) -> Option<&mut EditorBuffer> {
        self.buffers.get_mut(id).map(|b| b.as_mut()).flatten()
    }

    pub fn active_buffer(&mut self) -> Option<&mut EditorBuffer> {
        let pane = self.pane_tree.pane_by_index(self.active_pane_index)?;

        self.buffers
            .get_mut(pane.buffer_id)
            .map(|b| b.as_mut())
            .flatten()
    }

    pub fn clear_dirty(&mut self) {
        for buffer in &mut self.buffers {
            if let Some(buffer) = buffer {
                buffer.is_render_dirty = false;
            }
        }
    }

    pub fn create_buffer(&mut self) -> usize {
        let new_buffer_id = self.buffers.len();
        self.buffers.push(Some(EditorBuffer::new()));

        new_buffer_id
    }

    pub fn remove_buffer(&mut self, index: usize) -> Result<()> {
        if self
            .buffers
            .get(index)
            .map(|b| b.as_ref())
            .flatten()
            .is_none()
        {
            Err(Error::Unrecoverable(format!(
                "Attempted to remove a buffer at an index that doesn't have a buffer: {}",
                index
            )))
        } else {
            self.buffers[index] = None;
            Ok(())
        }
    }

    pub fn open_file(&mut self, path: String) -> Result<usize> {
        if self
            .files
            .iter()
            .find(|handle| handle.as_ref().map(|h| *h.path == *path).unwrap_or(false))
            .is_some()
        {
            return Err(Error::Recoverable(format!(
                "Attempted to open file that is already open: {:?}",
                path
            )));
        }

        let handle = FileHandle::new(path)
            .map_err(|e| Error::Recoverable(format!("Failed to open file: {:#?}", e)))?;
        let handle_id = self.files.len();
        self.files.push(Some(handle));

        Ok(handle_id)
    }

    pub fn close_file(&mut self, file_id: usize, should_force: bool) -> Result<()> {
        if self
            .files
            .get(file_id)
            .map(|f| f.as_ref())
            .flatten()
            .is_none()
        {
            return Err(Error::Recoverable(format!(
                "Attempted to close non-existent file at id: {}",
                file_id
            )));
        }

        if let Some(buffer_id) = self.buffer_file_map.get_by_right(&file_id) {
            let buffer_id = *buffer_id;
            if let Some(buffer) = self.mut_buffer_by_id(buffer_id) {
                if buffer.is_content_dirty && !should_force {
                    return Err(Error::Recoverable(format!("Attempted to close file with dirty buffer unforced. File id: {}, buffer id: {}", file_id, buffer_id)));
                }

                buffer.is_content_dirty = false;
            }

            self.buffer_file_map.remove_by_right(&file_id);
        }

        self.files[file_id] = None;

        Ok(())
    }

    pub fn link_buffer(
        &mut self,
        buffer_id: usize,
        file_id: usize,
        should_populate_buffer: bool,
    ) -> Result<()> {
        if self.buffer_file_map.contains_left(&buffer_id) {
            return Err(Error::Recoverable(format!(
                "Attempted to link buffer id that already has file associated. Buffer id: {}",
                buffer_id
            )));
        }
        if self.buffer_file_map.contains_right(&file_id) {
            return Err(Error::Recoverable(format!(
                "Attempted to link file id that already has file associated. File id: {}",
                file_id
            )));
        }

        let buffer = self
            .buffers
            .get_mut(buffer_id)
            .map(|b| b.as_mut())
            .flatten()
            .ok_or_else(|| {
                Error::Recoverable(format!(
                    "Attempted to link invalid buffer id to file. Buffer id: {}",
                    buffer_id
                ))
            })?;
        let mut file_handle = self
            .files
            .get_mut(file_id)
            .map(|b| b.as_mut())
            .flatten()
            .ok_or_else(|| {
                Error::Recoverable(format!(
                    "Attempted to link invalid file id to buffer: File id: {}",
                    file_id
                ))
            })?;

        self.buffer_file_map.insert(buffer_id, file_id);

        if should_populate_buffer {
            buffer
                .populate_from_read(&mut file_handle)
                .map_err(|e| Error::Recoverable(format!("Failed to read from file: {:#?}", e)))?;
        }

        Ok(())
    }

    pub fn unlink_buffer(&mut self, buffer_id: usize, force: bool) -> Result<usize> {
        let buffer = self
            .buffers
            .get_mut(buffer_id)
            .map(|b| b.as_mut())
            .flatten()
            .ok_or_else(|| {
                Error::Recoverable(format!(
                    "Attempted to unlink buffer with invalid buffer id: {}",
                    buffer_id
                ))
            })?;

        if !self.buffer_file_map.contains_left(&buffer_id) {
            return Err(Error::Recoverable(format!(
                "Attempted to unlink buffer that is not linked to a file. Buffer id: {}",
                buffer_id
            )));
        }

        if buffer.is_content_dirty && !force {
            return Err(Error::Recoverable(format!(
                "Attempted to unlink dirty buffer id {} without force",
                buffer_id
            )));
        }

        buffer.is_content_dirty = false;

        self.buffer_file_map
            .remove_by_left(&buffer_id)
            .map(|(_, file_id)| file_id)
            .ok_or_else(|| {
                Error::Recoverable(format!(
                    "Failed to find link for buffer id {} to unlink with",
                    buffer_id
                ))
            })
    }

    pub fn write_buffer(&mut self, buffer_id: usize) -> Result<()> {
        let Some(file_id) = self.buffer_file_map.get_by_left(&buffer_id) else {
            return Err(Error::Recoverable(format!(
                "Attempted to write from buffer id that has no file associated. Buffer id: {}",
                buffer_id
            )));
        };
        let buffer = self
            .buffers
            .get_mut(buffer_id)
            .map(|b| b.as_mut())
            .flatten()
            .ok_or_else(|| {
                Error::Recoverable(format!(
                    "Attempted to write from invalid buffer id: {}",
                    buffer_id
                ))
            })?;
        if !buffer.is_content_dirty {
            return Ok(());
        }

        let file_handle = self
            .files
            .get_mut(*file_id)
            .map(|f| f.as_mut())
            .flatten()
            .ok_or_else(|| {
                Error::Recoverable(format!(
                    "Attempted to write from buffer id {} to invalid file id: {}",
                    buffer_id, file_id
                ))
            })?;

        buffer.flush_to_write(file_handle).map_err(|e| {
            Error::Recoverable(format!(
                "Failed to write buffer id {} contents out to file id {}. {}",
                buffer_id, file_id, e
            ))
        })
    }
}

impl EditorState {
    pub fn vsplit(&mut self, index: usize) -> Result<()> {
        let active_pane = self.pane_tree.pane_node_by_index(index).ok_or_else(|| {
            Error::Unrecoverable(format!(
                "Attempted to split pane but could not find pane at index: {}",
                index
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

        let new_split_root_index = self
            .pane_tree
            .vsplit(index, buffer_id)
            .map_err(|e| Error::Recoverable(e))?;

        if self.active_pane_index == index {
            self.active_pane_index = new_split_root_index
        }

        Ok(())
    }

    pub fn hsplit(&mut self, index: usize) -> Result<()> {
        let active_pane = self.pane_tree.pane_node_by_index(index).ok_or_else(|| {
            Error::Unrecoverable(format!(
                "Attempted to split pane but could not find pane at index: {}",
                index
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

        let new_split_root_index = self
            .pane_tree
            .hsplit(index, buffer_id)
            .map_err(|e| Error::Recoverable(e))?;

        if self.active_pane_index == index {
            self.active_pane_index = new_split_root_index
        }

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

#[auto_lua]
#[derive(Clone)]
pub struct EditorOptions {
    pub tab_width: u16,
}

impl EditorOptions {
    pub fn update(&mut self, update_list: EditorOptionList) {
        for update in update_list.0 {
            match update {
                EditorOptionType::TabWidth(new_width) => self.tab_width = new_width,
            }
        }
    }
}

#[auto_lua]
pub enum EditorOptionType {
    TabWidth(u16),
}

pub struct EditorOptionList(Vec<EditorOptionType>);

impl<'lua> FromLua<'lua> for EditorOptionList {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        let mut option_list = vec![];

        for pair in value
            .as_table()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "EditorOptionList",
                message: Some(format!(
                    "Expected lua table as representation of EditorOptionList"
                )),
            })?
            .clone()
            .pairs::<mlua::Value, mlua::Value>()
        {
            let (option_key, option_value) = pair?;
            let Some(key_str) = option_key.as_str() else {
                continue;
            };
            let Ok(key) = EditorOptionTypeName::from_str(key_str) else {
                continue;
            };

            match key {
                EditorOptionTypeName::TabWidth => {
                    let Some(value) = option_value.as_u32() else {
                        continue;
                    };

                    option_list.push(EditorOptionType::TabWidth(value as u16));
                }
            }
        }

        Ok(EditorOptionList(option_list))
    }
}

impl<'lua> IntoLua<'lua> for EditorOptionList {
    fn into_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        let table = lua.create_table()?;
        for item in self.0 {
            match item {
                EditorOptionType::TabWidth(width) => {
                    table.set(EditorOptionTypeName::TabWidth, width)?
                }
            }
        }

        table.into_lua(lua)
    }
}
