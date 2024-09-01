// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{fs, io, panic, path::PathBuf, time::Duration};

use bad_red_lib::{
    display::Display,
    editor_state::{self, Editor},
    script_handler::ScriptHandler,
    script_runtime::SchedulerYield,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use clap::Parser;
use etcetera::{AppStrategy, AppStrategyArgs};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'd', long)]
    default_init: bool,
    #[arg(short, long)]
    init_path: Option<String>,
    #[arg(long)]
    init_name: Option<String>,
}

const DEFAULT_INIT_SCRIPT: &'static str = "init.lua";

fn main() -> io::Result<()> {
    let args = Args::parse();
    let stdout = io::stdout();
    let mut display = Display::new(stdout)?;

    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        Display::new(io::stdout())
            .unwrap()
            .cleanup_display()
            .unwrap();
        default_hook(panic_info);
    }));

    let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: "me".to_string(),
        author: "adumm".to_string(),
        app_name: "BadRed".to_string(),
    })
    .unwrap();
    let config_dir = strategy.config_dir();

    run(
        args.default_init,
        config_dir,
        args.init_name.unwrap_or(DEFAULT_INIT_SCRIPT.to_string()),
        &mut display,
    )
}

include!(concat!(env!("OUT_DIR"), "/consts_defs.rs"));

fn run(
    default_init: bool,
    init_path: PathBuf,
    init_file: String,
    display: &mut Display,
) -> io::Result<()> {
    let init_script = load_init_script(&init_path, &init_file, generated::INIT, default_init)?;

    let script_handler = ScriptHandler::new(init_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to init Lua: {}", e)))?;
    let mut editor = Editor::new(
        &script_handler.lua,
        generated::PRELOAD.to_string(),
        init_script,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    'editor_loop: loop {
        let var_name = false;
        let mut did_input = var_name;
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
            Ok(SchedulerYield::Run) => true,
            Ok(SchedulerYield::Skip) => false,
            Ok(SchedulerYield::Quit) => return Ok(()),
            Err(editor_state::Error::Unrecoverable(message))
            | Err(editor_state::Error::Recoverable(message))
            | Err(editor_state::Error::Script(message)) => {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Error found without hook capture: {:#?}", message),
                ))?;
                false
            }
        };

        if did_input || did_run_script {
            display.render(&editor)?;
        }

        editor.state.clear_dirty();
    }

    Ok(())
}

fn load_init_script(
    init_path: &PathBuf,
    init_file: &str,
    default_content: &str,
    always_default: bool,
) -> io::Result<String> {
    let file_path = init_path.join(init_file);
    if always_default {
        Ok(default_content.to_string())
    } else if file_path.exists() {
        fs::read_to_string(file_path)
    } else {
        fs::create_dir_all(init_path)?;
        fs::write(file_path, default_content)?;
        Ok(default_content.to_string())
    }
}
