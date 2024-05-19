use std::io::{self, Stdout, Write};

use crossterm::{
    event::{read, Event, KeyCode},
    queue,
    terminal::{self, *},
};
use editor_state::EditorState;

mod buffer;
mod editor_state;
mod pane;

const TITLE: &str = "BadRed";

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    if let Err(err) = setup_terminal(&mut stdout) {
        let _ = cleanup_terminal(&mut stdout);
        return Err(err);
    }

    let mut editor_state = EditorState::new();
    loop {
        let update = match read()? {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            event => println!("EVENT: {:?}", event),
        };
    }

    cleanup_terminal(&mut stdout)
}

#[derive(Clone)]
pub struct EditorSize {
    pub x_row: u16,
    pub y_row: u16,
    pub rows: u16,
    pub cols: u16,
}

impl EditorSize {
    pub fn with_x_row(&self, x_row: u16) -> Self {
        let mut new = self.clone();
        new.x_row = x_row;
        new
    }

    pub fn with_y_col(&self, y_row: u16) -> Self {
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

fn render(editor_state: &EditorState) -> io::Result<()> {
    let window_size = terminal::window_size()?;
    let editor_size = EditorSize {
        x_row: 0,
        y_row: 0,
        rows: window_size.rows,
        cols: window_size.columns,
    };

    for (buffer_index, buffer_pane_path) in editor_state.buffer_panes.iter().enumerate() {
        let Some(buffer_pane_path) = buffer_pane_path else {
            continue;
        };

        let buffer = &editor_state.buffers[buffer_index];
        let Some((pane, size)) = editor_state
            .root_pane
            .leaf_and_size_at_path(&editor_size, *buffer_pane_path)
        else {
            writeln!(io::stderr(), "Failed attempt to find pane by path")?;
            continue;
        };
    }
    todo!()
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
