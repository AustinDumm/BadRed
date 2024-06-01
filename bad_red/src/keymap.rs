use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mlua::{FromLua, IntoLua, UserData, Value};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RedKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl RedKeyEvent {
    pub const CONTROL_PREFIX: &'static str = "C+";
    pub const ALT_PREFIX: &'static str = "A+";
    pub const SUPER_PREFIX: &'static str = "S+";
    pub const HYPER_PREFIX: &'static str = "H+";
    pub const META_PREFIX: &'static str = "M+";

    pub const BACKSPACE_NAME: &'static str = "Backspace";
    pub const ENTER_NAME: &'static str = "Enter";
    pub const LEFT_NAME: &'static str = "Left";
    pub const RIGHT_NAME: &'static str = "Right";
    pub const UP_NAME: &'static str = "Up";
    pub const DOWN_NAME: &'static str = "Down";
    pub const HOME_NAME: &'static str = "Home";
    pub const END_NAME: &'static str = "End";
    pub const PAGE_UP_NAME: &'static str = "PageUp";
    pub const PAGE_DOWN_NAME: &'static str = "PageDown";
    pub const TAB_NAME: &'static str = "Tab";
    pub const BACK_TAB_NAME: &'static str = "BackTab";
    pub const DELETE_NAME: &'static str = "Delete";
    pub const INSERT_NAME: &'static str = "Insert";
    pub const NULL_NAME: &'static str = "Null";
    pub const ESC_NAME: &'static str = "Esc";
    pub const CAPS_LOCK_NAME: &'static str = "CapsLock";
    pub const SCROLL_LOCK_NAME: &'static str = "ScrollLock";
    pub const NUM_LOCK_NAME: &'static str = "NumLock";
    pub const PRINT_SCREEN_NAME: &'static str = "PrintScreen";
    pub const PAUSE_NAME: &'static str = "Pause";
    pub const MENU_NAME: &'static str = "Menu";
    pub const KEYPAD_BEGIN_NAME: &'static str = "KeypadBegin";
    pub const FUNCTION_KEY_PREFIX: &'static str = "F";
}

impl Default for RedKeyEvent {
    fn default() -> Self {
        Self {
            code: KeyCode::Null,
            modifiers: KeyModifiers::NONE,
        }
    }
}

impl From<KeyEvent> for RedKeyEvent {
    fn from(value: KeyEvent) -> Self {
        Self {
            code: value.code,
            modifiers: value.modifiers,
        }
    }
}

impl TryInto<String> for RedKeyEvent {
    type Error = String;

    fn try_into(self) -> Result<String, Self::Error> {
        let key_component = match self.code {
            KeyCode::Backspace => Self::BACKSPACE_NAME.to_string(),
            KeyCode::Enter => Self::ENTER_NAME.to_string(),
            KeyCode::Left => Self::LEFT_NAME.to_string(),
            KeyCode::Right => Self::RIGHT_NAME.to_string(),
            KeyCode::Up => Self::UP_NAME.to_string(),
            KeyCode::Down => Self::DOWN_NAME.to_string(),
            KeyCode::Home => Self::HOME_NAME.to_string(),
            KeyCode::End => Self::END_NAME.to_string(),
            KeyCode::PageUp => Self::PAGE_UP_NAME.to_string(),
            KeyCode::PageDown => Self::PAGE_DOWN_NAME.to_string(),
            KeyCode::Tab => Self::TAB_NAME.to_string(),
            KeyCode::BackTab => Self::BACK_TAB_NAME.to_string(),
            KeyCode::Delete => Self::DELETE_NAME.to_string(),
            KeyCode::Insert => Self::INSERT_NAME.to_string(),
            KeyCode::F(index) => format!("{}{}", Self::FUNCTION_KEY_PREFIX, index),
            KeyCode::Char(c) => format!(
                "{}",
                if self.modifiers.contains(KeyModifiers::SHIFT) {
                    c.to_ascii_uppercase()
                } else {
                    c.to_ascii_lowercase()
                }
            ),
            KeyCode::Null => Self::NULL_NAME.to_string(),
            KeyCode::Esc => Self::ESC_NAME.to_string(),
            KeyCode::CapsLock => Self::CAPS_LOCK_NAME.to_string(),
            KeyCode::ScrollLock => Self::SCROLL_LOCK_NAME.to_string(),
            KeyCode::NumLock => Self::NUM_LOCK_NAME.to_string(),
            KeyCode::PrintScreen => Self::PRINT_SCREEN_NAME.to_string(),
            KeyCode::Pause => Self::PAUSE_NAME.to_string(),
            KeyCode::Menu => Self::MENU_NAME.to_string(),
            KeyCode::KeypadBegin => Self::KEYPAD_BEGIN_NAME.to_string(),
            KeyCode::Media(key) => {
                return Ok(format!(
                    "Unexpected KeyCode type for Lua conversion: Media({:#?})",
                    key
                ))
            }
            KeyCode::Modifier(modifier) => {
                return Ok(format!(
                    "Unexpected KeyCode type for Lua conversion: Modifier({:#?})",
                    modifier
                ))
            }
        };

        let mut modifier_component = "".to_string();
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            modifier_component += Self::CONTROL_PREFIX
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            modifier_component += Self::ALT_PREFIX
        }
        if self.modifiers.contains(KeyModifiers::SUPER) {
            modifier_component += Self::SUPER_PREFIX
        }
        if self.modifiers.contains(KeyModifiers::HYPER) {
            modifier_component += Self::HYPER_PREFIX
        }
        if self.modifiers.contains(KeyModifiers::META) {
            modifier_component += Self::META_PREFIX
        };

