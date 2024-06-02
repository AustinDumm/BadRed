// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// 
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::collections::HashMap;
use std::str::FromStr;

use mlua::{FromLua, Function, IntoLua};
use strum_macros::{EnumDiscriminants, EnumString, IntoStaticStr};

use crate::keymap::RedKeyEvent;

#[derive(Clone, Hash, PartialEq, Eq, Debug, EnumString, IntoStaticStr, EnumDiscriminants)]
#[strum(serialize_all = "snake_case")]
#[strum_discriminants(derive(Hash, EnumString, IntoStaticStr))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
#[strum_discriminants(name(HookName))]
pub enum Hook {
    KeyEvent(RedKeyEvent),
}

impl<'lua> FromLua<'lua> for HookName {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        _lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        let hook_name = value
            .as_str()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "LuaValue",
                to: "Hook",
                message: Some(format!("Expected Lua string for Hook. Found: {:?}", value)),
            })?;

        HookName::from_str(hook_name).map_err(|e| mlua::Error::FromLuaConversionError {
            from: "String",
            to: "Hook",
            message: Some(format!("Failed to convert from string to Hook: {}", e)),
        })
    }
}

impl<'lua> IntoLua<'lua> for HookName {
    fn into_lua(
        self,
        lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        let self_string: &'static str = self.into();
        lua.create_string(self_string)?.into_lua(lua)
    }
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
