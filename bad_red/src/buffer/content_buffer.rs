// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::io::Read;

use crate::file_handle::FileWrite;

pub trait ContentBuffer {
    fn insert_at_cursor(&mut self, content: &str);
    fn delete_at_cursor(&mut self, char_count: usize) -> String;

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_>;
    fn content_byte_length(&self) -> usize;
    fn content_line_count(&self) -> usize;
    fn content_line_length(&self, line_index: usize) -> Option<usize>;
    fn content_copy(&self) -> String;
    fn content_copy_at_byte_index(&self, byte_index: usize, char_count: usize) -> Option<String>;
    fn content_copy_line(&self, line_index: usize) -> Option<String>;

    fn set_cursor_byte_index(&mut self, index: usize, keep_col_index: bool);
    fn set_cursor_line_index(&mut self, index: usize);
    fn cursor_byte_index(&self) -> usize;
    fn cursor_line_index(&self) -> usize;
    fn line_index_for_byte_index(&self, byte_index: usize) -> usize;
    fn line_start_byte_index(&self, line_index: usize) -> Option<usize>;
    fn line_end_byte_index(&self, line_index: usize) -> Option<usize>;

    fn cursor_moved_by_char(&self, char_count: isize) -> usize;
    fn index_moved_by_char(&self, start_byte_index: usize, char_count: isize) -> usize;

    fn populate_from_read(&mut self, read: &mut dyn Read) -> std::io::Result<()>;
    fn flush_to_write(&mut self, write: &mut dyn FileWrite) -> std::io::Result<()>;
}

pub enum BufferUpdate {
    None,
    Raw,
    Command(String),
}
