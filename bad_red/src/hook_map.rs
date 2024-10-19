// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::collections::HashMap;

use bad_red_proc_macros::auto_lua;
use mlua::{Function, Value};

use crate::keymap::RedKeyEvent;

#[auto_lua]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum HookType {
    KeyEvent(RedKeyEvent),
    PaneBufferChanged(PaneBufferChange),
    BufferFileLinked(BufferFileLink),
    Error(String),
    SecondaryError(String),
    PaneClosed { pane_id: usize },
}

#[auto_lua]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct PaneBufferChange {
    pub pane_id: usize,
    pub buffer_id: usize,
}

#[auto_lua]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum BufferFileLinkType {
    Link,
    Unlink,
}

#[auto_lua]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct BufferFileLink {
    pub link_type: BufferFileLinkType,
    pub buffer_id: usize,
    pub file_id: usize,
}

struct HookMapEntry<'lua> {
    function_index: usize,
    function_compare: Option<Value<'lua>>,
}

pub struct HookMap<'lua> {
    map: HashMap<HookTypeName, Vec<HookMapEntry<'lua>>>,
    hook_functions: Vec<Option<Function<'lua>>>,
}

impl<'lua> HookMap<'lua> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hook_functions: vec![],
        }
    }

    pub fn add_hook(
        &mut self,
        hook_name: HookTypeName,
        function: Function<'lua>,
        compare: Option<Value<'lua>>,
    ) -> usize {
        let new_function_index = self.hook_functions.len();
        self.hook_functions.push(Some(function));
        self.map
            .entry(hook_name)
            .or_insert(vec![])
            .push(HookMapEntry {
                function_index: new_function_index,
                function_compare: compare,
            });

        new_function_index
    }

    pub fn function_iter(
        &'lua self,
        hook: HookTypeName,
        compare: Option<Value<'lua>>,
    ) -> Option<impl Iterator<Item = &Function>> {
        Some(self.map.get(&hook)?.iter().filter_map(move |entry| {
            if entry.function_compare.is_none() || entry.function_compare == compare {
                self.hook_functions
                    .get(entry.function_index)
                    .and_then(|f| f.as_ref())
            } else {
                None
            }
        }))
    }
}
