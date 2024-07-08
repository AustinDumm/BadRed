// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

#![allow(unused_variables)]

use bad_gap::GapBuffer as UnderlyingBuf;

use super::{byte_char_iter::ByteCharIter, ContentBuffer};

struct GapBuffer {
    underlying_buf: UnderlyingBuf<u8>,
    sorted_newline_indices: Vec<usize>,
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

        for content_index in content
            .char_indices()
            .filter(|(_, c)| *c == '\n')
            .map(|(i, _)| i)
        {
            let insert_index = self
                .sorted_newline_indices
                .partition_point(|i| *i < content_index);
            self.sorted_newline_indices
                .insert(insert_index, content_index);
        }
    }

    #[cfg(debug_assertions)]
    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        use std::{collections::VecDeque, mem};

        let cursor_byte_index = self.cursor_byte_index();

        let first_remove_index = self
            .sorted_newline_indices
            .partition_point(|i| *i < cursor_byte_index);
        let first_unremove_index = self
            .sorted_newline_indices
            .partition_point(|i| (cursor_byte_index + char_count) <= *i);

        let old_newlines = mem::take(&mut self.sorted_newline_indices);
        let (removed_newlines, kept_newlines): (Vec<_>, Vec<_>) = old_newlines
            .into_iter()
            .enumerate()
            .partition(|(i, newline_index)| {
                !(first_remove_index <= *i && *i < first_unremove_index)
            });
        self.sorted_newline_indices = kept_newlines
            .into_iter()
            .map(|(_, newline_index)| newline_index)
            .collect();

        let removed_newlines = VecDeque::from(removed_newlines);
        let mut removed_bytes = Vec::<u8>::new();
        for i in 0..char_count {
            let Some(removed_byte) = self.underlying_buf.pop_before_cursor() else {
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
                }
            }
        }

        String::from_utf8(removed_bytes)
            .expect("Expected valid utf-8 string to be removed from buffer. Found: invalid string")
    }

    #[cfg(not(debug_assertions))]
    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        use std::mem;

        let cursor_byte_index = self.cursor_byte_index();

        let first_remove_index = self
            .sorted_newline_indices
            .partition_point(|i| *i < cursor_byte_index);
        let first_unremove_index = self
            .sorted_newline_indices
            .partition_point(|i| (cursor_byte_index + char_count) <= *i);

        let old_newlines = mem::take(&mut self.sorted_newline_indices);
        self.sorted_newline_indices = old_newlines
            .into_iter()
            .enumerate()
            .filter_map(|(i, newline_index)| {
                if !(first_remove_index <= i && i < first_unremove_index) {
                    None
                } else {
                    Some(newline_index)
                }
            })
            .collect();

        let mut removed_bytes = Vec::<u8>::new();
        for i in 0..char_count {
            let Some(removed_byte) = self.underlying_buf.pop_before_cursor() else {
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

    fn content_copy(&self) -> String {
        let utf8_bytes: Vec<u8> = self.underlying_buf.iter().map(|c| *c).collect();

        String::from_utf8(utf8_bytes).expect("Found invalid utf8 encoding in GapBuffer")
    }

    fn set_cursor_byte_index(&mut self, index: usize) {
        self.underlying_buf.set_cursor(index);
    }

    fn cursor_byte_index(&self) -> usize {
        self.underlying_buf.cursor_index()
    }

    fn cursor_moved_by_char(&mut self, mut char_count: isize) -> usize {
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

            byte_count

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

            byte_count
        }
    }

    fn cursor_moved_by_line(&mut self, line_count: usize, move_up: bool) -> usize {
        todo!()
    }

    fn populate_from_read(&mut self, read: &mut dyn std::io::prelude::Read) -> std::io::Result<()> {
        let mut read_vec = Vec::new();
        read.read_to_end(&mut read_vec)?;

        self.underlying_buf = UnderlyingBuf::from(read_vec);

        Ok(())
    }

    fn flush_to_write(
        &mut self,
        write: &mut dyn crate::file_handle::FileWrite,
    ) -> std::io::Result<()> {
        let write_buffer: Vec<_> = self.underlying_buf
            .iter()
            .map(|b| *b)
            .collect();

        write.write_file(write_buffer.as_slice())
    }
}
