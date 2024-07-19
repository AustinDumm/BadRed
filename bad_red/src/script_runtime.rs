// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{collections::VecDeque, path::Path};

use mlua::{Function, IntoLua, IntoLuaMulti, Lua, Thread, Value};

use crate::{
    buffer::ContentBuffer,
    editor_state::{EditorState, Error, Result},
    hook_map::{HookMap, HookType, HookTypeName},
    pane::{PaneNodeType, Split, SplitType},
    script_handler::RedCall,
};

pub struct ScriptScheduler<'lua> {
    lua: &'lua Lua,
    active: VecDeque<ProcessAwaiting<'lua>>,
}

struct ScriptProcess<'lua> {
    thread: Thread<'lua>,
    cause: Option<HookTypeName>,
}

struct ProcessAwaiting<'lua> {
    process: ScriptProcess<'lua>,
    awaiting: RedCall<'lua>,
}

pub enum SchedulerYield {
    Skip,
    Run,
    Quit,
}

impl<'lua> ScriptScheduler<'lua> {
    pub fn new(lua: &'lua Lua, init: Function<'lua>) -> Result<Self> {
        let thread = lua.create_thread(init).map_err(|e| {
            Error::Unrecoverable(format!("Failed to initialize init thread: {}", e))
        })?;
        let mut active = VecDeque::new();
        active.push_back(ProcessAwaiting {
            process: ScriptProcess {
                thread,
                cause: None,
            },
            awaiting: RedCall::None,
        });

        Ok(Self { lua, active })
    }

