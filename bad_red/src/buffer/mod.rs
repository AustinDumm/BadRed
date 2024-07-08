// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

pub use content_buffer::*;
pub use editor_buffer::*;
pub use byte_char_iter::expected_byte_length_from_starting;

mod content_buffer;
mod editor_buffer;

mod naive_buffer;
mod gap_buffer;

mod byte_char_iter;

pub const ZERO_WIDTH_JOINER: char = '\u{200D}';

