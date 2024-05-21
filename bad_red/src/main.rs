use std::io;

use crossterm::event::{read, Event, KeyCode};
use display::Display;
use editor_state::EditorState;
use mlua::Lua;

mod buffer;
mod display;
mod editor_state;
mod pane;
mod script_handler;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut display = Display::new(stdout)?;

    let mut editor_state = EditorState::new();
    let lua = Lua::new();
    loop {
        let update = match read()? {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            event => {
                editor_state
                    .dispatch_input(event)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        };

        display.render(&editor_state)?;
    }

    Ok(())
}

fn evaluate_command(
    command_text: &str,
    lua: &Lua,
    editor_state: &mut EditorState,
) -> mlua::Result<()> {
    Ok(())
}

#[derive(Clone)]
pub struct EditorFrame {
    pub x_col: u16,
    pub y_row: u16,
    pub rows: u16,
    pub cols: u16,
}

impl EditorFrame {
    pub fn with_x_col(&self, x_col: u16) -> Self {
        let mut new = self.clone();
        new.x_col = x_col;
        new
    }

    pub fn with_y_row(&self, y_row: u16) -> Self {
        let mut new = self.clone();
        new.y_row = y_row;
        new
    }

    pub fn with_rows(&self, rows: u16) -> Self {
        let mut new = self.clone();
        new.rows = rows;
        new
    }

    pub fn with_cols(&self, cols: u16) -> Self {
        let mut new = self.clone();
        new.cols = cols;
        new
    }

    pub fn less_rows(&self, rows: u16) -> Self {
        let mut new = self.clone();
        new.rows -= rows;
        new
    }

    pub fn less_cols(&self, cols: u16) -> Self {
        let mut new = self.clone();
        new.cols -= cols;
        new
    }
}
