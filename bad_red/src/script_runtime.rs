use std::{collections::VecDeque, process};

use mlua::{IntoLuaMulti, Lua, Thread};

use crate::{
    editor_state::{EditorState, Error, Result},
    script_handler::RedCall,
};

pub struct ScriptScheduler<'lua> {
    lua: &'lua Lua,
    active: VecDeque<(Thread<'lua>, RedCall)>,
}

impl<'a> ScriptScheduler<'a> {
    pub fn new(lua: &'a Lua) -> Self {
        Self {
            lua,
            active: VecDeque::new(),
        }
    }

    pub fn spawn_script(&mut self, script: String) -> Result<()> {
        let thread = self
            .lua
            .create_thread(self.lua.load(script).into_function()
                .map_err(|e| Error::Unrecoverable(format!("Failed to spawn script: {}", e)))?)
                .map_err(|e| Error::Unrecoverable(format!("Failed to spawn script thread: {}", e)))?;

        self.active.push_back((thread, RedCall::None));

        Ok(())
    }

    pub fn run_schedule(&mut self, editor_state: &mut EditorState) -> Result<()> {
        let Some((next, red_call)) = self.active.pop_front() else {
            return Ok(());
        };

        match red_call {
            RedCall::VSplit(pane_index) => {
                editor_state.vsplit(pane_index)?;
                self.run_script(next, ())
            }
            RedCall::None => self.run_script(next, ())
        }
    }

    fn run_script<A>(&mut self, thread: Thread<'a>, arg: A) -> Result<()>
    where
        A: IntoLuaMulti<'a>,
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
