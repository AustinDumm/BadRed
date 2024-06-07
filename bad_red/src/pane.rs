// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use mlua::{FromLua, IntoLua, Value};
use strum_macros::{EnumDiscriminants, EnumString, IntoStaticStr};

pub type Result<T> = std::result::Result<T, String>;

pub struct PaneTree {
    pub tree: Vec<PaneNode>,
}

impl PaneTree {
    pub fn new(initial_buffer_id: usize) -> Self {
        Self {
            tree: vec![PaneNode {
                node_type: PaneNodeType::Leaf(Pane::new(initial_buffer_id)),
                parent_index: None,
            }],
        }
    }

    pub fn pane_node_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a PaneNode> {
        self.tree.get(pane_index)
    }

    pub fn pane_node_mut_by_index<'a>(&'a mut self, pane_index: usize) -> Option<&'a mut PaneNode> {
        self.tree.get_mut(pane_index)
    }

    pub fn pane_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a Pane> {
        self.tree
            .get(pane_index)
            .map(|node| match &node.node_type {
                PaneNodeType::Leaf(pane) => Some(pane),
                PaneNodeType::VSplit(_) | PaneNodeType::HSplit(_) => None,
            })
            .flatten()
    }

    pub fn vsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<usize> {
        self.split(pane_id, new_pane_buffer, |left, right, split_percentage| {
            PaneNodeType::VSplit(Split {
                first: left,
                second: right,
                split_type: SplitType::Percent {
                    first_percent: split_percentage,
                },
            })
        })
    }

    pub fn hsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<usize> {
        self.split(pane_id, new_pane_buffer, |top, bottom, split_percentage| {
            PaneNodeType::HSplit(Split {
                first: top,
                second: bottom,
                split_type: SplitType::Percent {
                    first_percent: split_percentage,
                },
            })
        })
    }

    fn split(
        &mut self,
        pane_id: usize,
        new_pane_buffer: usize,
        split_constructor: impl FnOnce(usize, usize, f32) -> PaneNodeType,
    ) -> Result<usize> {
        let new_content_pane_index = self.tree.len();
        let moved_content_pane_index = self.tree.len() + 1;

        let current = self.tree.get_mut(pane_id).ok_or_else(|| {
            format!(
                "Failed to find pane for current id while splitting: {}",
                pane_id
            )
        })?;
        let current_parent = current.parent_index;
        current.parent_index = Some(pane_id);

        let new_content_pane = PaneNode {
            node_type: PaneNodeType::Leaf(Pane {
                top_line: 0,
                buffer_id: new_pane_buffer,
            }),
            parent_index: Some(pane_id),
        };

        let new_split_pane = PaneNode {
            node_type: split_constructor(moved_content_pane_index, new_content_pane_index, 0.5),
            parent_index: current_parent,
        };
        self.tree.push(new_content_pane);
        self.tree.push(new_split_pane);
        self.tree.swap(pane_id, moved_content_pane_index);

        Ok(moved_content_pane_index)
    }
}

pub struct PaneNode {
    pub node_type: PaneNodeType,
    pub parent_index: Option<usize>,
}

impl PaneNode {
    pub fn set_type(&mut self, node_type: PaneNodeType) {
        self.node_type = node_type
    }
}

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(name(PaneNodeTypeName))]
#[strum_discriminants(derive(EnumString, IntoStaticStr))]
pub enum PaneNodeType {
    Leaf(Pane),
    VSplit(Split),
    HSplit(Split),
}

impl<'lua> FromLua<'lua> for PaneNodeTypeName {
    fn from_lua(
        value: Value<'lua>,
        _lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        value
            .as_str()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "PaneNodeTypeName",
                message: Some(format!(
                    "Expected String type for PaneNodeTypeName. Found: {:?}",
                    value
                )),
            })?
            .try_into()
            .map_err(|e| mlua::Error::FromLuaConversionError {
                from: "String",
                to: "PaneNodeTypeName",
                message: Some(format!("{}", e)),
            })
    }
}

impl<'lua> IntoLua<'lua> for PaneNodeTypeName {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let string: &str = self.into();
        lua.create_string(string)?.into_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for PaneNodeType {
    fn from_lua(
        value: Value<'lua>,
        _lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "PaneNodeType",
                message: Some(format!(
                    "Expected table type when converting from Lua. Found: {:?}",
                    value
                )),
            })?;

        let node_type = match table.get::<&str, PaneNodeTypeName>("type")? {
            PaneNodeTypeName::Leaf => PaneNodeType::Leaf(
                table.get::<&str, Pane>("pane")?
            ),
            PaneNodeTypeName::VSplit => PaneNodeType::VSplit(table.get::<&str, Split>("split")?),
            PaneNodeTypeName::HSplit => PaneNodeType::HSplit(table.get::<&str, Split>("split")?),
        };

        Ok(node_type)
    }
}

impl<'lua> IntoLua<'lua> for PaneNodeType {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let table = lua.create_table()?;

