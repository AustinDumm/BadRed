use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal::{self, *},
};
use std::{
    io::{self, ErrorKind, Stdout, Write},
    iter::Peekable,
    thread,
    time::Duration,
};

use crate::{
    editor_frame::EditorFrame,
    editor_state::{Editor, EditorState},
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

        disable_raw_mode()?;

        self.stdout.flush()
    }

    pub fn render(&mut self, editor: &Editor) -> io::Result<()> {
        let editor_state = &editor.state.borrow();
        let window_size = terminal::window_size()?;
        let editor_frame = EditorFrame {
            x_col: 0,
            y_row: 0,
            rows: window_size.rows,
            cols: window_size.columns,
        };

        self.render_to_pane(editor_state, &editor_frame, &editor_state.pane_tree, 0)
    }

    fn render_to_pane(
        &mut self,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
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
                let left_frame = editor_frame.percent_cols(split.first_percent, -1);
                self.render_to_pane(editor_state, &left_frame, pane_tree, split.first)?;
                queue!(
                    self.stdout,
                    style::Print(format!("{:?} -> {:?}", editor_frame, left_frame)),
                    style::SetBackgroundColor(Color::Green),
                )?;
                self.render_to_pane(
                    editor_state,
                    &editor_frame.percent_cols_shift(split.first_percent, -1),
                    pane_tree,
                    split.second,
                )?;
                self.render_frame_v_gap(left_frame)
            }
            PaneNodeType::HSplit(split) => {
                let top_frame = editor_frame.percent_rows(split.first_percent, -1);
                self.render_to_pane(editor_state, &top_frame, pane_tree, split.first)?;
                self.render_to_pane(
                    editor_state,
                    &editor_frame.percent_rows_shift(split.first_percent, -1),
                    pane_tree,
                    split.second,
                )?;
                self.render_frame_h_gap(top_frame)
            }
        }
    }

    fn render_leaf_pane(
        &mut self,
        pane: &Pane,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
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
            cursor::MoveTo(editor_frame.y_row, editor_frame.x_col),
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
                for _ in buffer_col..editor_frame.cols - 1 {
                    queue!(self.stdout, style::Print(" "),)?;
                }
                buffer_row += 1;
                buffer_col = 0;
                queue!(
                    self.stdout,
                    cursor::MoveToColumn(editor_frame.x_col),
                    cursor::MoveDown(1),
                )?;
            } else if buffer_col >= editor_frame.cols {
                buffer_row += 1;
                buffer_col = 0;
                queue!(
                    self.stdout,
                    cursor::MoveToColumn(editor_frame.x_col),
                    cursor::MoveDown(1),
                )?;
            } else {
                queue!(self.stdout, style::Print(char))?;
                buffer_col += 1;
            }

            if char_count == buffer.cursor_index {
                cursor_position = Some((buffer_row, buffer_col));
            }

            if buffer_row >= editor_frame.rows {
                break;
            }
        }

        for _ in buffer_row + 1..editor_frame.rows {
            for _ in buffer_col..editor_frame.cols {
                queue!(self.stdout, style::Print(" "))?;
            }
            queue!(
                self.stdout,
                cursor::MoveDown(1),
                cursor::MoveToColumn(editor_frame.x_col)
            )?;
        }

        if let Some(cursor_position) = cursor_position {
            queue!(
                self.stdout,
                cursor::MoveTo(cursor_position.1, cursor_position.0)
            )?;
        }

        self.stdout.flush()
    }

    fn render_frame_v_gap(&mut self, left_frame: EditorFrame) -> io::Result<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(left_frame.y_row, left_frame.x_col + left_frame.cols),
            style::SetBackgroundColor(Color::DarkGreen),
        )?;
        for _ in 0..left_frame.cols {
            queue!(
                self.stdout,
                style::Print(" "),
                cursor::MoveLeft(1),
                cursor::MoveDown(1),
            )?;
        }
        queue!(self.stdout, style::ResetColor)?;

        Ok(())
    }

    fn render_frame_h_gap(&mut self, top_frame: EditorFrame) -> io::Result<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(top_frame.y_row + top_frame.rows, top_frame.x_col),
            style::SetBackgroundColor(Color::DarkGreen),
        )?;
        for _ in 0..top_frame.rows {
            queue!(
                self.stdout,
                style::Print(" "),
                cursor::MoveRight(1),
                cursor::MoveUp(1),
            )?;
        }
        queue!(self.stdout, style::ResetColor)?;

        Ok(())
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
