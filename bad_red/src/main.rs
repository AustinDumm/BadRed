// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{
    io::{self, Write},
    panic,
    time::Duration,
};

use bad_red_lib::{
    display::Display,
    editor_state::{self, Editor},
    script_handler::ScriptHandler,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    init_path: Option<String>,
    #[arg(long)]
    init_name: Option<String>,
}

const DEFAULT_INIT_PATH: &'static str = "../bad_red_lib";
const DEFAULT_INIT_SCRIPT: &'static str = "init.lua";

fn main() -> io::Result<()> {
    let args = Args::parse();

    let result = panic::catch_unwind(|| {
        let result = run(
            args.init_path.unwrap_or(DEFAULT_INIT_PATH.to_string()),
            args.init_name.unwrap_or(DEFAULT_INIT_SCRIPT.to_string()),
        );
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

fn run(init_path: String, init_file: String) -> io::Result<()> {
    let stdout = io::stdout();
    let init_script = load_init_script(format!("{}/{}", init_path, init_file))?;
    let mut display = Display::new(stdout)?;

    let script_handler = ScriptHandler::new(init_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to init Lua: {}", e)))?;
    let mut editor = Editor::new(&script_handler.lua, init_script)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    'editor_loop: loop {
        let mut did_input = false;
        if event::poll(editor.state.input_poll_rate)? {
            did_input = true;
            for _ in 0..10 {
                match event::read()? {
                    Event::Key(event)
                        if event.code == KeyCode::Delete
                            && event.modifiers.contains(KeyModifiers::CONTROL)
                            && event.modifiers.contains(KeyModifiers::ALT) =>
                    {
                        break 'editor_loop
                    }
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
                true
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

fn load_init_script(init_path: String) -> io::Result<String> {
    std::fs::read_to_string(init_path)
}
