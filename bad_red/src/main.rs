use std::io::{self, Stdout, Write};

use crossterm::{event::{read, Event, KeyCode}, queue, terminal::*};

const TITLE: &str = "BadREd";

fn main() -> io::Result<()> {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    if let Err(err) = setup_terminal(&mut stdout) {
        let _ = cleanup_terminal(&mut stdout);
        return Err(err)
    }

    loop {
        match read()? {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            event => println!("EVENT: {:?}", event),
        }
    }

    cleanup_terminal(&mut stdout)
}

fn setup_terminal(stdout: &mut Stdout) -> io::Result<()> {
    queue!(stdout, EnterAlternateScreen, SetTitle(TITLE))?;
    stdout.flush()?;

    enable_raw_mode()
}

fn cleanup_terminal(stdout: &mut Stdout) -> io::Result<()> {
    queue!(stdout, LeaveAlternateScreen)?;
    stdout.flush()?;

    disable_raw_mode()
}
