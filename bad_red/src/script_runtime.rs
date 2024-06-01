use std::{collections::VecDeque, process};

use mlua::{Function, IntoLuaMulti, Lua, Thread};

use crate::{
    editor_state::{EditorState, Error, Result},
    script_handler::RedCall,
};

pub struct ScriptScheduler<'lua> {
    lua: &'lua Lua,
    active: VecDeque<(Thread<'lua>, RedCall<'lua>)>,
}

impl<'lua> ScriptScheduler<'lua> {
    pub fn new(lua: &'lua Lua) -> Self {
        Self {
            lua,
            active: VecDeque::new(),
        }
    }

    pub fn spawn_function<'f>(&mut self, function: Function<'f>, arg: String) -> Result<()> {
        let thread = self
            .lua
            .create_thread(function)
            .map_err(|e| Error::Unrecoverable(format!("Failed to spawn function thread: {}", e)))?;

        self.active
            .push_back((thread, RedCall::Pass { string: arg }));

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

        self.active.push_back((thread, RedCall::None));

        Ok(())
    }

    pub fn run_schedule(&mut self, editor_state: &mut EditorState) -> Result<bool> {
        if self.active.len() == 0 {
            return Ok(false);
        }

        for _ in 0..(self.active.len().min(10)) {
            let Some((next, red_call)) = self.active.pop_front() else {
                return Ok(true);
            };

            match red_call {
                RedCall::None => self.run_script(next, ()),
                RedCall::Pass { string } => self.run_script(next, string),
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
                RedCall::PaneIndexDownTo { index, to_first } => {
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
                                crate::pane::PaneNodeType::Leaf(_) => None,
                                crate::pane::PaneNodeType::VSplit(split)
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
                RedCall::CurrentBufferInsert { key_event } => {
                    let string: String = key_event.try_into().map_err(|e| Error::Recoverable(e))?;

                    let Some(buffer) = editor_state.active_buffer() else {
                        return Ok(true);
                    };
                    buffer.insert_at_cursor(&string);

                    Ok(())
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
                RedCall::SetHook { hook, function } => todo!(),
            }?
        }

        Ok(true)
    }

    fn run_script<A>(&mut self, thread: Thread<'lua>, arg: A) -> Result<()>
    where
        A: IntoLuaMulti<'lua>,
    {
        match thread.status() {
            mlua::ThreadStatus::Resumable => {
                let red_call = thread
                    .resume(arg)
                    .map_err(|e| Error::Script(format!("{}", e)))?;

                self.active.push_back((thread, red_call));

                Ok(())
            }
            mlua::ThreadStatus::Unresumable => Ok(()),
            mlua::ThreadStatus::Error => Err(Error::Unrecoverable(format!(
                "Erring script attempted to be rewoken by scheduler"
            ))),
        }
    }
}
