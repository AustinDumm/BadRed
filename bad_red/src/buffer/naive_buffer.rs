// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::io::Read;

use crate::file_handle::FileWrite;

use super::ContentBuffer;

pub struct NaiveBuffer {
    pub cursor_byte_index: usize,
    pub cursor_line_index: usize,
    pub content: String,
}

impl NaiveBuffer {
    pub fn new() -> Self {
        Self {
            cursor_byte_index: 0,
            cursor_line_index: 0,
            content: String::new(),
        }
    }

    fn cursor_line_index_for_cursor(&self, mut cursor_byte_index: usize) -> usize {
        let chars = self.content.chars().collect::<Vec<_>>();
        let mut line_char_count = 0;

        while chars
            .get(cursor_byte_index)
            .map(|c| *c != '\n')
            .unwrap_or(false)
        {
            line_char_count += 1;

            let Some(new_cursor_byte_index) = cursor_byte_index.checked_sub(1) else {
                break;
            };
            cursor_byte_index = new_cursor_byte_index
        }

        line_char_count
    }

    fn shift_byte_cursor_by_character(
        &self,
        mut byte_index: usize,
        mut cursor_shift_count: isize,
    ) -> Option<usize> {
        let bytes = self.content.as_bytes();
        let current_byte = bytes.get(byte_index);
        if !current_byte
            .map(|b| is_byte_utf8_start_char(*b))
            .unwrap_or(true)
        {
            return None;
        }

        while cursor_shift_count != 0 {
            byte_index = byte_index.checked_add_signed(cursor_shift_count.signum())?;

            if byte_index >= bytes.len() {
                return Some(bytes.len());
            } else if byte_index == 0 {
                return Some(0);
            }

            let current_byte = bytes.get(byte_index)?;
            if is_byte_utf8_start_char(*current_byte) {
                cursor_shift_count -= cursor_shift_count.signum();
            }
        }

        Some(byte_index)
    }
}

impl ContentBuffer for NaiveBuffer {
    fn insert_at_cursor(&mut self, content: &str) {
        if self.cursor_byte_index == self.content.chars().count() {
            self.content.push_str(content);
        } else {
            self.content.insert_str(self.cursor_byte_index, content);
        }
        self.cursor_byte_index += content.as_bytes().len();
        self.cursor_line_index = self.cursor_line_index_for_cursor(self.cursor_byte_index);
    }

    fn delete_at_cursor(&mut self, mut char_count: usize) -> String {
        let bytes = self.content.as_bytes();
        let mut first_non_delete = self.cursor_byte_index;
        loop {
            if first_non_delete >= bytes.len() {
                break;
            }

            if is_byte_utf8_start_char(bytes[first_non_delete]) {
                let Some(new_char_count) = char_count.checked_sub(1) else {
                    break;
                };
                char_count = new_char_count;
            }
            first_non_delete += 1;
        }

        let string_to_delete = self.content[self.cursor_byte_index..first_non_delete].to_string();
        let new_content = format!(
            "{}{}",
            &self.content[..self.cursor_byte_index],
            &self.content[first_non_delete..]
        );
        self.content = new_content;

        string_to_delete
    }

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        Box::new(self.content.chars())
    }

    fn content_byte_length(&self) -> usize {
        self.content.len()
    }

    fn content_line_count(&self) -> usize {
        let mut count = 0;

        for char in self.content.chars() {
            if char == '\n' {
                count += 1
            }
        }

        count + 1
    }

    fn content_copy(&self) -> String {
        self.content.clone()
    }

    fn content_copy_at_byte_index(&self, byte_index: usize, char_count: usize) -> Option<String> {
        if let Some((first_excluded_char_byte_index, _)) = self
            .content
            .char_indices()
            .skip_while(|(char_byte_index, _)| *char_byte_index < byte_index)
            .skip(char_count)
            .next() {
                Some(self.content[byte_index..first_excluded_char_byte_index].to_string())
        } else {
            Some(self.content[byte_index..].to_string())
        }
    }

    fn set_cursor_byte_index(&mut self, index: usize) {
        self.cursor_byte_index = index;

        let mut col_index = 0;
        for (char_index, char) in self.content.char_indices() {
            if char_index == index {
                break;
            }

            col_index += 1;
            if char == '\n' {
                col_index = 0;
            }
        }

        self.cursor_line_index = col_index;
    }

    fn set_cursor_line_index(&mut self, index: usize) {
        let mut newline_count = 0;
        let mut new_byte_index: Option<usize> = None;

        let mut char_iter = self.content.char_indices();
        for (char_index, char) in &mut char_iter {
            if newline_count == index {
                new_byte_index = Some(char_index);
                break;
            }

            if char == '\n' {
                newline_count += 1;
            }
        }

        let new_byte_index = match new_byte_index {
            Some(mut last_byte_index) => {
                let mut line_count = 0;
                for (char_index, char) in &mut char_iter {
                    if line_count == self.cursor_line_index {
                        break;
                    }

                    line_count += 1;
                    if char == '\n' {
                        break;
                    }
                    last_byte_index = char_index;
                }
                last_byte_index
            }
            None => self.content_byte_length(),
        };

        self.cursor_byte_index = new_byte_index;
    }

    fn cursor_byte_index(&self) -> usize {
        self.cursor_byte_index
    }

    fn cursor_line_index(&self) -> usize {
        let mut newline_count = 0;

        for (char_index, char) in self.content.char_indices() {
            if char_index == self.cursor_byte_index {
                break;
            }

            if char == '\n' {
                newline_count += 1;
            }
        }

        newline_count
    }

    fn cursor_moved_by_char(&mut self, char_count: isize) -> usize {
        self.shift_byte_cursor_by_character(self.cursor_byte_index, char_count)
            .unwrap_or(0)
    }

    fn populate_from_read(&mut self, read: &mut dyn Read) -> std::io::Result<()> {
        let mut string = String::new();
        read.read_to_string(&mut string)?;
        self.content = string;
        self.cursor_byte_index = 0;
        self.cursor_line_index = 0;

        Ok(())
    }

    fn flush_to_write(&mut self, write: &mut dyn FileWrite) -> std::io::Result<()> {
        write.write_file(self.content.as_bytes())?;

        Ok(())
    }
}

fn is_byte_utf8_start_char(byte: u8) -> bool {
    let is_single_byte_char = (0b_1000_0000_u8 & byte) == 0;
    if is_single_byte_char {
        return true;
    }

    let multibyte_start_char_mask = 0b_1100_0000;
    return (multibyte_start_char_mask & byte) == multibyte_start_char_mask;
}
