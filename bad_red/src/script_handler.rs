// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// 
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

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
    Yield,
    PaneVSplit {
        index: usize,
    },
    PaneHSplit {
        index: usize,
    },
    ActivePaneIndex,
    PaneIsFirst {
        index: usize,
    },
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

    RunScript {
        script: String
    },

    CurrentBufferId,
    BufferInsert {
        buffer_id: usize,
        content: String,
    },
    BufferDelete {
        buffer_id: usize,
        char_count: usize,
    },
    BufferCursorMoveChar {
        buffer_id: usize,
        char_count: usize,
        move_left: bool,
    },
    BufferLength {
        buffer_id: usize,
    },
    BufferCursorIndex {
        buffer_id: usize,
    },
    BufferSetCursorIndex {
        buffer_id: usize,
        cursor_index: usize,
    },
    BufferContent {
        buffer_id: usize,
    }

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
                message: Some(format!(
                    "Failed to convert 'type' field to valid RedCall name: {:?}",
                    e
                )),
            })?;

        match call_name {
            RedCallName::None => Ok(RedCall::None),
            RedCallName::Yield => Ok(RedCall::Yield),
            RedCallName::PaneVSplit => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneVSplit { index })
            }
            RedCallName::PaneHSplit => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneHSplit { index })
            }
            RedCallName::ActivePaneIndex => Ok(RedCall::ActivePaneIndex),
            RedCallName::PaneIsFirst => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneIsFirst { index })
            }
            RedCallName::SetActivePane => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::SetActivePane { index })
            }
            RedCallName::PaneIndexUpFrom => {
                let index = table.get::<&str, usize>("index")?;
                Ok(RedCall::PaneIndexUpFrom { index })
            }
            RedCallName::PaneIndexDownTo => {
                let index = table.get::<&str, usize>("index")?;
                let to_first = table.get::<&str, bool>("to_first")?;
                Ok(RedCall::PaneIndexDownTo { index, to_first })
            }

            RedCallName::SetHook => {
                let hook_name = table.get::<&str, HookName>("hook_name")?;
                let function = table.get::<&str, Function<'_>>("function")?;
                Ok(RedCall::SetHook {
                    hook_name,
                    function,
                })
            }
            RedCallName::RunHook => Err(mlua::Error::FromLuaConversionError {
                from: "Table",
                to: "RedCall::RunHook",
                message: Some(format!("RunHook cannot be converted between Rust and Lua")),
            }),

            RedCallName::RunScript => {
                let script = table.get::<&str, String>("script")?;
                Ok(RedCall::RunScript {
                    script
                })
            },

            RedCallName::CurrentBufferId => Ok(RedCall::CurrentBufferId),
            RedCallName::BufferInsert => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                let content = table.get::<&str, String>("content")?;
                Ok(RedCall::BufferInsert { buffer_id, content })
            }
            RedCallName::BufferDelete => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                let char_count = table.get::<&str, usize>("char_count")?;
                Ok(RedCall::BufferDelete { buffer_id, char_count })
            },
            RedCallName::BufferCursorMoveChar => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                let char_count = table.get::<&str, usize>("char_count")?;
                let move_left = table.get::<&str, bool>("move_left")?;
                Ok(RedCall::BufferCursorMoveChar { buffer_id, char_count, move_left })
            },
            RedCallName::BufferLength => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                Ok(RedCall::BufferLength { buffer_id })
            },
            RedCallName::BufferCursorIndex => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                Ok(RedCall::BufferCursorIndex { buffer_id })
            },
            RedCallName::BufferSetCursorIndex => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                let cursor_index = table.get::<&str, usize>("cursor_index")?;
                Ok(RedCall::BufferSetCursorIndex { buffer_id, cursor_index })
            },
            RedCallName::BufferContent => {
                let buffer_id = table.get::<&str, usize>("buffer_id")?;
                Ok(RedCall::BufferContent { buffer_id })
            }
        }
    }
}

