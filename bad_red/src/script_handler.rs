// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::str::FromStr;

use bad_red_proc_macros::auto_lua;
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

#[auto_lua]
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
    PaneIndexDownFrom {
        index: usize,
        to_first: bool,
    },
    PaneType {
        index: usize,
    },
    PaneSetSplitPercent {
        index: usize,
        percent: f32,
    },
    PaneSetSplitFixed {
        index: usize,
        size: u16,
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
        script: String,
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
    },
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
                RedCallName::PaneIndexDownFrom => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (index, to_first): (usize, bool)| {
                            Ok(RedCall::PaneIndexDownFrom { index, to_first })
                        })?,
                    )?;
                }
                RedCallName::PaneType => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, index: usize| Ok(RedCall::PaneType { index }))?,
                    )?;
                }
                RedCallName::PaneSetSplitPercent => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (index, percent): (usize, f32)| {
                            Ok(RedCall::PaneSetSplitPercent { index, percent })
                        })?,
                    )?;
                }
                RedCallName::PaneSetSplitFixed => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (index, size, to_first): (usize, u16, bool)| {
                            Ok(RedCall::PaneSetSplitFixed {
                                index,
                                size,
                                to_first,
                            })
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
                        lua.create_function(|_, script: String| Ok(RedCall::RunScript { script }))?,
                    )?;
                }

                RedCallName::BufferDelete => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (buffer_id, char_count): (usize, usize)| {
                            Ok(RedCall::BufferDelete {
                                buffer_id,
                                char_count,
                            })
                        })?,
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
                        lua.create_function(|_, buffer_id: usize| {
                            Ok(RedCall::BufferLength { buffer_id })
                        })?,
                    )?;
                }
                RedCallName::BufferCursorIndex => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, buffer_id: usize| {
                            Ok(RedCall::BufferCursorIndex { buffer_id })
                        })?,
                    )?;
                }
                RedCallName::BufferSetCursorIndex => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, (buffer_id, cursor_index): (usize, usize)| {
                            Ok(RedCall::BufferSetCursorIndex {
                                buffer_id,
                                cursor_index,
                            })
                        })?,
                    )?;
                }
                RedCallName::BufferContent => {
                    table.set(
                        Into::<&'static str>::into(case),
                        lua.create_function(|_, buffer_id: usize| {
                            Ok(RedCall::BufferContent { buffer_id })
                        })?,
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
