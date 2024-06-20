// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.


use bad_red_proc_macros::{auto_lua_defaulting, auto_script_table};
use mlua::{Function, Lua, Table};

use crate::hook_map::{Hook, HookName};

pub struct ScriptHandler {
    pub lua: Lua,
}

trait ScriptObject {
    fn lua_object<'lua>(lua: &'lua Lua) -> mlua::Result<Table<'lua>>;
}

#[auto_lua_defaulting]
#[auto_script_table]
#[derive(Default)]
pub enum RedCall<'lua> {
    #[default]
    None,
    Yield,

    EditorExit,
    
    PaneVSplit {
        index: usize,
    },
    PaneHSplit {
        index: usize,
    },
    ActivePaneIndex,
    RootPaneIndex,
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
    PaneBufferIndex {
        index: usize,
    },
    PaneCloseChild {
        index: usize,
        first_child: bool,
    },
    PaneSetBuffer {
        pane_index: usize,
        buffer_index: usize,
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
    BufferOpen,
    BufferClose {
        buffer_id: usize,
    },
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
