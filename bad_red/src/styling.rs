// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::collections::HashMap;

use bad_red_proc_macros::auto_lua;
use regex::Regex;

pub struct Styling {
    pub style_list: Vec<Style>,
}

impl Styling {
    pub const DEFAULT_NAME: &str = "default";

    pub fn new() -> Self {
        Self { style_list: vec![] }
    }

    pub fn push_style(&mut self, name: String, regex: String) -> Result<(), String> {
        self.style_list.push(Style {
            name,
            regex: Regex::new(&format!("^({})", &regex)).map_err(|e| match e {
                regex::Error::Syntax(reason) => reason,
                regex::Error::CompiledTooBig(size) => {
                    format!("Could not compile regex to size: {}", size)
                }
                _ => format!("Unknown regex faliure"),
            })?,
        });

        Ok(())
    }

    pub fn clear(&mut self) {
        self.style_list.clear();
    }

    pub fn push(&mut self, style: Style) {
        self.style_list.push(style);
    }
}

pub type TextStyleMap = HashMap<String, TextStyle>;

#[auto_lua]
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<&Color> for crossterm::style::Color {
    fn from(value: &Color) -> Self {
        Self::Rgb {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

#[auto_lua]
#[derive(Debug)]
pub struct TextStyle {
    pub background: Option<Color>,
    pub foreground: Color,
}

pub struct Style {
    pub name: String,
    pub regex: Regex,
}
