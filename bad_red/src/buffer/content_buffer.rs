// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{io::Read, str::Chars};

use crate::file_handle::FileWrite;

pub trait ContentBuffer {
    fn insert_at_cursor(&mut self, content: &str);
    fn delete_at_cursor(&mut self, char_count: usize) -> String;

    fn chars(&self) -> Chars;
    fn content_char_length(&self) -> usize;
    fn content_copy(&self) -> String;

    fn set_cursor_byte_index(&mut self, index: usize);
    fn cursor_byte_index(&self) -> usize;

    fn cursor_moved_by_char(&mut self, char_count: isize) -> usize;
    fn cursor_moved_by_line(&mut self, line_count: usize, move_up: bool) -> usize;

    fn populate_from_read(&mut self, read: &mut dyn Read) -> std::io::Result<()>;
    fn flush_to_write(&mut self, write: &mut dyn FileWrite) -> std::io::Result<()>;
}

pub enum BufferUpdate {
    None,
    Raw,
    Command(String),
}

