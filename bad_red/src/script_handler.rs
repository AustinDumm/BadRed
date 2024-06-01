use std::{cell::RefCell, rc::Rc};

use mlua::{FromLua, IntoLua, Lua, Table, UserData, Value};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    editor_state::{EditorState, Error},
    keymap::RedKeyEvent,
};

pub struct ScriptHandler {
    pub lua: Lua,
}

trait ScriptObject {
    fn lua_object<'lua>(lua: &'lua Lua) -> mlua::Result<Table<'lua>>;
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum RedCall {
    None,
    Pass { string: String },
    PaneVSplit { index: usize },
    PaneHSplit { index: usize },
    ActivePaneIndex,
    SetActivePane { index: usize },
    PaneIndexUpFrom { index: usize },
    PaneIndexDownTo { index: usize, to_first: bool },

    CurrentBufferId,
    CurrentBufferInsert { key_event: RedKeyEvent },
}

impl RedCall {
    const YIELD_NAME: &'static str = "yield";
    const PASS_NAME: &'static str = "pass";
    const VSPLIT_NAME: &'static str = "pane_vsplit";
    const HSPLIT_NAME: &'static str = "pane_hsplit";
    const ACTIVE_PANE_NAME: &'static str = "active_pane_index";
    const SET_ACTIVE_PANE_NAME: &'static str = "set_active_pane_index";
    const PANE_INDEX_UP_NAME: &'static str = "pane_index_up_from";
    const PANE_INDEX_DOWN_NAME: &'static str = "pane_index_down_to";
    const CURRENT_BUFFER_INSERT_NAME: &'static str = "current_buffer_insert";
    const CURRENT_BUFFER_ID_NAME: &'static str = "current_buffer_id";
}

impl<'lua> FromLua<'lua> for RedCall {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let Some(table) = value.as_table() else {
            return Ok(RedCall::None);
        };

        match table.get::<&str, String>("type")?.as_str() {
            "none" => Ok(RedCall::None),
            Self::PASS_NAME => {
                let string = table.get::<&str, String>("string")?;
                Ok(RedCall::Pass { string })
            }
            Self::VSPLIT_NAME => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneVSplit { index })
            }
            Self::HSPLIT_NAME => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneHSplit { index })
            }
            Self::ACTIVE_PANE_NAME => Ok(RedCall::ActivePaneIndex),
            Self::SET_ACTIVE_PANE_NAME => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::SetActivePane { index })
            }
            Self::PANE_INDEX_UP_NAME => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneIndexUpFrom { index })
            }
            Self::PANE_INDEX_DOWN_NAME => {
                let index = table.get::<&str, usize>("index")?;
                let to_first = table.get::<&str, bool>("to_first")?;
                Ok(RedCall::PaneIndexDownTo { index, to_first })
            }
            Self::CURRENT_BUFFER_INSERT_NAME => {
                let raw_key_event = table.get::<&str, String>("key_event")?;
                let key_event = RedKeyEvent::try_from(raw_key_event.as_str()).map_err(|e| {
                    mlua::Error::FromLuaConversionError {
                        from: "Value",
                        to: "RedCall::BufferInsert",
                        message: Some(format!(
                            "Failed to convert raw key event into red key event: {}",
                            e
                        )),
                    }
                })?;
                Ok(RedCall::CurrentBufferInsert { key_event })
            }
            other_type => Err(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "RedCall",
                message: Some(format!("Invalid 'type' key found: {}", other_type)),
            }),
        }
    }
}