    pub fn spawn_all_hooks<'f>(
        &mut self,
        hook_map: &HookMap,
        hook: HookType,
        compare: Option<Value<'lua>>,
    ) -> Result<()> {
        let name = HookTypeName::from(&hook);

        let Some(function_iter) = hook_map.function_iter(name, compare) else {
            return Ok(());
        };
        for function in function_iter {
            self.spawn_hook(function.clone(), hook.clone())?
        }

        Ok(())
    }

    pub fn spawn_hook<'f>(&mut self, function: Function<'f>, hook: HookType) -> Result<()> {
        let thread = self
            .lua
            .create_thread(function)
            .map_err(|e| Error::Unrecoverable(format!("Failed to spawn function thread: {}", e)))?;

        self.active.push_back(ProcessAwaiting {
            process: ScriptProcess {
                thread,
                cause: Some(hook.clone().into()),
            },
            awaiting: RedCall::RunHook { hook },
        });

        Ok(())
    }

    pub fn spawn_script(&mut self, script: String) -> Result<()> {
        let thread = self
            .lua
            .create_thread(
                self.lua
                    .load(script)
                    .into_function()
                    .map_err(|e| Error::Unrecoverable(format!("Failed to spawn script: {}", e)))?,
            )
            .map_err(|e| Error::Unrecoverable(format!("Failed to spawn script thread: {}", e)))?;

        self.active.push_back(ProcessAwaiting {
            process: ScriptProcess {
                thread,
                cause: None,
            },
            awaiting: RedCall::None,
        });

        Ok(())
    }

    const MAX_SCRIPT_CALLS: u8 = std::u8::MAX;
    pub fn run_schedule(
        &mut self,
        editor_state: &mut EditorState,
        hook_map: &mut HookMap<'lua>,
    ) -> Result<SchedulerYield> {
        if self.active.len() == 0 {
            return Ok(SchedulerYield::Skip);
        }

        'script_loop: for _ in 0..Self::MAX_SCRIPT_CALLS {
            for _ in 0..(self.active.len().min(10)) {
                let Some(ProcessAwaiting {
                    process,
                    awaiting: red_call,
                }) = self.active.pop_front()
                else {
                    return Ok(SchedulerYield::Run);
                };

                let is_script_done = match red_call {
                    RedCall::None => self.run_script(process, hook_map, Value::Nil),
                    RedCall::Yield => self.yield_script(process, hook_map, Value::Nil),

                    RedCall::EditorExit => return Ok(SchedulerYield::Quit),

                    RedCall::PaneVSplit { index: pane_index } => {
                        editor_state.vsplit(pane_index)?;
                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::PaneHSplit { index: pane_index } => {
                        editor_state.hsplit(pane_index)?;
                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::ActivePaneIndex => {
                        let active_index = editor_state.active_pane_index;
                        self.run_script(process, hook_map, active_index)
                    }
                    RedCall::RootPaneIndex => {
                        let root_index = editor_state.pane_tree.root_index();
                        self.run_script(process, hook_map, root_index)
                    }
                    RedCall::PaneIsFirst { index } => {
                        let node = editor_state
                            .pane_tree
                            .pane_node_by_index(index)
                            .ok_or_else(|| {
                                Error::Unrecoverable(format!(
                                    "Could not find active pane node while making ActivePane call"
                                ))
                            })?;
                        let is_first = node
                            .parent_index
                            .map(|i| editor_state.pane_tree.pane_node_by_index(i))
                            .flatten()
                            .map(|p| match &p.node_type {
                                crate::pane::PaneNodeType::Leaf(_) => None,
                                crate::pane::PaneNodeType::VSplit(split)
                                | crate::pane::PaneNodeType::HSplit(split) => {
                                    if split.first == index {
                                        Some(true)
                                    } else if split.second == index {
                                        Some(false)
                                    } else {
                                        None
                                    }
                                }
                            });

                        self.run_script(process, hook_map, is_first)
                    }
                    RedCall::SetActivePane { index } => {
                        if editor_state.pane_tree.tree.len() <= index {
                            Err(Error::Script(format!(
                                "Attempted to set active pane to index out of bounds: {}",
                                index
                            )))
                        } else {
                            editor_state.active_pane_index = index;
                            self.run_script(process, hook_map, Value::Nil)
                        }
                    }
                    RedCall::PaneIndexUpFrom { index } => {
                        if editor_state.pane_tree.tree.len() <= index {
                            Err(Error::Script(format!(
                                "Attempted to get parent index from pane index out of bounds: {}",
                                index
                            )))
                        } else {
                            let up_index = editor_state
                                .pane_tree
                                .pane_node_by_index(index)
                                .map(|node| node.parent_index);

                            self.run_script(process, hook_map, up_index)
                        }
                    }
                    RedCall::PaneIndexDownFrom { index, to_first } => {
                        if editor_state.pane_tree.tree.len() <= index {
                            Err(Error::Script(format!(
                                "Attempted to get child index from pane index out of bounds: {}",
                                index
                            )))
                        } else {
                            let down_index = editor_state
                                .pane_tree
                                .pane_node_by_index(index)
                                .map(|node| &node.node_type)
                                .map(|node_type| match node_type {
                                    PaneNodeType::Leaf(_) => None,
                                    PaneNodeType::VSplit(split)
                                    | crate::pane::PaneNodeType::HSplit(split) => {
                                        if to_first {
                                            Some(split.first)
                                        } else {
                                            Some(split.second)
                                        }
                                    }
                                });

                            self.run_script(process, hook_map, down_index)
                        }
                    }
                    RedCall::PaneType { index } => {
                        let node_type = editor_state
                            .pane_tree
                            .pane_node_by_index(index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to get pane type from pane index out of bounds: {}",
                                    index
                                ))
                            })?
                            .node_type
                            .clone();

                        self.run_script(process, hook_map, node_type)
                    }
                    RedCall::PaneSetSplitPercent { index, percent } => {
                        let node = editor_state
                            .pane_tree
                            .pane_node_mut_by_index(index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to get pane from pane index out of bounds: {}",
                                    index
                                ))
                            })?;

                        let (first_changed_pane, second_changed_pane) = match &node.node_type {
                            PaneNodeType::Leaf(_) => Err(Error::Script(format!(
                                "Attempted to set split type for a leaf node at index: {}",
                                index
                            ))),
                            PaneNodeType::VSplit(old_split) => {
                                let panes_used = (old_split.first, old_split.second);
                                node.node_type = PaneNodeType::VSplit(Split {
                                    first: old_split.first,
                                    second: old_split.second,
                                    split_type: SplitType::Percent {
                                        first_percent: percent,
                                    },
                                });

                                Ok(panes_used)
                            }
                            PaneNodeType::HSplit(old_split) => {
                                let panes_used = (old_split.first, old_split.second);
                                node.node_type = PaneNodeType::HSplit(Split {
                                    first: old_split.first,
                                    second: old_split.second,
                                    split_type: SplitType::Percent {
                                        first_percent: percent,
                                    },
                                });

                                Ok(panes_used)
                            }
                        }?;

                        editor_state
                            .pane_tree
                            .pane_node_mut_by_index(first_changed_pane)
                            .ok_or_else(|| {
                                Error::Recoverable(format!(
                                    "Failed to find pane node while changing size for index: {}",
                                    first_changed_pane
                                ))
                            })?
                            .is_dirty = true;
                        editor_state
                            .pane_tree
                            .pane_node_mut_by_index(second_changed_pane)
                            .ok_or_else(|| {
                                Error::Recoverable(format!(
                                    "Failed to find pane node while changing size for index: {}",
                                    second_changed_pane
                                ))
                            })?
                            .is_dirty = true;

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::PaneSetSplitFixed {
                        index,
                        size,
                        to_first,
                    } => {
                        let node = editor_state
                            .pane_tree
                            .pane_node_mut_by_index(index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to get pane from pane index out of bounds: {}",
                                    index
                                ))
                            })?;

                        match &node.node_type {
                            PaneNodeType::Leaf(_) => Err(Error::Script(format!(
                                "Attempted to set split type for a leaf node at index: {}",
                                index
                            ))),
                            PaneNodeType::VSplit(old_split) => {
                                node.node_type = PaneNodeType::VSplit(Split {
                                    first: old_split.first,
                                    second: old_split.second,
                                    split_type: if to_first {
                                        SplitType::FirstFixed { size }
                                    } else {
                                        SplitType::SecondFixed { size }
                                    },
                                });

                                Ok(())
                            }
                            PaneNodeType::HSplit(old_split) => {
                                node.node_type = PaneNodeType::HSplit(Split {
                                    first: old_split.first,
                                    second: old_split.second,
                                    split_type: if to_first {
                                        SplitType::FirstFixed { size }
                                    } else {
                                        SplitType::SecondFixed { size }
                                    },
                                });

                                Ok(())
                            }
                        }?;

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::PaneBufferIndex { index } => {
                        let Some(pane) = editor_state.pane_tree.pane_by_index(index) else {
                            return Err(Error::Script(format!(
                                "Attempted to retrieve buffer of pane at invalid index: {}",
                                index
                            )));
                        };

                        self.run_script(process, hook_map, pane.buffer_id)
                    }
                    RedCall::PaneCloseChild { index, first_child } => {
                        let (new_active_pane_index, closed_index) = editor_state
                            .pane_tree
                            .close_child(index, first_child, editor_state.active_pane_index)
                            .map_err(|e| {
                                Error::Script(format!("Failed to close pane child: {}", e))
                            })?
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "No such pane found while closing child: {}",
                                    index
                                ))
                            })?;

                        editor_state.active_pane_index = new_active_pane_index;

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::PaneSetBuffer {
                        pane_index,
                        buffer_index,
                    } => {
                        let pane = editor_state
                            .pane_tree
                            .pane_node_mut_by_index(pane_index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to set buffer {} for invalid pane: {}",
                                    buffer_index, pane_index
                                ))
                            })?;
                        match pane.node_type {
                            PaneNodeType::Leaf(ref mut pane) => {
                                pane.buffer_id = buffer_index;
                                self.run_script(process, hook_map, Value::Nil)
                            }
                            PaneNodeType::VSplit(_) | PaneNodeType::HSplit(_) => {
                                Err(Error::Script(format!(
                                    "Attempted to set buffer {} for split pane at index {}",
                                    buffer_index, pane_index
                                )))
                            }
                        }
                    }
                    RedCall::PaneWrap { pane_index } => {
                        let pane = editor_state
                            .pane_tree
                            .pane_node_by_index(pane_index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to get pane wrap flag for invalid pane index"
                                ))
                            })?;
                        match &pane.node_type {
                            PaneNodeType::Leaf(leaf) => {
                                self.run_script(process, hook_map, leaf.should_wrap)
                            }
                            PaneNodeType::VSplit(_) | PaneNodeType::HSplit(_) => {
                                self.run_script(process, hook_map, Value::Nil)
                            }
                        }
                    }
                    RedCall::PaneSetWrap {
                        pane_index,
                        should_wrap,
                    } => {
                        let pane = editor_state
                            .pane_tree
                            .pane_node_mut_by_index(pane_index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to get pane wrap flag for invalid pane index"
                                ))
                            })?;
                        match &mut pane.node_type {
                            PaneNodeType::Leaf(leaf) => leaf.should_wrap = should_wrap,
                            PaneNodeType::VSplit(_) | PaneNodeType::HSplit(_) => (),
                        }

                        self.run_script(process, hook_map, Value::Nil)
                    }

                    RedCall::BufferInsert { buffer_id, content } => {
                        let Some(buffer) = editor_state.mut_buffer_by_id(buffer_id) else {
                            return Err(Error::Script(format!(
                                "Attempted to insert text into a buffer with invalid id: {}",
                                buffer_id
                            )));
                        };
                        buffer.insert_at_cursor(&content);

                        self.run_script(process, hook_map, RedCall::None)
                    }
                    RedCall::CurrentBufferId => {
                        let pane = editor_state
                            .pane_tree
                            .pane_by_index(editor_state.active_pane_index)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to find active buffer id without active pane"
                                ))
                            })?;

                        self.run_script(process, hook_map, pane.buffer_id)
                    }
                    RedCall::SetHook {
                        hook_name,
                        function,
                        compare,
                    } => {
                        hook_map.add_hook(hook_name, function, compare);

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::RunHook { hook } => match hook {
                        HookType::KeyEvent(event) => self.run_script(process, hook_map, event),
                        HookType::Error(error_description) => {
                            self.run_script(process, hook_map, error_description)
                        }
                        HookType::SecondaryError(error_description) => {
                            self.run_script(process, hook_map, error_description)
                        }
                        HookType::PaneClosed { pane_id } => {
                            self.run_script(process, hook_map, pane_id)
                        }
                    },

                    RedCall::RunScript { script } => {
                        fn spawn_thread<'lua>(
                            lua: &'lua Lua,
                            script: String,
                        ) -> mlua::Result<mlua::Thread<'lua>> {
                            let function = lua.load(script).into_function()?;
                            lua.create_thread(function)
                        }

                        let script_thread = spawn_thread(&self.lua, script).map_err(|e| {
                            Error::Script(format!(
                                "Failed to create Lua thread for RunScript: {}",
                                e
                            ))
                        });

                        match script_thread {
                            Ok(script_thread) => {
                                self.active.push_back(ProcessAwaiting {
                                    process: ScriptProcess {
                                        thread: script_thread,
                                        cause: None,
                                    },
                                    awaiting: RedCall::None,
                                });
                                self.run_script(process, hook_map, Value::Nil)
                            }
                            Err(error) => self
                                .spawn_all_hooks(
                                    hook_map,
                                    HookType::Error(format!("{}", error)),
                                    None,
                                )
                                .map(|_| true),
                        }
                    }
                    RedCall::BufferDelete {
                        buffer_id,
                        char_count,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted to delete characters from non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        let deleted_string = buffer.delete_at_cursor(char_count);

                        self.run_script(process, hook_map, deleted_string)
                    }
                    RedCall::BufferCursorMovedByChar {
                        buffer_id,
                        char_count,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferCursorMoveChar for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        let moved_cursor = buffer.cursor_moved_by_char(char_count);

                        self.run_script(process, hook_map, moved_cursor)
                    }
                    RedCall::BufferLength { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferLength for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(process, hook_map, buffer.content_byte_length())
                    }
                    RedCall::BufferLineCount { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferLinecount for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(process, hook_map, buffer.content_line_count())
                    }
                    RedCall::BufferCursor { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferCursorIndex for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(process, hook_map, buffer.cursor_byte_index())
                    }
                    RedCall::BufferCursorLine { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferCursorLine for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(process, hook_map, buffer.cursor_line_index())
                    }
                    RedCall::BufferSetCursor {
                        buffer_id,
                        cursor_index,
                        keep_col_index,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferSetCursorIndex for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        buffer.set_cursor_byte_index(cursor_index, keep_col_index);

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::BufferSetCursorLine {
                        buffer_id,
                        line_index,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferSeCursorLine for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        buffer.set_cursor_line_index(line_index);

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::BufferContent { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferContent for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(process, hook_map, buffer.content_copy())
                    }
                    RedCall::BufferOpen => {
                        let new_buffer_id = editor_state.create_buffer();
                        self.run_script(process, hook_map, new_buffer_id)
                    }
                    RedCall::BufferClose { buffer_id } => {
                        editor_state.remove_buffer(buffer_id)?;
                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::BufferLinkFile {
                        buffer_id,
                        file_id,
                        should_overwrite_buffer,
                    } => {
                        editor_state.link_buffer(buffer_id, file_id, should_overwrite_buffer)?;
                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::BufferUnlinkFile {
                        buffer_id,
                        should_force,
                    } => {
                        let file_id = editor_state.unlink_buffer(buffer_id, should_force)?;
                        self.run_script(process, hook_map, file_id)
                    }
                    RedCall::BufferWriteToFile { buffer_id } => {
                        editor_state.write_buffer(buffer_id)?;
                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::BufferCurrentFile { buffer_id } => {
                        let file_id = editor_state.buffer_file_map.get_by_left(&buffer_id).ok_or_else(||
                            Error::Script(format!("Attempted to get current file id for buffer without linked file id: {}", buffer_id))
                        )?;

                        self.run_script(process, hook_map, *file_id)
                    }
                    RedCall::FileOpen { path_string } => {
                        let id = editor_state.open_file(Path::new(&path_string))?;

                        self.run_script(process, hook_map, id)
                    }
                    RedCall::FileClose {
                        file_id,
                        should_force_close,
                    } => {
                        editor_state.close_file(file_id, should_force_close)?;

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::FileCurrentBuffer { file_id } => {
                        let buffer_id = editor_state
                            .buffer_file_map
                            .get_by_right(&file_id)
                            .ok_or_else(|| {
                                Error::Script(format!(
                                    "Attempted to get current buffer id for file at id: {}",
                                    file_id
                                ))
                            })?;

                        self.run_script(process, hook_map, *buffer_id)
                    }
                    RedCall::BufferContentAt {
                        buffer_id,
                        byte_index,
                        char_count,
                    } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted to get current buffer with invalid id: {}",
                                buffer_id
                            ))
                        })?;

                        let content = buffer.content_copy_at_byte_index(byte_index, char_count);

                        self.run_script(process, hook_map, content)
                    }
                    RedCall::BufferLineContent {
                        buffer_id,
                        line_index,
                    } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted to get buffer line content with invalid id: {}",
                                buffer_id
                            ))
                        })?;

                        let content = buffer.content_copy_line(line_index);

                        self.run_script(process, hook_map, content)
                    }
                    RedCall::BufferSetType {
                        buffer_id,
                        buffer_type,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted to get buffer for type set with invalid id: {}",
                                buffer_id
                            ))
                        })?;

                        buffer.set_type(buffer_type);

                        self.run_script(process, hook_map, Value::Nil)
                    }
                    RedCall::BufferType { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted to get buffer type with invalid id: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(process, hook_map, buffer.buffer_type)
                    }
                    RedCall::Value { value } => {
                        self.run_script(process, hook_map, value)
                    }
                }?;

                if is_script_done {
                    break 'script_loop;
                }
            }
        }

        Ok(SchedulerYield::Run)
    }

    fn run_script<A>(
        &mut self,
        process: ScriptProcess<'lua>,
        hook_map: &HookMap,
        arg: A,
    ) -> Result<bool>
    where
        A: IntoLua<'lua>,
    {
        self.execute_script(process, None, hook_map, arg, false)
    }

    fn yield_script<A>(
        &mut self,
        process: ScriptProcess<'lua>,
        hook_map: &HookMap,
        arg: A,
    ) -> Result<bool>
    where
        A: IntoLua<'lua>,
    {
        self.execute_script(process, None, hook_map, arg, true)
    }

    fn execute_script<A>(
        &mut self,
        process: ScriptProcess<'lua>,
        hook_triggered: Option<(HookType, Option<Value<'lua>>)>,
        hook_map: &HookMap,
        arg: A,
        should_yield: bool,
    ) -> Result<bool>
    where
        A: IntoLua<'lua>,
    {
        if let Some((hook_triggered, hook_compare)) = hook_triggered {
            self.active.push_front(ProcessAwaiting {
                process,
                awaiting: RedCall::Value {
                    value: arg.into_lua(self.lua).map_err(|e| {
                        Error::Recoverable(format!(
                            "Failed to convert argument value into lua: {}",
                            e
                        ))
                    })?,
                },
            });

            self.spawn_all_hooks(hook_map, hook_triggered, hook_compare)?;

            Ok(true)
        } else {
            match process.thread.status() {
                mlua::ThreadStatus::Resumable => {
                    match process
                        .thread
                        .resume(arg)
                        .map_err(|e| Error::Script(format!("{}", e)))
                    {
                        Ok(red_call) => {
                            if should_yield {
                                self.active.push_back(ProcessAwaiting {
                                    process: ScriptProcess {
                                        thread: process.thread,
                                        cause: process.cause,
                                    },
                                    awaiting: red_call,
                                });
                            } else {
                                self.active.push_front(ProcessAwaiting {
                                    process: ScriptProcess {
                                        thread: process.thread,
                                        cause: process.cause,
                                    },
                                    awaiting: red_call,
                                });
                            }
                        }
                        Err(err) => match process.cause {
                            Some(HookTypeName::Error) => self.spawn_all_hooks(
                                hook_map,
                                HookType::SecondaryError(format!("{}", err)),
                                None,
                            )?,
                            Some(HookTypeName::SecondaryError) => Err(err)?,
                            Some(_) | None => self.spawn_all_hooks(
                                hook_map,
                                HookType::Error(format!("{}", err)),
                                None,
                            )?,
                        },
                    }

                    Ok(should_yield)
                }
                mlua::ThreadStatus::Unresumable => Ok(true),
                mlua::ThreadStatus::Error => Err(Error::Unrecoverable(format!(
                    "Erring script attempted to be rewoken by scheduler"
                ))),
            }
        }
    }
}
