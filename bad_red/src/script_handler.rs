// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::path::PathBuf;

use bad_red_proc_macros::{auto_lua_defaulting, auto_script_table};
use mlua::{Function, Lua, Table, Value};

use crate::{
    buffer::EditorBufferType, editor_state::EditorOptionList, hook_map::{HookType, HookTypeName}, styling::Color
};

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
    PaneWrap {
        pane_index: usize,
    },
    PaneSetWrap {
        pane_index: usize,
        should_wrap: bool,
    },
    PaneTopLine {
        pane_index: usize,
    },
    PaneSetTopLine {
        pane_index: usize,
        line: usize,
    },
    PaneFrame {
        pane_index: usize,
    },

    SetHook {
        hook_name: HookTypeName,
        function: Function<'lua>,
        compare: Option<Value<'lua>>,
    },
    RunHook {
        hook: HookType,
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
    BufferCursorMovedByChar {
        buffer_id: usize,
        char_count: isize,
    },
    BufferIndexMovedByChar {
        buffer_id: usize,
        start_byte_index: usize,
        char_count: isize,
    },
    BufferSetType {
        buffer_id: usize,
        buffer_type: EditorBufferType,
    },
    BufferType {
        buffer_id: usize,
    },
    BufferSetCursor {
        buffer_id: usize,
        cursor_index: usize,
        keep_col_index: bool,
    },
    BufferSetCursorLine {
        buffer_id: usize,
        line_index: usize,
    },
    BufferLength {
        buffer_id: usize,
    },
    BufferLineLength {
        buffer_id: usize,
        line_index: usize,
    },
    BufferLineCount {
        buffer_id: usize,
    },
    BufferLineStart {
        buffer_id: usize,
        line_index: usize,
    },
    BufferLineEnd {
        buffer_id: usize,
        line_index: usize,
    },
    BufferCursor {
        buffer_id: usize,
    },
    BufferCursorLine {
        buffer_id: usize,
    },
    BufferContent {
        buffer_id: usize,
    },
    BufferContentAt {
        buffer_id: usize,
        byte_index: usize,
        char_count: usize,
    },
    BufferLineContaining {
        buffer_id: usize,
        byte_index: usize,
    },
    BufferLineContent {
        buffer_id: usize,
        line_index: usize,
    },
    BufferOpen,
    BufferClose {
        buffer_id: usize,
    },
    BufferLinkFile {
        buffer_id: usize,
        file_id: usize,
        should_overwrite_buffer: bool,
    },
    BufferUnlinkFile {
        buffer_id: usize,
        should_force: bool,
    },
    BufferWriteToFile {
        buffer_id: usize,
    },
    BufferCurrentFile {
        buffer_id: usize,
    },
    BufferClearStyle {
        buffer_id: usize,
    },
    BufferPushStyle {
        buffer_id: usize,
        name: String,
        regex: String,
    },

    SetTextStyle {
        name: String,
        background: Option<Color>,
        foreground: Color,
    },

    FileOpen {
        path_string: String,
    },
    FileClose {
        file_id: usize,
        should_force_close: bool,
    },
    FileCurrentBuffer {
        file_id: usize,
    },
    FileExtension {
        file_id: usize,
    },

    Value {
        value: Value<'lua>,
    },

    EditorOptions,
    UpdateOptions {
        option_list: EditorOptionList,
    },
}

impl ScriptHandler {
    pub fn new(red_script_path: PathBuf) -> mlua::Result<Self> {
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
                current_path, red_script_path.to_string_lossy()
            );
            package.set("path", new_path)?;
        }

        Ok(Self { lua })
    }

    pub fn run(&self, script: String) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
