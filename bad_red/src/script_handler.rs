use std::{cell::RefCell, rc::Rc};

use mlua::{Lua, Table};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::editor_state::EditorState;

pub struct ScriptHandler {
    pub state: Rc<RefCell<EditorState>>,
    pub lua: Lua,
}

trait ScriptObject {
    fn lua_object<'lua>(
        lua: &'lua Lua,
        state: &Rc<RefCell<EditorState>>,
    ) -> mlua::Result<Table<'lua>>;
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum PaneBuiltIn {
    VSplit,
    HSplit,
    Up,
    Down,
    IsFirst,
}

impl ScriptObject for PaneBuiltIn {
    fn lua_object<'lua>(
        lua: &'lua Lua,
        state: &Rc<RefCell<EditorState>>,
    ) -> mlua::Result<Table<'lua>> {
        let table = lua.create_table()?;

        for case in Self::iter() {
            match case {
                PaneBuiltIn::VSplit => {
                    let state = state.clone();
                    table.set(
                        Self::V_SPLIT_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<()> {
                            state.try_borrow_mut()
                                .map_err(|e| mlua::Error::RuntimeError(format!("Command (v_split) attempted without unique access to editor state: {:#?}", e)))?
                                .vsplit_active()
                                .map_err(|e| mlua::Error::RuntimeError(e))
                        })?
                    )?;
                }
                PaneBuiltIn::HSplit => {
                    let state = state.clone();
                    table.set(
                        Self::H_SPLIT_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<()> {
                            state
                                .try_borrow_mut()
                                .map_err(|e| mlua::Error::RuntimeError(format!("Command (h_split) attempted without unique access to editor state: {:#?}", e)))?
                                .hsplit_active()
                                .map_err(|e| mlua::Error::RuntimeError(e))
                        })?,
                    )?;
                },
                PaneBuiltIn::Up => {
                    let state = state.clone();
                    table.set(
                        Self::UP_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<()> {
                            state
                                .try_borrow_mut()
                                .map_err(|e| mlua::Error::RuntimeError(format!("Command (up) attempted without unique access to editor state: {:#?}", e)))?
                                .move_active_up()
                                .map_err(|e| mlua::Error::RuntimeError(e))
                        })?,
                    )?;
                },
                PaneBuiltIn::Down => {
                    let state = state.clone();
                    table.set(
                        Self::DOWN_NAME,
                        lua.create_function(move |_, to_first: bool| -> mlua::Result<()> {
                            state
                                .try_borrow_mut()
                                .map_err(|e| mlua::Error::RuntimeError(format!("Command (up) attempted without unique access to editor state: {:#?}", e)))?
                                .move_down_child(to_first)
                                .map_err(|e| mlua::Error::RuntimeError(e))
                        })?,
                    )?;
                },
                PaneBuiltIn::IsFirst => {
                    let state = state.clone();
                    table.set(
                        Self::IS_FIRST_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<Option<bool>> {
                            state
                                .try_borrow_mut()
                                .map_err(|e| mlua::Error::RuntimeError(format!("Command (is_first) attempted without unique access to editor state: {:#?}", e)))?
                                .is_first_child()
                                .map_err(|e| mlua::Error::RuntimeError(e))
                        })?,
                    )?;
                },
            }
        }

        Ok(table)
    }
}

impl PaneBuiltIn {
    const V_SPLIT_NAME: &'static str = "vsplit";
    const H_SPLIT_NAME: &'static str = "hsplit";
    const UP_NAME: &'static str = "up";
    const DOWN_NAME: &'static str = "down";
    const IS_FIRST_NAME: &'static str = "is_first";
}

impl ScriptHandler {
    pub fn new(state: &Rc<RefCell<EditorState>>) -> mlua::Result<Self> {
        let lua = Lua::new();

        let pane_object = PaneBuiltIn::lua_object(&lua, &state)?;

        let red_table = lua.create_table()?;
        red_table.set("pane", pane_object)?;

        lua.globals().set("red", red_table)?;

        Ok(Self {
            state: state.clone(),
            lua,
        })
    }

    pub fn run(&self, script: String) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
