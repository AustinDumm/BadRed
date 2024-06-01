use std::str::FromStr;

use mlua::{FromLua, Function, IntoLua, Lua, Table, Value};
use strum::IntoEnumIterator;
use strum_macros::{EnumDiscriminants, EnumIter, EnumString, IntoStaticStr};

use crate::hook_map::{Hook, HookName};

pub struct ScriptHandler {
    pub lua: Lua,
}

trait ScriptObject {
    fn lua_object<'lua>(lua: &'lua Lua) -> mlua::Result<Table<'lua>>;
}

#[derive(EnumDiscriminants)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(derive(IntoStaticStr, EnumString, EnumIter))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[strum_discriminants(name(RedCallName))]
pub enum RedCall<'lua> {
    None,
    PaneVSplit {
        index: usize,
    },
    PaneHSplit {
        index: usize,
    },
    ActivePaneIndex,
    SetActivePane {
        index: usize,
    },
    PaneIndexUpFrom {
        index: usize,
    },
    PaneIndexDownTo {
        index: usize,
        to_first: bool,
    },

    SetHook {
        hook_name: HookName,
        function: Function<'lua>,
    },
    RunHook {
        hook: Hook,
    },

    CurrentBufferId,
    BufferInsert {
        buffer_id: usize,
        content: String,
    },
}

impl<'lua> FromLua<'lua> for RedCall<'lua> {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => return Ok(RedCall::None),
        };

        let call_name = RedCallName::from_str(table.get::<&str, String>("type")?.as_str())
            .map_err(|e| mlua::Error::FromLuaConversionError {
                from: "Table",
                to: "RedCall",
                message: Some(format!("Failed to convert 'type' field to valid RedCall name: {:?}", e))
            })?;

        match call_name {
            RedCallName::None => Ok(RedCall::None),
            RedCallName::PaneVSplit => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneVSplit { index })
            },
            RedCallName::PaneHSplit => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneHSplit { index })
            },
            RedCallName::ActivePaneIndex => Ok(RedCall::ActivePaneIndex),
            RedCallName::SetActivePane => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::SetActivePane { index })
            },
            RedCallName::PaneIndexUpFrom => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneIndexUpFrom { index })
            },
            RedCallName::PaneIndexDownTo => {
                let index = table.get::<&str, usize>("index")?;
                let to_first = table.get::<&str, bool>("to_first")?;
                Ok(RedCall::PaneIndexDownTo { index, to_first })
            }

            RedCallName::SetHook => {
                let hook_name = table.get::<&str, HookName>("hook_name")?;
                let function = table.get::<&str, Function<'_>>("function")?;
                Ok(RedCall::SetHook { hook_name, function })
            },
            RedCallName::RunHook => {
                Err(mlua::Error::FromLuaConversionError {
                    from: "Table",
                    to: "RedCall::RunHook",
                    message: Some(format!("RunHook cannot be converted between Rust and Lua"))
                })
            },

            RedCallName::CurrentBufferId => Ok(RedCall::CurrentBufferId),
            RedCallName::BufferInsert => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                let content = table.get::<&str, String>("content")?;
                Ok(RedCall::BufferInsert { buffer_id, content })
            },
        }
    }
}

impl<'lua> IntoLua<'lua> for RedCall<'_> {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let type_name: &'static str = RedCallName::from(&self).into();
        let table = lua.create_table_from([("type", type_name)])?;
        match self {
            RedCall::None => (),
            RedCall::PaneVSplit { index } => {
                table.set("index", index)?;
            },
            RedCall::PaneHSplit { index } => {
                table.set("index", index)?;
            },
            RedCall::ActivePaneIndex => (),
            RedCall::SetActivePane { index } => {
                table.set("index", index)?;
            },
            RedCall::PaneIndexUpFrom { index } => {
                table.set("index", index)?;
            },
            RedCall::PaneIndexDownTo { index, to_first } => {
                table.set("index", index)?;
                table.set("to_first", to_first)?;
            },
            RedCall::BufferInsert { buffer_id, content } => {
                table.set("buffer_id", buffer_id)?;
                table.set("content", content)?;
            },
            RedCall::CurrentBufferId => (),
            RedCall::SetHook { hook_name, function } => {
                table.set("hook_name", hook_name)?;
                table.set("function", function)?;
            },
            RedCall::RunHook { .. } => {
                Err(mlua::Error::ToLuaConversionError {
                    from: "RedCall::RunHook",
                    to: "Table",
                    message: Some(format!("RedCall::RunHook cannot be converted between Rust and Lua"))
                })?;
            },
        }

        table.into_lua(lua)
    }
}

impl ScriptObject for RedCall<'_> {
    fn lua_object<'lua>(lua: &'lua Lua) -> mlua::Result<Table<'lua>> {
        let table = lua.create_table()?;

        for case in RedCallName::iter() {
            match case {
                RedCallName::None => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, _: ()| Ok(RedCall::None))?,
                    )?;
                }
                RedCallName::PaneVSplit => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, index: usize| Ok(RedCall::PaneVSplit { index }))?,
                    )?;
                }
                RedCallName::PaneHSplit => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, index: usize| Ok(RedCall::PaneHSplit { index }))?,
                    )?;
                }
                RedCallName::ActivePaneIndex => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, _: ()| Ok(RedCall::ActivePaneIndex))?,
                    )?;
                }
                RedCallName::SetActivePane => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, index: usize| {
                            Ok(RedCall::SetActivePane { index })
                        })?,
                    )?;
                }
                RedCallName::PaneIndexUpFrom => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, index: usize| {
                            Ok(RedCall::PaneIndexUpFrom { index })
                        })?,
                    )?;
                }
                RedCallName::PaneIndexDownTo => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (index, to_first): (usize, bool)| {
                            Ok(RedCall::PaneIndexDownTo { index, to_first })
                        })?,
                    )?;
                }
                RedCallName::BufferInsert => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (buffer_id, content): (usize, String)| {
                            Ok(RedCall::BufferInsert { buffer_id, content })
                        })?,
                    )?;
                }
                RedCallName::CurrentBufferId => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, _: ()| {
                            Ok(RedCall::CurrentBufferId)
                        })?,
                    )?;
                }
                RedCallName::SetHook => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (hook_name, function): (HookName, Function<'lua>)| {
                            Ok(RedCall::SetHook { hook_name, function })
                        })?,
                    )?;
                }
                RedCallName::RunHook => { /* RunHook not intended to be a Lua-accessible call */ },
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
