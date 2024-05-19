
pub enum Update {
    All
}

pub struct Buffer {
    pub title: String,
    pub cursor_index: usize,
    pub content: String,
}

impl Buffer {
    pub fn new(title: String) -> Self {
        Self {
            title,
            cursor_index: 0,
            content: String::new(),
        }
    }
}