        Ok(format!("{}{}", modifier_component, key_component))
    }
}

impl<'lua> IntoLua<'lua> for RedKeyEvent {
    fn into_lua(
        self,
        lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        let string: String = self
            .try_into()
            .map_err(|e| mlua::Error::ToLuaConversionError {
                from: "RedKeyEvent",
                to: "String",
                message: Some(e),
            })?;

        lua.create_string(string)?.into_lua(lua)
    }
}

impl<'lua> FromLua<'lua> for RedKeyEvent {
    fn from_lua(
        value: mlua::prelude::LuaValue<'lua>,
        lua: &'lua mlua::prelude::Lua,
    ) -> mlua::prelude::LuaResult<Self> {
        let Value::String(key_string) = value else {
            return Err(mlua::Error::FromLuaConversionError {
                from: "String",
                to: "RedKeyEvent",
                message: Some(format!(
                    "Expected String for RedKeyValue. Found: {:#?}",
                    value
                )),
            });
        };
        let key_string = key_string.to_str()?;

        RedKeyEvent::try_from(key_string).map_err(|e| mlua::Error::FromLuaConversionError {
            from: "String",
            to: "RedKeyEvent",
            message: Some(e),
        })
    }
}

impl TryFrom<&str> for RedKeyEvent {
    type Error = String;

    fn try_from(key_string: &str) -> Result<Self, Self::Error> {
        let mut modifiers = KeyModifiers::NONE;
        if key_string.contains(Self::CONTROL_PREFIX) {
            modifiers.insert(KeyModifiers::CONTROL)
        }
        if key_string.contains(Self::ALT_PREFIX) {
            modifiers.insert(KeyModifiers::ALT)
        }
        if key_string.contains(Self::SUPER_PREFIX) {
            modifiers.insert(KeyModifiers::SUPER)
        }
        if key_string.contains(Self::HYPER_PREFIX) {
            modifiers.insert(KeyModifiers::HYPER)
        }
        if key_string.contains(Self::META_PREFIX) {
            modifiers.insert(KeyModifiers::META)
        }

        let raw_key_code = key_string
            .split('+')
            .next_back()
            .ok_or_else(|| format!("Could not find key code for RedKeyEvent: {}", key_string))?;

        let code = match raw_key_code {
            Self::BACKSPACE_NAME => KeyCode::Backspace,
            Self::ENTER_NAME => KeyCode::Enter,
            Self::LEFT_NAME => KeyCode::Left,
            Self::RIGHT_NAME => KeyCode::Right,
            Self::UP_NAME => KeyCode::Up,
            Self::DOWN_NAME => KeyCode::Down,
            Self::HOME_NAME => KeyCode::Home,
            Self::END_NAME => KeyCode::End,
            Self::PAGE_UP_NAME => KeyCode::PageUp,
            Self::PAGE_DOWN_NAME => KeyCode::PageDown,
            Self::TAB_NAME => KeyCode::Tab,
            Self::BACK_TAB_NAME => KeyCode::BackTab,
            Self::DELETE_NAME => KeyCode::Delete,
            Self::INSERT_NAME => KeyCode::Insert,
            Self::NULL_NAME => KeyCode::Null,
            Self::ESC_NAME => KeyCode::Esc,
            Self::CAPS_LOCK_NAME => KeyCode::CapsLock,
            Self::SCROLL_LOCK_NAME => KeyCode::ScrollLock,
            Self::NUM_LOCK_NAME => KeyCode::NumLock,
            Self::PRINT_SCREEN_NAME => KeyCode::PrintScreen,
            Self::PAUSE_NAME => KeyCode::Pause,
            Self::MENU_NAME => KeyCode::Menu,
            Self::KEYPAD_BEGIN_NAME => KeyCode::KeypadBegin,
            raw_key_code
                if let Some(index) = raw_key_code.strip_prefix(Self::FUNCTION_KEY_PREFIX) =>
            {
                KeyCode::F(u8::from_str_radix(index, 10).map_err(|e| {
                    format!(
                        "Failed to convert Function key index to number. Found: F{}. Reason: {}",
                        index, e
                    )
                })?)
            }
            key => {
                let mut chars = key.chars();
                let char = chars
                    .next()
                    .ok_or_else(|| format!("Found a key string without any key code"))?;
                if chars.clone().count() != 0 {
                    Err(format!(
                        "Found an unexpected key string with more than one character: {}",
                        chars.collect::<String>()
                    ))?
                } else {
                    if char.is_ascii_uppercase() {
                        modifiers.insert(KeyModifiers::SHIFT);
                    }
                    KeyCode::Char(char)
                }
            }
        };

        Ok(Self { code, modifiers })
    }
}

pub enum KeyMapNode<'lua> {
    Map(Box<KeyMap<'lua>>),
    Function(mlua::Function<'lua>),
}

pub struct KeyMap<'lua> {
    map: HashMap<RedKeyEvent, KeyMapNode<'lua>>,
    pub fallback: Option<KeyMapNode<'lua>>,
}

impl<'lua> KeyMap<'lua> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            fallback: None,
        }
    }

    pub fn with_fallback(mut self, fallback: Option<KeyMapNode<'lua>>) -> Self {
        self.fallback = fallback;
        self
    }
}

impl KeyMap<'_> {
    pub fn node_for(&self, event: &RedKeyEvent) -> Option<&KeyMapNode> {
        self.map.get(&event).or(self.fallback.as_ref())
    }
}
