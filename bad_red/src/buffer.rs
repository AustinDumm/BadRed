use std::mem;

use crossterm::event::{self, Event, KeyEvent, KeyModifiers};

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

    pub fn handle_event(&mut self, event: Event) -> BufferUpdate {
        match event {
            Event::FocusGained | Event::FocusLost | Event::Mouse(_) | Event::Resize(_, _) => BufferUpdate::None,
            Event::Paste(_) => BufferUpdate::None,
            Event::Key(key) => self.handle_key_event(key),
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> BufferUpdate {
        match event.code {
            event::KeyCode::Backspace => {
                if self.cursor_index > 0 {
                    self.content.remove(self.cursor_index - 1);
                    self.cursor_index -= 1;
                }
                self.is_dirty = true;
            }
            event::KeyCode::Enter => {
                self.content.insert(self.cursor_index, '\n');
                self.cursor_index += 1;
                self.is_dirty = true;
            }
            event::KeyCode::Left => {
                self.cursor_index = self.cursor_index.saturating_sub(1);
                self.is_dirty = true;
            }
            event::KeyCode::Right => {
                self.cursor_index = self.cursor_index.saturating_add(1);
                let char_count = self.content.chars().count();
                if self.cursor_index > char_count {
                    self.cursor_index = char_count;
                }
                self.is_dirty = true;
            }
            event::KeyCode::Up => (),
            event::KeyCode::Down => (),
            event::KeyCode::Home => (),
            event::KeyCode::End => (),
            event::KeyCode::PageUp => (),
            event::KeyCode::PageDown => (),
            event::KeyCode::Tab => {
                self.content.insert(self.cursor_index, '\t');
                self.is_dirty = true;
            },
            event::KeyCode::BackTab => (),
            event::KeyCode::Delete => {
                if self.cursor_index < self.content.chars().count() {
                    self.content.remove(self.cursor_index);
                }
                self.is_dirty = true;
            },
            event::KeyCode::Insert => (),
            event::KeyCode::F(_) => (),
            event::KeyCode::Char(char) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) && char == 'e' {
                    let command = mem::replace(&mut self.content, String::new());
                    self.cursor_index = 0;
                    self.is_dirty = true;
                    return BufferUpdate::Command(command);
                }

                let char = if event.modifiers.contains(KeyModifiers::SHIFT) {
                    char.to_ascii_uppercase()
                } else {
                    char
                };

                if self.cursor_index == self.content.chars().count() {
                    self.content.push(char);
                } else {
                    self.content.insert(self.cursor_index, char);
                }
                self.cursor_index += 1;
                self.is_dirty = true;
            },
            event::KeyCode::Null => (),
            event::KeyCode::Esc => (),
            event::KeyCode::CapsLock => (),
            event::KeyCode::ScrollLock => (),
            event::KeyCode::NumLock => (),
            event::KeyCode::PrintScreen => (),
            event::KeyCode::Pause => (),
            event::KeyCode::Menu => (),
            event::KeyCode::KeypadBegin => (),
            event::KeyCode::Media(_) => (),
            event::KeyCode::Modifier(_) => (),
        }

        BufferUpdate::Raw
    }
}