        let pane_type_name: PaneNodeTypeName = self.clone().into();
        table.set("type", pane_type_name)?;

        match self {
            PaneNodeType::Leaf(pane) => {
                table.set("pane", pane)?
            }
            PaneNodeType::VSplit(split) |
            PaneNodeType::HSplit(split) => {
                table.set("split", split)?
            }
        }

        table.into_lua(lua)
    }
}

#[derive(Clone, Debug)]
pub struct Split {
    pub first: usize,
    pub second: usize,
    pub split_type: SplitType,
}

impl<'lua> FromLua<'lua> for Split {
    fn from_lua(
        value: Value<'lua>,
        _lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "Split",
                message: Some(format!(
                    "Expected table value for Split from Lua. Found: {:?}",
                    value
                )),
            })?;

        Ok(Split {
            first: table.get::<&str, usize>("first")?,
            second: table.get::<&str, usize>("second")?,
            split_type: table.get::<&str, SplitType>("split_type")?,
        })
    }
}

impl<'lua> IntoLua<'lua> for Split {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let table = lua.create_table()?;
        table.set("first", self.first)?;
        table.set("second", self.second)?;
        table.set("split_type", self.split_type)?;
        table.into_lua(lua)
    }
}

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(name(SplitTypeName))]
#[strum_discriminants(derive(EnumString, IntoStaticStr))]
pub enum SplitType {
    Percent { first_percent: f32 },
    FirstFixed { size: u16 },
    SecondFixed { size: u16 },
}

impl<'lua> FromLua<'lua> for SplitTypeName {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        _lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        value
            .as_str()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "SplitTypeName",
                message: Some(format!(
                    "Expected string type for SplitTypeName. Found: {:?}",
                    value
                )),
            })?
            .try_into()
            .map_err(|e| mlua::Error::FromLuaConversionError {
                from: "String",
                to: "SplitTypeName",
                message: Some(format!("{}", e)),
            })
    }
}

impl<'lua> FromLua<'lua> for SplitType {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        _lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        let table = value
            .as_table()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "SplitType",
                message: Some(format!(
                    "Expected table for SplitType lua representation. Found: {:?}",
                    value
                )),
            })?;

        let split = match table.get::<&str, SplitTypeName>("type")? {
            SplitTypeName::Percent => SplitType::Percent {
                first_percent: table.get::<&str, f32>("first_percent").map_err(|e| {
                    mlua::Error::FromLuaConversionError {
                        from: "Table",
                        to: "SplitType",
                        message: Some(format!(
                            "Could not retreive f32 percentage value under key 'first_percent'. {}", e
                        )),
                    }
                })?,
            },
            SplitTypeName::FirstFixed => SplitType::FirstFixed {
                size: table.get::<&str, u16>("size").map_err(|e| {
                    mlua::Error::FromLuaConversionError {
                        from: "Table",
                        to: "SplitType",
                        message: Some(format!(
                            "Could not retreive u16 size under key 'size' for FirstFixed split. {}", e
                        )),
                    }
                })?,
            },
            SplitTypeName::SecondFixed => SplitType::SecondFixed {
                size: table.get::<&str, u16>("size").map_err(|e| {
                    mlua::Error::FromLuaConversionError {
                        from: "Table",
                        to: "SplitType",
                        message: Some(format!(
                            "Could not retreive u16 size under key 'size' for FirstSecond split. {}", e
                        )),
                    }
                })?,
            },
        };

        Ok(split)
    }
}

impl<'lua> IntoLua<'lua> for SplitTypeName {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let string: &str = self.into();
        lua.create_string(string)?.into_lua(lua)
    }
}

impl<'lua> IntoLua<'lua> for SplitType {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let table = lua.create_table()?;

        match self {
            SplitType::Percent { first_percent } => {
                table.set("first_percent", first_percent)?;
            }
            SplitType::FirstFixed { size } | SplitType::SecondFixed { size } => {
                table.set("size", size)?;
            }
        }
        let name: SplitTypeName = self.into();
        table.set("type", name)?;

        table.into_lua(lua)
    }
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub top_line: u16,
    pub buffer_id: usize,
}

impl<'lua> FromLua<'lua> for Pane {
    fn from_lua(value: Value<'lua>, _lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Self> {
        let table = value.as_table()
            .ok_or_else(|| mlua::Error::FromLuaConversionError {
                from: "Value",
                to: "Pane",
                message: Some(format!("Expected table to convert from Lua for Pane. Found: {:?}", value)),
            })?;

        Ok(Pane {
            top_line: table.get::<&str, u16>("top_line")?,
            buffer_id: table.get::<&str, usize>("buffer_id")?,
        })
    }
}

impl<'lua> IntoLua<'lua> for Pane {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        let table = lua.create_table()?;
        table.set("top_line", self.top_line)?;
        table.set("buffer_id", self.buffer_id)?;
        table.into_lua(lua)
    }
}

impl Pane {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            top_line: 0,
            buffer_id,
        }
    }
}
