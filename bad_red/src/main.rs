use std::{
    io::{self, Write},
    panic,
    time::Duration,
};

use bad_red_lib::{
    display::Display,
    editor_state::{self, Editor},
    keymap::KeyMap,
    script_handler::ScriptHandler,
};
use crossterm::event::{self, read, Event, KeyCode};
use mlua::Lua;

fn main() -> io::Result<()> {
    let result = panic::catch_unwind(|| {
        let result = run();
        if let Err(ref error) = result {
            write!(io::stderr(), "{:?}", error)?;
        }
        result
    });

    if let Err(panic_err) = result {
        write!(io::stderr(), "Panic: {:?}", panic_err)?;
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
    'editor_loop: loop {
        let mut did_input = false;
        if event::poll(editor.state.input_poll_rate)? {
            did_input = true;
            for _ in 0..10 {
                match event::read()? {
                    Event::Key(event) if event.code == KeyCode::Esc => break 'editor_loop,
                    Event::Key(event) => {
                        match editor.handle_key_event(event) {
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
                    _ => (),
                };

                if !event::poll(Duration::from_secs(0))? {
                    break;
                }
            }
        }

        let script_result = editor.run_scripts();
        let did_run_script = match script_result {
            Ok(did_run) => did_run,
            Err(editor_state::Error::Unrecoverable(message)) => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("{:#?}", message),
                ))?;
                false
            }
            Err(e) => {
                editor.state.push_to_buffer(format!("{}", e), 0);
                false
            }
        };

        if did_input || did_run_script {
            display.render(&editor)?;
        }

        editor.state.clear_dirty();
    }

    drop(display);
    Ok(())
}
