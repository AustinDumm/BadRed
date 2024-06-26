// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::io::Read;

use crate::file_handle::FileWrite;

pub struct Buffer {
    pub title: String,
    pub cursor_index: usize,
    pub cursor_line_index: usize,
    pub content: String,

    pub is_render_dirty: bool,
    pub is_content_dirty: bool,
}

pub enum BufferUpdate {
    None,
    Raw,
    Command(String),
}

impl Buffer {
    pub fn new(title: String) -> Self {
        Self {
            title,
            cursor_index: 0,
            cursor_line_index: 0,
            content: String::new(),
            is_render_dirty: false,
            is_content_dirty: false,
        }
    }

    pub fn insert_at_cursor(&mut self, content: &str) {
        if self.cursor_index == self.content.chars().count() {
            self.content.push_str(content);
        } else {
            self.content.insert_str(self.cursor_index, content);
        }
        self.cursor_index += content.len();
        self.cursor_line_index = self.cursor_line_index_for_cursor(self.cursor_index);
        self.is_render_dirty = true;
        self.is_content_dirty = true;
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

    pub fn delete_at_cursor(&mut self, char_count: usize) -> String {
        let first_non_delete = (self.cursor_index + char_count).min(self.content.len());
        let string_to_delete = self.content[self.cursor_index..first_non_delete].to_string();
        let new_content = format!(
            "{}{}",
            &self.content[..self.cursor_index],
            &self.content[first_non_delete..]
        );
        self.content = new_content;
        self.is_render_dirty = true;
        self.is_content_dirty = true;

        string_to_delete
    }

    pub fn move_cursor(&mut self, char_count: usize, move_left: bool) {
        self.cursor_index = if move_left {
            self.cursor_index.saturating_sub(char_count)
        } else {
            self.cursor_index
                .saturating_add(char_count)
                .min(self.content.len())
        };

        self.cursor_line_index = self.cursor_line_index_for_cursor(self.cursor_index);

        self.is_render_dirty = true;
    }

    pub fn move_cursor_line(&mut self, line_count: usize, move_up: bool) {
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

    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }

    pub fn cursor_content_index(&self) -> usize {
        self.cursor_index
    }

    pub fn set_cursor_content_index(&mut self, index: usize) {
        self.cursor_index = index.min(self.content_length());
    }

    pub fn populate_from_read(&mut self, read: &mut impl Read) -> std::io::Result<()> {
        let mut string = String::new();
        read.read_to_string(&mut string)?;
        self.content = string;
        self.cursor_index = 0;
        self.cursor_line_index = 0;

        self.is_content_dirty = false;
        self.is_render_dirty = true;

        Ok(())
    }

    pub fn flush_to_write(&mut self, write: &mut impl FileWrite) -> std::io::Result<()> {
        write.write_file(self.content.as_bytes())?;
        self.is_content_dirty = false;

        Ok(())
    }
}
