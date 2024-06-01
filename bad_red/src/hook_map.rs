use std::collections::HashMap;
use std::str::FromStr;

use mlua::{FromLua, Function, IntoLua};
use strum_macros::{EnumString, IntoStaticStr};

#[derive(Clone, Hash, PartialEq, Eq, Debug, EnumString, IntoStaticStr)]
pub enum Hook {
    KeyEvent,
}

impl<'lua> FromLua<'lua> for Hook {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        let hook_name = value
            .as_str()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "LuaValue",
                to: "Hook",
                message: Some(format!("Expected Lua string for Hook. Found: {:?}", value)),
            })?;

        Hook::from_str(hook_name).map_err(|e| mlua::Error::FromLuaConversionError {
            from: "String",
            to: "Hook",
            message: Some(format!("Failed to convert from string to Hook: {}", e)),
        })
    }
}

impl<'lua> IntoLua<'lua> for Hook {
    fn into_lua(
        self,
        lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        let self_string: &'static str = self.into();
        lua.create_string(self_string)?
            .into_lua(lua)
    }
}

pub struct HookMap<'lua> {
    map: HashMap<Hook, Vec<usize>>,
    hook_functions: Vec<Option<Function<'lua>>>,
}

impl<'lua> HookMap<'lua> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hook_functions: vec![],
        }
    }

    pub fn add_hook(&mut self, hook: Hook, function: Function<'lua>) -> usize {
        let new_function_index = self.hook_functions.len();
        self.hook_functions.push(Some(function));
        self.map
            .entry(hook)
            .or_insert(vec![])
            .push(new_function_index);

        new_function_index
    }
}
