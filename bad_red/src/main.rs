
use std::{io::{self, Write}, panic};

use crossterm::event::{read, Event, KeyCode};
use display::Display;
use editor_state::Editor;

mod buffer;
mod display;
mod editor_state;
mod pane;
mod script_handler;
mod editor_frame;

fn main() -> io::Result<()> {
    let result = panic::catch_unwind(|| {
        let result = run();
        if let Err(ref error) = result {
            write!(io::stderr(), "{:#?}", error)?;
        }
        result
    });

    if let Err(panic_err) = result {
        write!(io::stderr(), "Panic: {:#?}", panic_err)?;
    }

    Ok(())
}

fn run() -> io::Result<()> {
    let stdout = io::stdout();
    let mut display = Display::new(stdout)?;

    let mut editor = Editor::new()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    loop {
        match read()? {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            event => {
                editor
                    .handle_event(event)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        };

        display.render(&editor)?;
    }

    drop(display);
    Ok(())
}

