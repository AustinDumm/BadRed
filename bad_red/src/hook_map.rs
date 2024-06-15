// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// 
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::collections::HashMap;

use bad_red_proc_macros::auto_lua;
use mlua::Function;

use crate::keymap::RedKeyEvent;

#[auto_lua]
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Hook {
    KeyEvent(RedKeyEvent),
}

pub struct HookMap<'lua> {
    map: HashMap<HookName, Vec<usize>>,
    hook_functions: Vec<Option<Function<'lua>>>,
}

impl<'lua> HookMap<'lua> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hook_functions: vec![],
        }
    }

    pub fn add_hook(&mut self, hook_name: HookName, function: Function<'lua>) -> usize {
        let new_function_index = self.hook_functions.len();
        self.hook_functions.push(Some(function));
        self.map
            .entry(hook_name)
            .or_insert(vec![])
            .push(new_function_index);

        new_function_index
    }

    pub fn function_iter(&self, hook: HookName) -> Option<impl Iterator<Item = &Function>> {
        Some(
            self.map
                .get(&hook)?
                .iter()
                .filter_map(|i| self.hook_functions.get(*i).and_then(|f| f.as_ref())),
        )
    }
}
