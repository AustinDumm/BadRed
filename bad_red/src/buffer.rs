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
    fn content_length(&self) -> usize;
    fn content_copy(&self) -> String;

    fn set_cursor_char_index(&mut self, index: usize);
    fn cursor_char_index(&self) -> usize;

    fn move_cursor(&mut self, char_count: usize, move_left: bool);
    fn move_cursor_line(&mut self, line_count: usize, move_up: bool);

    fn populate_from_read(&mut self, read: &mut dyn Read) -> std::io::Result<()>;
    fn flush_to_write(&mut self, write: &mut dyn FileWrite) -> std::io::Result<()>;
}

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

    fn content_length(&self) -> usize {
        self.content.content_length()
    }

    fn content_copy(&self) -> String {
        self.content.content_copy()
    }

    fn set_cursor_char_index(&mut self, index: usize) {
        self.content.set_cursor_char_index(index);
    }

    fn cursor_char_index(&self) -> usize {
        self.content.cursor_char_index()
    }

    fn move_cursor(&mut self, char_count: usize, move_left: bool) {
        self.is_render_dirty = true;

        self.content.move_cursor(char_count, move_left)
    }

    fn move_cursor_line(&mut self, line_count: usize, move_up: bool) {
        self.is_render_dirty = true;

        self.content.move_cursor_line(line_count, move_up);
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

pub struct NaiveBuffer {
    pub cursor_index: usize,
    pub cursor_line_index: usize,
    pub content: String,
}

pub enum BufferUpdate {
    None,
    Raw,
    Command(String),
}

impl ContentBuffer for NaiveBuffer {
    fn chars(&self) -> Chars {
        self.content.chars()
    }

    fn content_copy(&self) -> String {
        self.content.clone()
    }

    fn content_length(&self) -> usize {
        self.content.len()
    }

    fn cursor_char_index(&self) -> usize {
        self.cursor_index
    }

    fn set_cursor_char_index(&mut self, index: usize) {
        self.cursor_index = index.min(self.content_length());
    }

    fn insert_at_cursor(&mut self, content: &str) {
        if self.cursor_index == self.content.chars().count() {
            self.content.push_str(content);
        } else {
            self.content.insert_str(self.cursor_index, content);
        }
        self.cursor_index += content.len();
        self.cursor_line_index = self.cursor_line_index_for_cursor(self.cursor_index);
    }

    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        let first_non_delete = (self.cursor_index + char_count).min(self.content.len());
        let string_to_delete = self.content[self.cursor_index..first_non_delete].to_string();
        let new_content = format!(
            "{}{}",
            &self.content[..self.cursor_index],
            &self.content[first_non_delete..]
        );
        self.content = new_content;

        string_to_delete
    }

    fn move_cursor(&mut self, char_count: usize, move_left: bool) {
        self.cursor_index = if move_left {
            self.cursor_index.saturating_sub(char_count)
        } else {
            self.cursor_index
                .saturating_add(char_count)
                .min(self.content.len())
        };

        self.cursor_line_index = self.cursor_line_index_for_cursor(self.cursor_index);
    }

    fn move_cursor_line(&mut self, line_count: usize, move_up: bool) {
        if move_up {
            let mut lines_left = line_count;
            let content_chars = self.content.chars().collect::<Vec<_>>();

            let mut index_iter = (0..=self.cursor_index).rev().skip(1);
            while let Some(i) = index_iter.next() {
                if content_chars.get(i).map(|c| *c == '\n').unwrap_or(false) {
                    if let Some(l) = lines_left.checked_sub(1) {
                        lines_left = l
                    } else {
                        break;
                    }
                }
            }

            let mut new_index = index_iter.next().map(|i| i + 2).unwrap_or(0);
            let mut current_line_index = 0;

            while let Some(c) = content_chars.get(new_index) {
                if *c == '\n'
                    || self
                        .cursor_line_index
                        .checked_sub(1)
                        .map(|i| i == current_line_index)
                        .unwrap_or(false)
                {
                    break;
                }

                new_index += 1;
                current_line_index += 1;
            }

            self.cursor_index = new_index
        } else {
            let mut lines_left = line_count;
            let content_chars = self.content.chars().collect::<Vec<_>>();

            let mut index_iter = self.cursor_index..content_chars.len();
            while let Some(i) = index_iter.next() {
                if content_chars.get(i).map(|c| *c == '\n').unwrap_or(false) {
                    lines_left -= 1;

                    if lines_left == 0 {
                        break;
                    }
                }
            }
            let Some(mut new_index) = index_iter.next() else {
                self.cursor_index = content_chars.len();
                return;
            };

            let mut current_line_index = 0;
            while let Some(c) = content_chars.get(new_index) {
                if *c == '\n'
                    || self
                        .cursor_line_index
                        .checked_sub(1)
                        .map(|i| i == current_line_index)
                        .unwrap_or(false)
                {
                    break;
                }

                new_index += 1;
                current_line_index += 1;
            }

            self.cursor_index = new_index
        }
    }
    fn populate_from_read(&mut self, read: &mut dyn Read) -> std::io::Result<()> {
        let mut string = String::new();
        read.read_to_string(&mut string)?;
        self.content = string;
        self.cursor_index = 0;
        self.cursor_line_index = 0;

        Ok(())
    }

    fn flush_to_write(&mut self, write: &mut dyn FileWrite) -> std::io::Result<()> {
        write.write_file(self.content.as_bytes())?;

        Ok(())
    }
}

impl NaiveBuffer {
    pub fn new() -> Self {
        Self {
            cursor_index: 0,
            cursor_line_index: 0,
            content: String::new(),
        }
    }

    fn cursor_line_index_for_cursor(&self, mut cursor_index: usize) -> usize {
        let chars = self.content.chars().collect::<Vec<_>>();
        let mut line_char_count = 0;

        while chars.get(cursor_index).map(|c| *c != '\n').unwrap_or(false) {
            line_char_count += 1;

            let Some(new_cursor_index) = cursor_index.checked_sub(1) else {
                break;
            };
            cursor_index = new_cursor_index
        }

        line_char_count
    }
}
