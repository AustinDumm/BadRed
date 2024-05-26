use std::{cell::RefCell, rc::Rc};

use mlua::{FromLua, IntoLua, Lua, Table, UserData, Value};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::editor_state::{EditorState, Error};

pub struct ScriptHandler {
    pub lua: Lua,
}

trait ScriptObject {
    fn lua_object<'lua>(
        lua: &'lua Lua,
    ) -> mlua::Result<Table<'lua>>;
}

pub enum RedCall {
    None,
    VSplit(usize),
}

impl<'lua> FromLua<'lua> for RedCall {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or(mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "RedCall",
                message: Some(format!("Found non-table value.")),
            })?;

        match table.get::<&str, String>("type")?.as_str() {
            "none" => Ok(RedCall::None),
            "vsplit" => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::VSplit(index))
            },
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
            RedCall::VSplit(index) => {
                let table = lua.create_table()?;
                table.set("type", "vsplit")?;
                table.set("index", index)?;
                table.into_lua(lua)
            }
        }
    }
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
    ) -> mlua::Result<Table<'lua>> {
        let table = lua.create_table()?;

        for case in Self::iter() {
            match case {
                PaneBuiltIn::VSplit => {
                    table.set(
                        Self::V_SPLIT_NAME,
                        lua.create_function(move |_, index: usize| -> mlua::Result<RedCall> {
                            Ok(RedCall::VSplit(index))
                        })?
                    )?;
                }
                PaneBuiltIn::HSplit => {
                    table.set(
                        Self::H_SPLIT_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<RedCall> {
                            Ok(RedCall::None)
                        })?,
                    )?;
                }
                PaneBuiltIn::Up => {
                    table.set(
                        Self::UP_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<RedCall> {
                            Ok(RedCall::None)
                        })?,
                    )?;
                }
                PaneBuiltIn::Down => {
                    table.set(
                        Self::DOWN_NAME,
                        lua.create_function(move |_, to_first: bool| -> mlua::Result<RedCall> {
                            Ok(RedCall::None)
                        })?,
                    )?;
                }
                PaneBuiltIn::IsFirst => {
                    table.set(
                        Self::IS_FIRST_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<RedCall> {
                            Ok(RedCall::None)
                        })?,
                    )?;
                }
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
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();

        let pane_object = PaneBuiltIn::lua_object(&lua)?;

        let red_table = lua.create_table()?;
        red_table.set("pane", pane_object)?;

        lua.globals().set("red", red_table)?;

        Ok(Self {
            lua,
        })
    }

    pub fn run(&self, script: String) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