impl<'lua> IntoLua<'lua> for RedCall<'_> {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let type_name: &'static str = RedCallName::from(&self).into();
        let table = lua.create_table_from([("type", type_name)])?;
        match self {
            RedCall::None => (),
            RedCall::Yield => (),
            RedCall::PaneVSplit { index } => {
                table.set("index", index)?;
            }
            RedCall::PaneHSplit { index } => {
                table.set("index", index)?;
            }
            RedCall::ActivePaneIndex => (),
            RedCall::PaneIsFirst { index } => {
                table.set("index", index)?;
            }
            RedCall::SetActivePane { index } => {
                table.set("index", index)?;
            }
            RedCall::PaneIndexUpFrom { index } => {
                table.set("index", index)?;
            }
            RedCall::PaneIndexDownTo { index, to_first } => {
                table.set("index", index)?;
                table.set("to_first", to_first)?;
            }
            RedCall::BufferInsert { buffer_id, content } => {
                table.set("buffer_id", buffer_id)?;
                table.set("content", content)?;
            }
            RedCall::CurrentBufferId => (),
            RedCall::SetHook {
                hook_name,
                function,
            } => {
                table.set("hook_name", hook_name)?;
                table.set("function", function)?;
            }
            RedCall::RunHook { .. } => {
                Err(mlua::Error::ToLuaConversionError {
                    from: "RedCall::RunHook",
                    to: "Table",
                    message: Some(format!(
                        "RedCall::RunHook cannot be converted between Rust and Lua"
                    )),
                })?;
            }
            RedCall::RunScript { script } => {
                table.set("script", script)?;
            }
            RedCall::BufferDelete { buffer_id, char_count } => {
                table.set("buffer_id", buffer_id)?;
                table.set("char_count", char_count)?;
            }
            RedCall::BufferCursorMoveChar { buffer_id, char_count, move_left } => {
                table.set("buffer_id", buffer_id)?;
                table.set("char_count", char_count)?;
                table.set("move_left", move_left)?;
            }
            RedCall::BufferLength { buffer_id } => {
                table.set("buffer_id", buffer_id)?;
            }
            RedCall::BufferCursorIndex { buffer_id } => {
                table.set("buffer_id", buffer_id)?;
            }
            RedCall::BufferSetCursorIndex { buffer_id, cursor_index } => {
                table.set("buffer_id", buffer_id)?;
                table.set("cursor_index", cursor_index)?;
            }
            RedCall::BufferContent { buffer_id } => {
                table.set("buffer_id", buffer_id)?;
            }
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
                RedCallName::Yield => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, _: ()| Ok(RedCall::Yield))?,
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
                RedCallName::PaneIsFirst => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, index: usize| Ok(RedCall::PaneIsFirst { index }))?,
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
                        lua.create_function(|_, _: ()| Ok(RedCall::CurrentBufferId))?,
                    )?;
                }
                RedCallName::SetHook => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, (hook_name, function): (HookName, Function<'lua>)| {
                                Ok(RedCall::SetHook {
                                    hook_name,
                                    function,
                                })
                            },
                        )?,
                    )?;
                }
                RedCallName::RunHook => { /* RunHook not intended to be a Lua-accessible call */ }

                RedCallName::RunScript => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, script: String| {
                                Ok(RedCall::RunScript {
                                    script
                                })
                            },
                        )?,
                    )?;
                }

                RedCallName::BufferDelete => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, (buffer_id, char_count): (usize, usize)| {
                                Ok(RedCall::BufferDelete {
                                    buffer_id,
                                    char_count,
                                })
                            },
                        )?,
                    )?;
                }
                RedCallName::BufferCursorMoveChar => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, (buffer_id, char_count, move_left): (usize, usize, bool)| {
                                Ok(RedCall::BufferCursorMoveChar {
                                    buffer_id,
                                    char_count,
                                    move_left,
                                })
                            },
                        )?,
                    )?;
                }
                RedCallName::BufferLength => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, buffer_id: usize| {
                                Ok(RedCall::BufferLength {
                                    buffer_id
                                })
                            },
                        )?,
                    )?;
                }
                RedCallName::BufferCursorIndex => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, buffer_id: usize| {
                                Ok(RedCall::BufferCursorIndex {
                                    buffer_id
                                })
                            },
                        )?,
                    )?;
                }
                RedCallName::BufferSetCursorIndex => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, (buffer_id, cursor_index): (usize, usize)| {
                                Ok(RedCall::BufferSetCursorIndex {
                                    buffer_id,
                                    cursor_index,
                                })
                            },
                        )?,
                    )?;
                }
                RedCallName::BufferContent => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(
                            |_, buffer_id: usize| {
                                Ok(RedCall::BufferContent {
                                    buffer_id,
                                })
                            },
                        )?,
                    )?;
                }
            }
        }

        Ok(table)
    }
}

impl ScriptHandler {
    pub fn new(red_script_path: String) -> mlua::Result<Self> {
        let lua = Lua::new();

        let redcall_object = RedCall::lua_object(&lua)?;

        let red_table = lua.create_table()?;
        red_table.set("call", redcall_object)?;

        lua.globals().set("red", red_table)?;

        {
            let package: mlua::Table = lua.globals().get("package")?;
            let current_path: String = package.get("path")?;
            let new_path = format!(
                "{0};{1}/?.lua;{1}/?/init.lua",
                current_path, red_script_path
            );
            package.set("path", new_path)?;
        }

        Ok(Self { lua })
    }

    pub fn run(&self, script: String) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
