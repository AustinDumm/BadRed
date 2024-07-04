
pub use content_buffer::*;
pub use editor_buffer::*;

mod content_buffer;
mod editor_buffer;

mod naive_buffer;

pub const ZERO_WIDTH_JOINER: char = '\u{200D}';

