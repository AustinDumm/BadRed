use std::{io::Read, str::Chars};

use crate::file_handle::FileWrite;

use super::{content_buffer::ContentBuffer, naive_buffer::NaiveBuffer};

pub struct EditorBuffer {
    pub content: Box<dyn ContentBuffer>,

    pub is_render_dirty: bool,
    pub is_content_dirty: bool,
}

impl EditorBuffer {
    pub fn new() -> Self {
        Self {
            content: Box::new(NaiveBuffer::new()),
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

    fn chars(&self) -> Chars {
        self.content.chars()
    }

    fn content_char_length(&self) -> usize {
        self.content.content_char_length()
    }

    fn content_copy(&self) -> String {
        self.content.content_copy()
    }

    fn set_cursor_byte_index(&mut self, index: usize) {
        self.content.set_cursor_byte_index(index);
    }

    fn cursor_byte_index(&self) -> usize {
        self.content.cursor_byte_index()
    }

    fn cursor_moved_by_char(&mut self, char_count: isize) -> usize {
        self.is_render_dirty = true;

        self.content.cursor_moved_by_char(char_count)
    }

    fn cursor_moved_by_line(&mut self, line_count: usize, move_up: bool) -> usize {
        self.is_render_dirty = true;

        self.content.cursor_moved_by_line(line_count, move_up)
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
