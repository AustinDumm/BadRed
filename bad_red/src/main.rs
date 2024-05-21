use std::io;

use buffer::Update;
use crossterm::event::{read, Event, KeyCode};
use display::Display;
use editor_state::EditorState;
use mlua::Lua;

use crate::script_handler::BuiltIn;

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
                //editor_state.buffers[editor_state.active_buffer].handle_event(event);
                Update::All
            }
        };

        match update {
            Update::None => continue,
            Update::All => (),
            Update::Command(command_text) => {
                evaluate_command(command_text, &lua, &mut editor_state).unwrap()
            }
        }

        display.render(&editor_state)?;
    }

    Ok(())
}

fn evaluate_command(
    command_text: &str,
    lua: &Lua,
    editor_state: &mut EditorState,
) -> mlua::Result<()> {
    let chunk = lua.load(command_text);
    let commands: Vec<BuiltIn> = chunk.call(())?;
    editor_state.execute_commands(commands);

    Ok(())
}

#[derive(Clone)]
pub struct EditorSize {
    pub x_row: u16,
    pub y_col: u16,
    pub rows: u16,
    pub cols: u16,
}

impl EditorSize {
    pub fn with_x_row(&self, x_row: u16) -> Self {
        let mut new = self.clone();
        new.x_row = x_row;
        new
    }

    pub fn with_y_col(&self, y_col: u16) -> Self {
        let mut new = self.clone();
        new.y_col = y_col;
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
