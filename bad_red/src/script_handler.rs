use std::{cell::RefCell, rc::Rc};

use mlua::{FromLua, IntoLua, Lua, Result, Table, Value};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::editor_state::EditorState;

pub struct ScriptHandler {
    pub state: Rc<RefCell<EditorState>>,
    pub lua: Lua,
}

trait ScriptObject {
    fn lua_object<'lua>(
        lua: &'lua mut Lua,
        state: &Rc<RefCell<EditorState>>,
    ) -> mlua::Result<Table<'lua>>;
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum PaneBuiltIn {
    VSplit,
    HSplit,
}

impl ScriptObject for PaneBuiltIn {
    fn lua_object<'lua>(
        lua: &'lua mut Lua,
        state: &Rc<RefCell<EditorState>>,
    ) -> mlua::Result<Table<'lua>> {
        let mut table = lua.create_table()?;

        for case in Self::iter() {
            match case {
                PaneBuiltIn::VSplit => {
                    let state = state.clone();
                    table.set(
                        Self::V_SPLIT_NAME,
                        lua.create_function(move |_, _: ()| -> mlua::Result<()> {
                            Ok(())
                        })?
                    )?;
                }
                PaneBuiltIn::HSplit => todo!(),
            }
        }

        Ok(table)
    }
}

impl PaneBuiltIn {
    const V_SPLIT_NAME: &'static str = "v_split";
    const H_SPLIT_NAME: &'static str = "h_split";
    fn type_name(&self) -> &str {
        match self {
            PaneBuiltIn::VSplit => Self::V_SPLIT_NAME,
            PaneBuiltIn::HSplit => Self::H_SPLIT_NAME,
        }
    }

    fn from_table(table: &Table) -> Option<Self> {
        let Ok(type_name): mlua::Result<String> = table.get(String::from("type")) else {
            return None;
        };
        match type_name.as_str() {
            Self::V_SPLIT_NAME => Some(PaneBuiltIn::VSplit),
            Self::H_SPLIT_NAME => Some(PaneBuiltIn::HSplit),
            _ => None,
        }
    }
}

impl<'lua> IntoLua<'lua> for PaneBuiltIn {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("type", self.type_name())?;

        Ok(Value::Table(table))
    }
}

impl<'lua> FromLua<'lua> for PaneBuiltIn {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        lua: &'lua Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        match value {
            Value::Table(ref table) => {
                if let Some(built_in) = PaneBuiltIn::from_table(table) {
                    Ok(built_in)
                } else {
                    Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: "BadRed-PaneBuiltIn",
                        message: Some(format!("Found unexpected PaneBuiltIn table: {:#?}", table)),
                    })
                }
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "BadRed-PaneBuiltIn",
                message: Some(format!(
                    "Found non-table type while trying to parse PaneBuiltIn command"
                )),
            }),
        }
    }
}

impl ScriptHandler {
    pub fn new(state: Rc<RefCell<EditorState>>) -> mlua::Result<Self> {
        let mut lua = Lua::new();

        register_builtins(&mut lua)?;

        Ok(Self { lua, state })
    }
}

fn register_builtins(lua: &mut Lua) -> mlua::Result<()> {
    let table = lua.create_table()?;

    table.set("pane", pane_builtins(&lua)?)?;

    lua.globals().set("red", table)
}

fn pane_builtins(lua: &Lua) -> mlua::Result<Table> {
    let mut pane = lua.create_table()?;

    pane.set(
        "vsplit",
        lua.create_function(|_, _: ()| Ok(PaneBuiltIn::VSplit))?,
    )?;
    pane.set(
        "hsplit",
        lua.create_function(|_, _: ()| Ok(PaneBuiltIn::HSplit))?,
    )?;
    Ok(pane)
}
