use mlua::{FromLua, IntoLua, Lua, Result, Table, Value};

pub struct ScriptHandler {
    pub lua: Lua,
}

pub enum BuiltIn {
    VSplit,
    HSplit,
}

impl BuiltIn {
    const V_SPLIT_NAME: &'static str = "v_split";
    const H_SPLIT_NAME: &'static str = "h_split";
    fn type_name(&self) -> &str {
        match self {
            BuiltIn::VSplit => Self::V_SPLIT_NAME,
            BuiltIn::HSplit => Self::H_SPLIT_NAME,
        }
    }

    fn from_table(table: &Table) -> Option<Self> {
        let Ok(type_name): mlua::Result<String> = table.get(String::from("type")) else { return None; };
        match type_name.as_str() {
            Self::V_SPLIT_NAME => Some(BuiltIn::VSplit),
            Self::H_SPLIT_NAME => Some(BuiltIn::HSplit),
            _ => None
        }
    }
}

impl<'lua> IntoLua<'lua> for BuiltIn {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        let table = lua.create_table()?;
        table.set("type", self.type_name())?;

        Ok(Value::Table(table))
    }
}

impl<'lua> FromLua<'lua> for BuiltIn {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        lua: &'lua Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        match value {
            Value::Table(ref table) => {
                if let Some(built_in) = BuiltIn::from_table(table) {
                    Ok(built_in)
                } else {
                    Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: "BadRed-BuiltIn",
                        message: Some(format!("Found unexpected BuiltIn table: {:#?}", table)), 
                    })
                }
            },
            _ => {
                Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "BadRed-BuiltIn",
                    message: Some(format!("Found non-table type while trying to parse BuiltIn command")),
                })
            }
        }
    }
}

impl ScriptHandler {
    pub fn new() -> mlua::Result<Self> {
        let mut lua = Lua::new();

        register_builtins(&mut lua)?;

        Ok(Self { lua })
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
        lua.create_function(|_, _: ()| Ok(BuiltIn::VSplit))?,
    )?;
    pane.set(
        "hsplit",
        lua.create_function(|_, _: ()| Ok(BuiltIn::HSplit))?,
    )?;
    Ok(pane)
}
