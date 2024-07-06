// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

//! A gap buffer implementation intended for use in the BadRed text editor.
//!
//! [GapBuffer] is built for efficient insertion of elements within a block of contiguous memory
//! through the use of a cursor. A gap buffer is a good solution for when elements need to be
//! inserted in the middle of the buffer more often than the cursor insertion point needs to be
//! moved, such as in a text editor.

#![warn(missing_docs)]

pub use gap_buffer::*;

mod gap_buffer;

