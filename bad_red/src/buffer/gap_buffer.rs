// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

#![allow(unused_variables)]

use bad_gap::GapBuffer as UnderlyingBuf;

use super::ContentBuffer;

struct GapBuffer {
    underlying_buf: UnderlyingBuf<u8>,
}

impl ContentBuffer for GapBuffer {
    fn insert_at_cursor(&mut self, content: &str) {
        todo!()
    }

    fn delete_at_cursor(&mut self, char_count: usize) -> String {
        todo!()
    }

    fn chars(&self) -> Box<dyn Iterator<Item = char> + '_> {
        todo!()
    }

    fn content_byte_length(&self) -> usize {
        todo!()
    }

    fn content_copy(&self) -> String {
        todo!()
    }

    fn set_cursor_byte_index(&mut self, index: usize) {
        todo!()
    }

    fn cursor_byte_index(&self) -> usize {
        todo!()
    }

    fn cursor_moved_by_char(&mut self, char_count: isize) -> usize {
        todo!()
    }

    fn cursor_moved_by_line(&mut self, line_count: usize, move_up: bool) -> usize {
        todo!()
    }

    fn populate_from_read(&mut self, read: &mut dyn std::io::prelude::Read) -> std::io::Result<()> {
        todo!()
    }

    fn flush_to_write(&mut self, write: &mut dyn crate::file_handle::FileWrite) -> std::io::Result<()> {
        todo!()
    }
}