impl<'lua> IntoLua<'lua> for RedCall {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        match self {
            RedCall::None => lua.create_table_from([("type", "none")])?.into_lua(lua),
            RedCall::Pass { string } => {
                let table = lua.create_table()?;
                table.set("type", Self::PASS_NAME)?;
                table.set("string", string)?;
                table.into_lua(lua)
            }
            RedCall::PaneVSplit { index } => {
                let table = lua.create_table()?;
                table.set("type", Self::VSPLIT_NAME)?;
                table.set("index", index)?;
                table.into_lua(lua)
            }
            RedCall::PaneHSplit { index } => {
                let table = lua.create_table()?;
                table.set("type", Self::HSPLIT_NAME)?;
                table.set("index", index)?;
                table.into_lua(lua)
            }
            RedCall::ActivePaneIndex => lua
                .create_table_from([("type", Self::ACTIVE_PANE_NAME)])?
                .into_lua(lua),
            RedCall::SetActivePane { index } => {
                let table = lua.create_table()?;
                table.set("type", Self::SET_ACTIVE_PANE_NAME)?;
                table.set("index", index)?;
                table.into_lua(lua)
            }
            RedCall::PaneIndexUpFrom { index } => {
                let table = lua.create_table()?;
                table.set("type", Self::PANE_INDEX_UP_NAME)?;
                table.set("index", index)?;
                table.into_lua(lua)
            }
            RedCall::PaneIndexDownTo { index, to_first } => {
                let table = lua.create_table()?;
                table.set("type", Self::PANE_INDEX_UP_NAME)?;
                table.set("index", index)?;
                table.set("to_first", to_first)?;
                table.into_lua(lua)
            }
            RedCall::CurrentBufferInsert { key_event } => {
                let table = lua.create_table()?;
                table.set("type", Self::CURRENT_BUFFER_INSERT_NAME)?;
                table.set("key_event", key_event)?;
                table.into_lua(lua)
            }
            RedCall::CurrentBufferId => lua
                .create_table_from([("type", Self::CURRENT_BUFFER_ID_NAME)])?
                .into_lua(lua),
        }
    }
}

impl ScriptObject for RedCall {
    fn lua_object<'lua>(lua: &'lua Lua) -> mlua::Result<Table<'lua>> {
        let table = lua.create_table()?;

        for case in Self::iter() {
            match case {
                RedCall::None => {
                    table.set(
                        Self::YIELD_NAME,
                        lua.create_function(|_, _: ()| Ok(RedCall::None))?,
                    )?;
                }
                RedCall::Pass { .. } => { /* Pass is only used by the editor, not for Lua use */ }
                RedCall::PaneVSplit { .. } => {
                    table.set(
                        Self::VSPLIT_NAME,
                        lua.create_function(|_, index: usize| Ok(RedCall::PaneVSplit { index }))?,
                    )?;
                }
                RedCall::PaneHSplit { .. } => {
                    table.set(
                        Self::HSPLIT_NAME,
                        lua.create_function(|_, index: usize| Ok(RedCall::PaneHSplit { index }))?,
                    )?;
                }
                RedCall::ActivePaneIndex => {
                    table.set(
                        Self::ACTIVE_PANE_NAME,
                        lua.create_function(|_, _: ()| Ok(RedCall::ActivePaneIndex))?,
                    )?;
                }
                RedCall::SetActivePane { .. } => {
                    table.set(
                        Self::SET_ACTIVE_PANE_NAME,
                        lua.create_function(|_, index: usize| {
                            Ok(RedCall::SetActivePane { index })
                        })?,
                    )?;
                }
                RedCall::PaneIndexUpFrom { .. } => {
                    table.set(
                        Self::PANE_INDEX_UP_NAME,
                        lua.create_function(|_, index: usize| {
                            Ok(RedCall::PaneIndexUpFrom { index })
                        })?,
                    )?;
                }
                RedCall::PaneIndexDownTo { .. } => {
                    table.set(
                        Self::PANE_INDEX_DOWN_NAME,
                        lua.create_function(|_, (index, to_first): (usize, bool)| {
                            Ok(RedCall::PaneIndexDownTo { index, to_first })
                        })?,
                    )?;
                }
                RedCall::CurrentBufferInsert { .. } => {
                    table.set(
                        Self::CURRENT_BUFFER_INSERT_NAME,
                        lua.create_function(|_, key_event: RedKeyEvent| {
                            Ok(RedCall::CurrentBufferInsert { key_event })
                        })?,
                    )?;
                }
                RedCall::CurrentBufferId => {
                    table.set(
                        Self::CURRENT_BUFFER_ID_NAME,
                        lua.create_function(|_, _: ()| {
                            Ok(RedCall::CurrentBufferId)
                        })?,
                    )?;
                }
            }
        }

        Ok(table)
    }
}

impl ScriptHandler {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();

        let redcall_object = RedCall::lua_object(&lua)?;

        let red_table = lua.create_table()?;
        red_table.set("call", redcall_object)?;

        lua.globals().set("red", red_table)?;

        Ok(Self { lua })
    }

    pub fn run(&self, script: String) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
