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
mod editor_frame;

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut display = Display::new(stdout)?;

    let mut editor_state = EditorState::new();
    let lua = Lua::new();
    loop {
        match read()? {
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

