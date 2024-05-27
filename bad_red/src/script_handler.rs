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

#[derive(Debug, EnumIter, PartialEq)]
pub enum RedCall {
    None,
    VSplit(usize),
}

impl RedCall {
    const YIELD_NAME: &'static str = "yield";
    const VSPLIT_NAME: &'static str = "vsplit";
}

impl<'lua> FromLua<'lua> for RedCall {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let Some(table) = value.as_table() else {
            return Ok(RedCall::None);
        };

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

impl ScriptObject for RedCall {

    fn lua_object<'lua>(
        lua: &'lua Lua,
    ) -> mlua::Result<Table<'lua>> {
        let table = lua.create_table()?;

        for case in Self::iter() {
            match case {
                RedCall::None => {
                    table.set(
                        Self::YIELD_NAME,
                        lua.create_function(|_, _: ()| Ok(RedCall::None))?
                    )?;
                },
                RedCall::VSplit(_) => {
                    table.set(
                        Self::VSPLIT_NAME,
                        lua.create_function(|_, index: usize| Ok(RedCall::VSplit(index)))?
                    )?;
                },
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

        Ok(Self {
            lua,
        })
    }

    pub fn run(&self, script: String) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
