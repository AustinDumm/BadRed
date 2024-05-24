use crossterm::{
    cursor, queue, style,
    terminal::{self, *},
};
use std::{
    io::{self, ErrorKind, Stdout, Write},
    iter::Peekable,
};

use crate::{
    editor_frame::EditorFrame,
    editor_state::EditorState,
    pane::{Pane, PaneNode, PaneNodeType, PaneTree},
};

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
        let editor_frame = EditorFrame {
            x_col: 0,
            y_row: 0,
            rows: window_size.rows,
            cols: window_size.columns,
        };

        self.render_to_pane(editor_state, editor_frame, &editor_state.pane_tree, 0)
    }

    fn render_to_pane(
        &mut self,
        editor_state: &EditorState,
        editor_frame: EditorFrame,
        pane_tree: &PaneTree,
        node_index: usize,
    ) -> io::Result<()> {
        let node = pane_tree.tree.get(node_index).ok_or(io::Error::new(
            ErrorKind::Other,
            format!("Failed to find pane at index: {}", node_index),
        ))?;

        match &node.node_type {
            PaneNodeType::Leaf(ref pane) => self.render_leaf_pane(pane, editor_state, editor_frame),
            PaneNodeType::VSplit(split) => {
                self.render_to_pane(
                    editor_state,
                    editor_frame.less_cols(split.first_char_size - 1),
                    pane_tree,
                    split.first,
                )?;
                self.render_to_pane(
                    editor_state,
                    editor_frame
                        .with_x_col(split.first_char_size + 1)
                        .less_cols(split.first_char_size),
                    pane_tree,
                    split.second,
                )
            }
            PaneNodeType::HSplit(split) => {
                self.render_to_pane(
                    editor_state,
                    editor_frame.less_rows(split.first_char_size - 1),
                    pane_tree,
                    split.first,
                )?;
                self.render_to_pane(
                    editor_state,
                    editor_frame
                        .with_y_row(split.first_char_size + 1)
                        .less_rows(split.first_char_size),
                    pane_tree,
                    split.second,
                )
            }
        }
    }

    fn render_leaf_pane(
        &mut self,
        pane: &Pane,
        editor_state: &EditorState,
        editor_frame: EditorFrame,
    ) -> io::Result<()> {
        let Some(buffer) = &editor_state.buffers.get(pane.buffer_id) else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to buffer id {} associated with pane",
                    pane.buffer_id
                ),
            ));
        };

        queue!(
            self.stdout,
            cursor::MoveTo(editor_frame.y_row, editor_frame.x_col)
        )?;

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
                char_count += 1;
                if handle_newline(char, &mut char_count, &mut chars) {
                    buffer_row += 1;
                }
            }
        }
        buffer_row = 0;

        while let Some(char) = chars.next() {
            char_count += 1;
            let is_newline = handle_newline(char, &mut char_count, &mut chars);
            if is_newline {
                queue!(
                    self.stdout,
                    style::Print(" "),
                    Clear(ClearType::UntilNewLine),
                    style::Print("\n\r"),
                )?;
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

        queue!(self.stdout, Clear(ClearType::FromCursorDown))?;

        if let Some(cursor_position) = cursor_position {
            queue!(
                self.stdout,
                cursor::MoveTo(cursor_position.1, cursor_position.0)
            )?;
        }

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
