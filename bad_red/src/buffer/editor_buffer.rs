// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::io::Read;

use crate::file_handle::FileWrite;

use super::{content_buffer::ContentBuffer, gap_buffer::GapBuffer, naive_buffer::NaiveBuffer};

pub struct EditorBuffer {
    pub content: Box<dyn ContentBuffer>,

    pub is_render_dirty: bool,
    pub is_content_dirty: bool,
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            content: Box::new(GapBuffer::new()),
            is_render_dirty: false,
            is_content_dirty: false,
        }
    }
}

impl ContentBuffer for EditorBuffer {
    fn insert_at_cursor(&mut self, content: &str) {
        self.is_render_dirty = true;
        self.is_content_dirty = true;
        self.content.insert_at_cursor(content);
    }

    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        self.is_render_dirty = true;
        self.is_content_dirty = true;
        self.content.delete_at_cursor(char_count)
    }

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        self.content.chars()
    }

    fn content_byte_length(&self) -> usize {
        self.content.content_byte_length()
    }

    fn content_line_count(&self) -> usize {
        self.content.content_line_count()
    }

    fn content_copy(&self) -> String {
        self.content.content_copy()
    }

    fn content_copy_at_byte_index(&self, byte_index: usize, length: usize,) -> Option<String> {
        self.content.content_copy_at_byte_index(byte_index, length)
    }

    fn content_copy_line(&self, line_index: usize) -> Option<String> {
        self.content.content_copy_line(line_index)
    }

    fn set_cursor_byte_index(&mut self, index: usize, keep_col_index: bool) {
        self.content.set_cursor_byte_index(index, keep_col_index);
    }

    fn set_cursor_line_index(&mut self, index: usize) {
        self.is_render_dirty = true;

        self.content.set_cursor_line_index(index);
    }

    fn cursor_byte_index(&self) -> usize {
        self.content.cursor_byte_index()
    }

    fn cursor_line_index(&self) -> usize {
        self.content.cursor_line_index()
    }

    fn cursor_moved_by_char(&mut self, char_count: isize) -> usize {
        self.is_render_dirty = true;

        self.content.cursor_moved_by_char(char_count)
    }

    fn populate_from_read(&mut self, read: &mut dyn Read) -> std::io::Result<()> {
        self.is_content_dirty = false;
        self.is_render_dirty = true;

        self.content.populate_from_read(read)
    }

    fn flush_to_write(&mut self, write: &mut dyn FileWrite) -> std::io::Result<()> {
        self.is_content_dirty = false;

        self.content.flush_to_write(write)
    }
}
