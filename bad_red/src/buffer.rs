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

    pub fn delete_at_cursor(&mut self, char_count: usize) -> String {
        let first_non_delete = (self.cursor_index + char_count).min(self.content.len());
        let string_to_delete = self.content[self.cursor_index..first_non_delete].to_string();
        let new_content = format!(
            "{}{}",
            &self.content[..self.cursor_index],
            &self.content[first_non_delete..]
        );
        self.content = new_content;
        self.is_dirty = true;

        string_to_delete
    }

    pub fn move_cursor(&mut self, char_count: usize, move_left: bool) {
        self.cursor_index = if move_left {
            self.cursor_index
                .saturating_sub(char_count)
        } else {
            self.cursor_index
                .saturating_add(char_count)
                .min(self.content.len())
        };
        self.is_dirty = true;
    }

    pub fn content_length(&self) -> usize {
        self.content.len()
    }

    pub fn cursor_content_index(&self) -> usize {
        self.cursor_index
    }

    pub fn set_cursor_content_index(&mut self, index: usize) {
        self.cursor_index = index.min(self.content_length());
    }
}
