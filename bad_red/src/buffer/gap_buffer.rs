// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

#![allow(unused_variables)]

use std::ops::Range;

use bad_gap::GapBuffer as UnderlyingBuf;

use super::{byte_char_iter::ByteCharIter, ContentBuffer};

pub struct GapBuffer {
    underlying_buf: UnderlyingBuf<u8>,
    sorted_newline_indices: Vec<usize>,

    char_col_index: usize,
    line_index: usize,
}

impl GapBuffer {
    pub fn new() -> Self {
        Self {
            underlying_buf: UnderlyingBuf::new(),
            sorted_newline_indices: Vec::new(),
            char_col_index: 0,
            line_index: 0,
        }
    }

    fn lookup_index_of_preceeding_newline(&self, byte_index: usize) -> Option<usize> {
        let found = self.sorted_newline_indices.binary_search(&byte_index);

        match found {
            Ok(0) =>
            // Being on the index of the first newline means there is no preceeding newline
            {
                None
            }
            Ok(found_idx) => Some(found_idx - 1),
            Err(expected_idx) => {
                if expected_idx > 0 {
                    Some(expected_idx - 1)
                } else {
                    None
                }
            }
        }
    }

    fn char_count_in(&self, range: Range<usize>) -> u32 {
        let mut char_count = 0;

        let mut byte_index = range.start;
        while byte_index < range.end {
            let byte = self.underlying_buf[byte_index];
            match super::expected_byte_length_from_starting(byte) {
                None => panic!("Attempted to find char count in range with invalid utf8 encoding"),
                Some(length) => {
                    byte_index += length as usize;
                    char_count += 1;
                }
            }
        }

        char_count
    }

    pub fn populate_from_vec(&mut self, vec: &[u8]) {
        self.underlying_buf = UnderlyingBuf::from(vec);

        let mut char_byte_index = 0;
        let mut newline_indices = vec![];
        for char in self.chars() {
            if char == '\n' {
                newline_indices.push(char_byte_index);
            }

            char_byte_index += char.len_utf8();
        }

        self.sorted_newline_indices = newline_indices;
    }
}

impl ContentBuffer for GapBuffer {
    fn insert_at_cursor(&mut self, content: &str) {
        let cursor_byte_index = self.cursor_byte_index();

        let content_bytes = content.as_bytes();
        for byte in content_bytes {
            self.underlying_buf.push_before_cursor(*byte);
        }

        for newline_index in self
            .sorted_newline_indices
            .iter_mut()
            .filter(|i| **i >= cursor_byte_index)
        {
            *newline_index += content_bytes.len();
        }

        for (content_index, content_char) in content.char_indices() {
            if content_char == '\n' {
                let newline_index = content_index + cursor_byte_index;
                let insert_index = self
                    .sorted_newline_indices
                    .partition_point(|i| *i < newline_index);
                self.sorted_newline_indices
                    .insert(insert_index, newline_index);
                self.char_col_index = 0;
                self.line_index += 1;
            } else {
                self.char_col_index += 1;
            }
        }
    }

    #[cfg(debug_assertions)]
    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        use std::{collections::VecDeque, mem};

        let cursor_byte_index = self.cursor_byte_index();

        let mut bytes_to_remove = 0;
        let mut chars_remaining = char_count;
        while chars_remaining > 0 {
            let Some(byte) = self.underlying_buf.get(cursor_byte_index + bytes_to_remove) else {
                break;
            };

            match super::expected_byte_length_from_starting(*byte) {
                Some(length) => {
                    bytes_to_remove += length as usize;
                    chars_remaining -= 1;
                }
                None => panic!("Found invalid utf8 encoded bytes while deleting at cursor"),
            }
        }

        let bytes_to_remove = bytes_to_remove;
        let first_remove_index = self
            .sorted_newline_indices
            .partition_point(|i| *i < cursor_byte_index);
        let first_unremove_index = self
            .sorted_newline_indices
            .partition_point(|i| *i < (cursor_byte_index + bytes_to_remove));

        let old_newlines = mem::take(&mut self.sorted_newline_indices);

        let (kept_newlines, removed_newlines): (Vec<_>, Vec<_>) = old_newlines
            .into_iter()
            .enumerate()
            .partition(|(i, newline_index)| {
                !(first_remove_index <= *i && *i < first_unremove_index)
            });

        self.sorted_newline_indices = kept_newlines
            .into_iter()
            .map(|(_, newline_index)| newline_index)
            .collect();

        for shifted_newline_index in first_remove_index..self.sorted_newline_indices.len() {
            self.sorted_newline_indices[shifted_newline_index] -= bytes_to_remove;
        }

