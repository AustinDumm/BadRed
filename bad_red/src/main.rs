use std::{
    io::{self, Stdout, Write},
    iter::Peekable,
};

use buffer::Update;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    queue, style, terminal::{self, *},
};
use editor_state::EditorState;
use mlua::Lua;

use crate::script_handler::BuiltIn;

mod buffer;
mod editor_state;
mod pane;
mod script_handler;


struct Terminal {
    stdout: Stdout,
}

impl Terminal {
    const TITLE: &'static str = "BadRed";

    fn new(stdout: Stdout) -> io::Result<Self> {
        let mut new = Self { stdout };
        if let Err(e) = new.setup_terminal() {
            let _ = new.cleanup_terminal();

            Err(e)
        } else {
            Ok(new)
        }
    }

    fn setup_terminal(&mut self) -> io::Result<()> {
        queue!(
            self.stdout,
            EnterAlternateScreen,
            SetTitle(Self::TITLE),
            cursor::MoveTo(0, 0)
        )?;
        self.stdout.flush()?;

        enable_raw_mode()
    }

    fn cleanup_terminal(&mut self) -> io::Result<()> {
        queue!(self.stdout, LeaveAlternateScreen)?;
        self.stdout.flush()?;

        disable_raw_mode()
    }

    fn render(&mut self, editor_state: &EditorState) -> io::Result<()> {
        let window_size = terminal::window_size()?;
        let editor_size = EditorSize {
            x_row: 0,
            y_col: 0,
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

            queue!(self.stdout, cursor::MoveTo(size.y_col, size.x_row))?;
            let mut buffer_row = 0;
            let mut buffer_col = 0;
            let mut cursor_position: Option<(u16, u16)> = None;
            let mut chars = buffer.content.chars().peekable();
            let mut char_count = 0;

            while let Some(char) = chars.peek() {
                if buffer_row == pane.top_line {
                    break;
                }

                if *char == '\n' || *char == '\r' {
                    let char = chars.next().unwrap();
                    if handle_newline(char, &mut char_count, &mut chars) {
                        buffer_row += 1;
                    }
                }
            }

            while let Some(char) = chars.next() {
                char_count += 1;
                let is_newline = handle_newline(char, &mut char_count, &mut chars);
                if is_newline {
                    queue!(self.stdout, style::Print(" "))?;
                    for _ in buffer_col..size.cols - 2 {
                        queue!(self.stdout, Clear(ClearType::UntilNewLine))?;
                    }
                    queue!(self.stdout, style::Print("\n\r"))?;
                    buffer_row += 1;
                    buffer_col = 0;
                } else {
                    queue!(self.stdout, style::Print(char))?;
                    buffer_col += 1;
                }

                if char_count == buffer.cursor_index {
                    cursor_position = Some((buffer_row, buffer_col));
                }
            }

            for _ in buffer_row..size.rows {
                for _ in buffer_col..size.cols {
                    queue!(self.stdout, style::Print(" "))?;
                }
                buffer_col = 0;
            }

            if let Some(cursor_position) = cursor_position {
                queue!(self.stdout, cursor::MoveTo(cursor_position.1, cursor_position.0))?;
            } else {
                panic!()
            }
        }

        self.stdout.flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.cleanup_terminal();
    }
}

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut terminal = Terminal::new(stdout)?;

    let mut editor_state = EditorState::new();
    let lua = Lua::new();
    loop {
        let update = match read()? {
            Event::Key(event) if event.code == KeyCode::Esc => break,
            event => {
                editor_state.buffers[editor_state.active_buffer].handle_event(event);
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

        terminal.render(&editor_state)?;
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


fn handle_newline<I>(char: char, char_count: &mut usize, chars: &mut Peekable<I>) -> bool
where
    I: Iterator<Item = char>,
{
    if char == '\r' {
        if chars.peek() == Some(&'\n') {
            *char_count += 1;
            _ = chars.next();
        }
        true
    } else if char == '\n' {
        true
    } else {
        false
    }
}

