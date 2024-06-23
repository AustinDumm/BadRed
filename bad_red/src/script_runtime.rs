// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::collections::VecDeque;

use mlua::{Function, IntoLuaMulti, Lua, Thread};

use crate::{
    editor_state::{EditorState, Error, Result},
    hook_map::{Hook, HookMap},
    pane::{PaneNodeType, Split, SplitType},
    script_handler::RedCall,
};

pub struct ScriptScheduler<'lua> {
    lua: &'lua Lua,
    active: VecDeque<ScriptProcess<'lua>>,
}

struct ScriptProcess<'lua> {
    thread: Thread<'lua>,
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
        active.push_back(ScriptProcess {
            thread,
            awaiting: RedCall::None,
        });

        Ok(Self { lua, active })
    }

    pub fn spawn_hook<'f>(&mut self, function: Function<'f>, hook: Hook) -> Result<()> {
        let thread = self
            .lua
            .create_thread(function)
            .map_err(|e| Error::Unrecoverable(format!("Failed to spawn function thread: {}", e)))?;

        self.active.push_back(ScriptProcess {
            thread,
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

        self.active.push_back(ScriptProcess {
            thread,
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
                let Some(ScriptProcess {
                    thread: next,
                    awaiting: red_call,
                }) = self.active.pop_front()
                else {
                    return Ok(SchedulerYield::Run);
                };

                let is_script_done = match red_call {
                    RedCall::None => self.run_script(next, ()),
                    RedCall::Yield => self.yield_script(next, ()),

                    RedCall::EditorExit => return Ok(SchedulerYield::Quit),

                    RedCall::PaneVSplit { index: pane_index } => {
                        editor_state.vsplit(pane_index)?;
                        self.run_script(next, ())
                    }
                    RedCall::PaneHSplit { index: pane_index } => {
                        editor_state.hsplit(pane_index)?;
                        self.run_script(next, ())
                    }
                    RedCall::ActivePaneIndex => {
                        let active_index = editor_state.active_pane_index;
                        self.run_script(next, active_index)
                    }
                    RedCall::RootPaneIndex => {
                        let root_index = editor_state.pane_tree.root_index();
                        self.run_script(next, root_index)
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

                        self.run_script(next, is_first)
                    }
                    RedCall::SetActivePane { index } => {
                        if editor_state.pane_tree.tree.len() <= index {
                            Err(Error::Script(format!(
                                "Attempted to set active pane to index out of bounds: {}",
                                index
                            )))
                        } else {
                            editor_state.active_pane_index = index;
                            self.run_script(next, ())
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

                            self.run_script(next, up_index)
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

                            self.run_script(next, down_index)
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

                        self.run_script(next, node_type)
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

                        self.run_script(next, ())
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

                        self.run_script(next, ())
                    }
                    RedCall::PaneBufferIndex { index } => {
                        let Some(pane) = editor_state.pane_tree.pane_by_index(index) else {
                            return Err(Error::Script(format!(
                                "Attempted to retrieve buffer of pane at invalid index: {}",
                                index
                            )));
                        };

                        self.run_script(next, pane.buffer_id)
                    }
                    RedCall::PaneCloseChild { index, first_child } => {
                        let new_active_pane_index = editor_state
                            .pane_tree
                            .close_child(index, first_child, editor_state.active_pane_index)
                            .map_err(|e| {
                                Error::Script(format!("Failed to close pane child: {}", e))
                            })?;

                        editor_state.active_pane_index =
                            new_active_pane_index.unwrap_or(editor_state.active_pane_index);

                        self.run_script(next, ())
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
                                self.run_script(next, ())
                            }
                            PaneNodeType::VSplit(_) | PaneNodeType::HSplit(_) => {
                                Err(Error::Script(format!(
                                    "Attempted to set buffer {} for split pane at index {}",
                                    buffer_index, pane_index
                                )))
                            }
                        }
                    }

                    RedCall::BufferInsert { buffer_id, content } => {
                        let Some(buffer) = editor_state.mut_buffer_by_id(buffer_id) else {
                            return Err(Error::Script(format!(
                                "Attempted to insert text into a buffer with invalid id: {}",
                                buffer_id
                            )));
                        };
                        buffer.insert_at_cursor(&content);

                        self.run_script(next, RedCall::None)
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

                        self.run_script(next, pane.buffer_id)
                    }
                    RedCall::SetHook {
                        hook_name,
                        function,
                    } => {
                        hook_map.add_hook(hook_name, function);

                        self.run_script(next, ())
                    }
                    RedCall::RunHook { hook } => match hook {
                        Hook::KeyEvent(event) => self.run_script(next, event),
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
                        })?;

                        self.active.push_back(ScriptProcess {
                            thread: script_thread,
                            awaiting: RedCall::None,
                        });
                        self.run_script(next, ())
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

                        self.run_script(next, deleted_string)
                    }
                    RedCall::BufferCursorMoveChar {
                        buffer_id,
                        char_count,
                        move_left,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferCursorMoveChar for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        buffer.move_cursor(char_count, move_left);

                        self.run_script(next, ())
                    }
                    RedCall::BufferCursorMoveLine {
                        buffer_id,
                        line_count,
                        move_up,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferCursorMoveLine for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        buffer.move_cursor_line(line_count, move_up);

                        self.run_script(next, ())
                    }
                    RedCall::BufferLength { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferLength for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(next, buffer.content_length())
                    }
                    RedCall::BufferCursorIndex { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferCursorIndex for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(next, buffer.cursor_content_index())
                    }
                    RedCall::BufferSetCursorIndex {
                        buffer_id,
                        cursor_index,
                    } => {
                        let buffer = editor_state.mut_buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferSetCursorIndex for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        buffer.set_cursor_content_index(cursor_index);

                        self.run_script(next, ())
                    }
                    RedCall::BufferContent { buffer_id } => {
                        let buffer = editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
                            Error::Script(format!(
                                "Attempted BufferContent for non-existent buffer: {}",
                                buffer_id
                            ))
                        })?;

                        self.run_script(next, buffer.content())
                    }
                    RedCall::BufferOpen => {
                        let new_buffer_id = editor_state.create_buffer();
                        self.run_script(next, new_buffer_id)
                    }
                    RedCall::BufferClose { buffer_id } => {
                        editor_state.remove_buffer(buffer_id)?;
                        self.run_script(next, ())
                    }
                }?;

                if is_script_done {
                    break 'script_loop;
                }
            }
        }

        Ok(SchedulerYield::Run)
    }

    fn run_script<A>(&mut self, thread: Thread<'lua>, arg: A) -> Result<bool>
    where
        A: IntoLuaMulti<'lua>,
    {
        self.execute_script(thread, arg, false)
    }

    fn yield_script<A>(&mut self, thread: Thread<'lua>, arg: A) -> Result<bool>
    where
        A: IntoLuaMulti<'lua>,
    {
        self.execute_script(thread, arg, true)
    }

    fn execute_script<A>(
        &mut self,
        thread: Thread<'lua>,
        arg: A,
        should_yield: bool,
    ) -> Result<bool>
    where
        A: IntoLuaMulti<'lua>,
    {
        match thread.status() {
            mlua::ThreadStatus::Resumable => {
                let red_call = thread
                    .resume(arg)
                    .map_err(|e| Error::Script(format!("{}", e)))?;

                if should_yield {
                    self.active.push_back(ScriptProcess {
                        thread,
                        awaiting: red_call,
                    });
                } else {
                    self.active.push_front(ScriptProcess {
                        thread,
                        awaiting: red_call,
                    });
                }

                Ok(false || should_yield)
            }
            mlua::ThreadStatus::Unresumable => Ok(true),
            mlua::ThreadStatus::Error => Err(Error::Unrecoverable(format!(
                "Erring script attempted to be rewoken by scheduler"
            ))),
        }
    }
}
