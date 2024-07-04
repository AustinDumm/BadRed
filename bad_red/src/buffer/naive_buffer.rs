use std::{io::Read, str::Chars};

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
    fn chars(&self) -> Chars {
        self.content.chars()
    }

    fn content_copy(&self) -> String {
        self.content.clone()
    }

    fn content_char_length(&self) -> usize {
        self.content.len()
    }

    fn set_cursor_byte_index(&mut self, index: usize) {
        self.cursor_byte_index = index;
    }

    fn cursor_byte_index(&self) -> usize {
        self.cursor_byte_index
    }

    fn insert_at_cursor(&mut self, content: &str) {
        if self.cursor_byte_index == self.content.chars().count() {
            self.content.push_str(content);
            //self.content.push_str("👩‍❤️‍👨");
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

    fn move_cursor(&mut self, char_count: isize) {
        self.cursor_byte_index = self
            .shift_byte_cursor_by_character(self.cursor_byte_index, char_count)
            .unwrap_or(0);
        self.cursor_line_index = self.cursor_line_index_for_cursor(self.cursor_byte_index);
    }

    fn move_cursor_line(&mut self, line_count: usize, move_up: bool) {
        if move_up {
            let mut lines_left = line_count;
            let content_chars = self.content.chars().collect::<Vec<_>>();

            let mut index_iter = (0..=self.cursor_byte_index).rev().skip(1);
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

            self.cursor_byte_index = new_index
        } else {
            let mut lines_left = line_count;
            let content_chars = self.content.chars().collect::<Vec<_>>();

            let mut index_iter = self.cursor_byte_index..content_chars.len();
            while let Some(i) = index_iter.next() {
                if content_chars.get(i).map(|c| *c == '\n').unwrap_or(false) {
                    lines_left -= 1;

                    if lines_left == 0 {
                        break;
                    }
                }
            }
            let Some(mut new_index) = index_iter.next() else {
                self.cursor_byte_index = content_chars.len();
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

            self.cursor_byte_index = new_index
        }
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
