
pub struct Buffer {
    pub title: String,
    pub cursor_index: usize,
    pub content: String,
    pub is_dirty: bool,
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
            content: String::new(),
            is_dirty: false,
        }
    }

    pub fn insert_at_cursor(&mut self, content: &str) {
        if self.cursor_index == self.content.chars().count() {
            self.content.push_str(content);
        } else {
            self.content.insert_str(self.cursor_index, content);
        }
        self.cursor_index += content.len();
        self.is_dirty = true;
    }

    pub fn backspace_at_cursor(&mut self) {
        if self.cursor_index > 0 {
            self.content.remove(self.cursor_index - 1);
            self.cursor_index -= 1;
        }
        self.is_dirty = true;
    }

    pub fn delete_at_cursor(&mut self) {
        if self.cursor_index < self.content.chars().count() {
            self.content.remove(self.cursor_index);
        }
        self.is_dirty = true;
    }
}
