use std::{
    io::{self, Write},
    panic,
};

use bad_red_lib::{
    display::Display,
    editor_state::{self, Editor},
    script_handler::ScriptHandler,
};
use crossterm::event::{self, read, Event, KeyCode};
use mlua::Lua;

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

    let script_handler = ScriptHandler::new()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to init Lua: {}", e)))?;
    let mut editor =
        Editor::new(&script_handler.lua).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    loop {
        if event::poll(editor.state.input_poll_rate)? {
            match event::read()? {
                Event::Key(event) if event.code == KeyCode::Esc => break,
                event => {
                    match editor.handle_event(event) {
                        Ok(_) => Ok(()),
                        Err(e) => match e {
                            editor_state::Error::Unrecoverable(e) => Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("Internal unrecoverable error: {}", e),
                            )),
                            editor_state::Error::Recoverable(_) => Ok(()),
                            editor_state::Error::Script(_) => Ok(()),
                        },
                    }?;
                }
            };
        }

        let script_result = editor.run_scripts();
        match script_result {
            Ok(_) => (),
            Err(editor_state::Error::Unrecoverable(message)) => 
                Err(io::Error::new(io::ErrorKind::Other, format!("{:#?}", message)))?,
            Err(_) => (),
        }

        display.render(&editor)?;

        editor.state.clear_dirty();
    }

    drop(display);
    Ok(())
}