        let mut removed_newlines = VecDeque::from(removed_newlines);
        let mut removed_bytes = Vec::<u8>::new();
        for i in 0..bytes_to_remove {
            let Some(removed_byte) = self.underlying_buf.pop_after_cursor() else {
                assert!(
                    removed_newlines.is_empty(),
                    "Reached the end of bytes pre-cursor but still had more newlines expected to be hit."
                );

                break;
            };

            removed_bytes.push(removed_byte);

            if let Some((_, next_newline_index)) = removed_newlines.front() {
                if i + cursor_byte_index == *next_newline_index {
                    assert_eq!(
                        char::try_from(removed_byte).expect("Failed to convert expected newline byte to char"),
                        '\n',
                        "Expected newline character at stored newline while removing. Found: non newline"
                    );
                    _ = removed_newlines.pop_front();
                }
            }
        }

        assert!(
            removed_newlines.is_empty(),
            "Expected all removed newlines to be found while removing bytes from buffer."
        );

        String::from_utf8(removed_bytes)
            .expect("Expected valid utf-8 string to be removed from buffer. Found: invalid string")
    }

    #[cfg(not(debug_assertions))]
    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        use std::mem;

        let cursor_byte_index = self.cursor_byte_index();

        let mut bytes_to_remove = 0;
        let mut chars_remaining = char_count;
        while chars_remaining > 0 {
            let byte = self.underlying_buf[cursor_byte_index + bytes_to_remove];
            match super::expected_byte_length_from_starting(byte) {
                Some(length) => {
                    bytes_to_remove += length as usize;
                    chars_remaining -= 1;
                }
                None => panic!("Found invalid utf8 encoded bytes while deleting at cursor"),
            }
        }

        let bytes_to_remove = bytes_to_remove;
        let first_remove_index = self
            .sorted_newline_indices
            .partition_point(|i| *i < cursor_byte_index);
        let first_unremove_index = self
            .sorted_newline_indices
            .partition_point(|i| *i < (cursor_byte_index + bytes_to_remove));

        let old_newlines = mem::take(&mut self.sorted_newline_indices);

        let kept_newlines: Vec<_> = old_newlines
            .into_iter()
            .enumerate()
            .filter_map(|(i, newline_index)| {
                if first_remove_index <= i && i < first_unremove_index {
                    None
                } else {
                    Some(newline_index)
                }
            })
            .collect();

        self.sorted_newline_indices = kept_newlines;

        for shifted_newline_index in first_remove_index..self.sorted_newline_indices.len() {
            self.sorted_newline_indices[shifted_newline_index] -= bytes_to_remove;
        }

        let mut removed_bytes = Vec::<u8>::new();
        for i in 0..bytes_to_remove {
            let Some(removed_byte) = self.underlying_buf.pop_after_cursor() else {
                break;
            };
            removed_bytes.push(removed_byte);
        }

        String::from_utf8(removed_bytes)
            .expect("Expected valid utf-8 string to be removed from buffer. Found: invalid string")
    }

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        Box::new(ByteCharIter::new(self.underlying_buf.iter()))
    }

    fn content_byte_length(&self) -> usize {
        self.underlying_buf.len()
    }

    fn content_line_count(&self) -> usize {
        self.sorted_newline_indices.len() + 1
    }

    fn content_copy(&self) -> String {
        let utf8_bytes: Vec<u8> = self.underlying_buf.iter().map(|c| *c).collect();

        String::from_utf8(utf8_bytes).expect("Found invalid utf8 encoding in GapBuffer")
    }

    fn content_copy_at_byte_index(
        &self,
        mut byte_index: usize,
        mut char_count: usize,
    ) -> Option<String> {
        let mut bytes_copy = vec![];

        while char_count > 0 {
            let start_byte = *self.underlying_buf.get(byte_index)?;
            let char_length = super::expected_byte_length_from_starting(start_byte)? as usize;

            bytes_copy.push(start_byte);
            for i in 0..(char_length - 1) {
                bytes_copy.push(*self.underlying_buf.get(byte_index + i + 1)?);
            }

            byte_index += char_length;
            char_count -= 1;
        }

        std::str::from_utf8(&bytes_copy).ok().map(|s| s.to_string())
    }

    fn content_copy_line(&self, line_index: usize) -> Option<String> {
        let mut start_index = line_index
            .checked_sub(1)
            .map(|newline_index| {
                self.sorted_newline_indices
                    .get(newline_index)
                    .map(|newline| newline + 1)
            })
            .flatten()?;
        let end_index = self
            .sorted_newline_indices
            .get(line_index)
            .map(|newline| newline + 1)?;

        let mut bytes = vec![];
        while start_index < end_index {
            let byte = self.underlying_buf[start_index];
            match super::expected_byte_length_from_starting(byte) {
                Some(length) => {
                    bytes.push(byte);
                    for i in 1..length {
                        bytes.push(self.underlying_buf[start_index + i as usize]);
                    }
                    start_index += length as usize;
                }
                None => panic!(
                    "Invalid utf8 encoding. Found non-start byte at expected character byte index"
                ),
            }
        }

        std::str::from_utf8(&bytes).map(|str| str.to_string()).ok()
    }

    fn set_cursor_byte_index(&mut self, index: usize, keep_col_index: bool) {
        self.underlying_buf.set_cursor(index);

        if !keep_col_index {
            match self.lookup_index_of_preceeding_newline(index) {
                Some(lookup_newline_index) => {
                    let preceeding_newline_index =
                        self.sorted_newline_indices[lookup_newline_index];
                    let char_count = self.char_count_in((preceeding_newline_index + 1)..index);
                    self.char_col_index = char_count as usize;
                    self.line_index = lookup_newline_index + 1;
                }
                None => {
                    self.char_col_index = index;
                    self.line_index = 0;
                }
            }
        }
    }

    fn set_cursor_line_index(&mut self, index: usize) {
        self.line_index = index;

        let mut byte_index = if index == 0 {
            0
        } else {
            let Some(new_line_index) = self.sorted_newline_indices.get(index - 1) else {
                return;
            };

            *new_line_index + 1
        };

        let mut line_char_count = 0;
        while let Some(byte) = self.underlying_buf.get(byte_index) {
            if line_char_count == self.char_col_index {
                break;
            }

            match super::expected_byte_length_from_starting(*byte) {
                Some(length) => {
                    if length == 1 && *byte == 0xA {
                        // Found newline. Out of characters in this line
                        break;
                    }

                    line_char_count += 1;
                    byte_index += length as usize;
                }
                None => panic!("Found invalid utf8 encoding while setting cursor_line_index"),
            }
        }

        self.underlying_buf.set_cursor(byte_index);
    }

    fn cursor_byte_index(&self) -> usize {
        self.underlying_buf.cursor_index()
    }

    fn cursor_line_index(&self) -> usize {
        self.line_index
    }

    fn line_index_for_byte_index(&self, byte_index: usize) -> usize {
        match self.sorted_newline_indices.binary_search(&byte_index) {
            Ok(on_newline_index) => on_newline_index,
            Err(insert_newline_index) => insert_newline_index,
        }
    }

    fn cursor_moved_by_char(&self, mut char_count: isize) -> usize {
        if char_count == 0 {
            0
        } else if char_count < 0 {
            let precursor_iter = self.underlying_buf.precursor_iter();
            let mut byte_count = 0;

            for precursor_byte in precursor_iter {
                if let Some(_) = super::expected_byte_length_from_starting(*precursor_byte) {
                    char_count += 1;
                }

                byte_count += 1;

                if char_count == 0 {
                    break;
                }
            }

            self.underlying_buf.cursor_index() - byte_count
        } else {
            let postcursor_iter = self.underlying_buf.postcursor_iter();
            let mut byte_count = 0;

            for postcursor_byte in postcursor_iter {
                if let Some(_) = super::expected_byte_length_from_starting(*postcursor_byte) {
                    char_count -= 1;
                }

                byte_count += 1;

                if char_count == 0 {
                    break;
                }
            }

            self.underlying_buf.cursor_index() + byte_count
        }
    }

    fn index_moved_by_char(&self, start_byte_index: usize, mut char_count: isize) -> usize {
        let mut result_byte_index = start_byte_index;

        // Need to move all the way to character start char
        char_count += char_count.signum();

        loop {
            let Some(byte) = self.underlying_buf.get(result_byte_index) else {
                let Some(new_result) = result_byte_index.checked_add_signed(char_count.signum())
                else {
                    return 0;
                };
                result_byte_index = new_result;
                char_count -= char_count.signum();

                if result_byte_index >= self.underlying_buf.len() {
                    return self.underlying_buf.len()
                } else {
                    continue;
                }
            };

            if let Some(_) = super::expected_byte_length_from_starting(*byte) {
                char_count -= char_count.signum();
            }

            if char_count == 0 {
                break;
            }

            let Some(new_result) = result_byte_index.checked_add_signed(char_count.signum()) else {
                return 0;
            };
            result_byte_index = new_result;

            if result_byte_index == self.underlying_buf.len() {
                return self.underlying_buf.len();
            }
        }

        result_byte_index
    }

    fn populate_from_read(&mut self, read: &mut dyn std::io::prelude::Read) -> std::io::Result<()> {
        let mut string = String::new();
        read.read_to_string(&mut string)?;

        self.populate_from_vec(string.as_bytes());

        Ok(())
    }

    fn flush_to_write(
        &mut self,
        write: &mut dyn crate::file_handle::FileWrite,
    ) -> std::io::Result<()> {
        let write_buffer: Vec<_> = self.underlying_buf.iter().map(|b| *b).collect();

        write.write_file(write_buffer.as_slice())
    }
}
