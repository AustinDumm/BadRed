use std::{
    io::{self, Write},
    panic,
};

use bad_red_lib::{display::Display, editor_state::{self, Editor}};
use crossterm::event::{read, Event, KeyCode};

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

    let mut editor = Editor::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    loop {
        match read()? {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            event => {
                match editor
                    .handle_event(event)
                {
                    Ok(_) => Ok(()),
                    Err(e) => match e {
                        editor_state::Error::Unrecoverable(e) =>
                            Err(io::Error::new(io::ErrorKind::Other, format!("Internal unrecoverable error: {}", e))),
                        editor_state::Error::Recoverable(_) => Ok(()),
                        editor_state::Error::Script(_) => Ok(()),
                    }
                }?
            }
        };

        display.render(&editor)?;
    }

    drop(display);
    Ok(())
}
