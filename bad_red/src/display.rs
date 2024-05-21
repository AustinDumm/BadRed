use crossterm::{
    cursor,
    queue, style,
    terminal::{self, *},
};
use std::{
    io::{self, Stdout, Write},
    iter::Peekable,
};

use crate::{editor_state::EditorState, EditorSize};

pub struct Display {
    stdout: Stdout,
}

impl Display {
    const TITLE: &'static str = "BadRed";

    pub fn new(stdout: Stdout) -> io::Result<Self> {
        let mut new = Self { stdout };
        if let Err(e) = new.setup_display() {
            let _ = new.cleanup_display();

            Err(e)
        } else {
            Ok(new)
        }
    }

    fn setup_display(&mut self) -> io::Result<()> {
        queue!(
            self.stdout,
            EnterAlternateScreen,
            SetTitle(Self::TITLE),
            cursor::MoveTo(0, 0)
        )?;
        self.stdout.flush()?;

        enable_raw_mode()
    }

    fn cleanup_display(&mut self) -> io::Result<()> {
        queue!(self.stdout, LeaveAlternateScreen)?;
        self.stdout.flush()?;

        disable_raw_mode()
    }

    pub fn render(&mut self, editor_state: &EditorState) -> io::Result<()> {
        let window_size = terminal::window_size()?;
        let editor_size = EditorSize {
            x_row: 0,
            y_col: 0,
            rows: window_size.rows,
            cols: window_size.columns,
        };

       // for (buffer_index, buffer_pane_path) in editor_state.buffer_panes.iter().enumerate() {
       //     let Some(buffer_pane_path) = buffer_pane_path else {
       //         continue;
       //     };

       //     let buffer = &editor_state.buffers[buffer_index];
       //     let Some((pane, size)) = editor_state
       //         .root_pane
       //         .leaf_and_size_at_path(&editor_size, *buffer_pane_path)
       //     else {
       //         writeln!(io::stderr(), "Failed attempt to find pane by path")?;
       //         continue;
       //     };

       //     queue!(self.stdout, cursor::MoveTo(size.y_col, size.x_row))?;
       //     let mut buffer_row = 0;
       //     let mut buffer_col = 0;
       //     let mut cursor_position: Option<(u16, u16)> = None;
       //     let mut chars = buffer.content.chars().peekable();
       //     let mut char_count = 0;

       //     while let Some(char) = chars.peek() {
       //         if buffer_row == pane.top_line {
       //             break;
       //         }

       //         if *char == '\n' || *char == '\r' {
       //             let char = chars.next().unwrap();
       //             if handle_newline(char, &mut char_count, &mut chars) {
       //                 buffer_row += 1;
       //             }
       //         }
       //     }

       //     while let Some(char) = chars.next() {
       //         char_count += 1;
       //         let is_newline = handle_newline(char, &mut char_count, &mut chars);
       //         if is_newline {
       //             queue!(self.stdout, style::Print(" "))?;
       //             for _ in buffer_col..size.cols - 2 {
       //                 queue!(self.stdout, Clear(ClearType::UntilNewLine))?;
       //             }
       //             queue!(self.stdout, style::Print("\n\r"))?;
       //             buffer_row += 1;
       //             buffer_col = 0;
       //         } else {
       //             queue!(self.stdout, style::Print(char))?;
       //             buffer_col += 1;
       //         }

       //         if char_count == buffer.cursor_index {
       //             cursor_position = Some((buffer_row, buffer_col));
       //         }
       //     }

       //     for _ in buffer_row..size.rows {
       //         for _ in buffer_col..size.cols {
       //             queue!(self.stdout, style::Print(" "))?;
       //         }
       //         buffer_col = 0;
       //     }

       //     if let Some(cursor_position) = cursor_position {
       //         queue!(
       //             self.stdout,
       //             cursor::MoveTo(cursor_position.1, cursor_position.0)
       //         )?;
       //     } else {
       //         panic!()
       //     }
       // }

        self.stdout.flush()
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

impl Drop for Display {
    fn drop(&mut self) {
        let _ = self.cleanup_display();
    }
}
