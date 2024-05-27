use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal::{self, *},
};
use std::{
    io::{self, ErrorKind, Stdout, Write},
    iter::Peekable,
};

use crate::{
    editor_frame::EditorFrame,
    editor_state::{Editor, EditorState},
    pane::{Pane, PaneNodeType, PaneTree},
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
        let editor_state = &editor.state;
        let window_size = terminal::window_size()?;
        let editor_frame = EditorFrame {
            x_col: 0,
            y_row: 0,
            rows: window_size.rows,
            cols: window_size.columns,
        };

        let cursor =
            self.render_to_pane(editor_state, &editor_frame, &editor_state.pane_tree, 0)?;
        if let Some((row, col)) = cursor {
            queue!(self.stdout, cursor::MoveTo(col, row))?;
        }

        self.stdout.flush()
    }

    fn render_to_pane(
        &mut self,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
        pane_tree: &PaneTree,
        node_index: usize,
    ) -> io::Result<Option<(u16, u16)>> {
        let node = pane_tree.tree.get(node_index).ok_or(io::Error::new(
            ErrorKind::Other,
            format!("Failed to find pane at index: {}", node_index),
        ))?;

        match &node.node_type {
            PaneNodeType::Leaf(ref pane) => {
                let pane_cursor = self.render_leaf_pane(pane, editor_state, editor_frame)?;
                if editor_state.active_pane_index == node_index {
                    Ok(pane_cursor)
                } else {
                    Ok(None)
                }
            }
            PaneNodeType::VSplit(split) => {
                if !node.is_dirty { return Ok(None) }

                let left_frame = editor_frame.percent_cols(split.first_percent, -1);
                let right_frame = &editor_frame.percent_cols_shift(split.first_percent, -1);

                let left_cursor =
                    self.render_to_pane(editor_state, &left_frame, pane_tree, split.first)?;

                let right_cursor =
                    self.render_to_pane(editor_state, right_frame, pane_tree, split.second)?;
                self.render_frame_v_gap(&left_frame, &right_frame)?;

                Ok(left_cursor.or(right_cursor))
            }
            PaneNodeType::HSplit(split) => {
                if !node.is_dirty { return Ok(None) }

                let top_frame = editor_frame.percent_rows(split.first_percent, -1);
                let bottom_frame = editor_frame.percent_rows_shift(split.first_percent, -1);

                let top_cursor =
                    self.render_to_pane(editor_state, &top_frame, pane_tree, split.first)?;
                let bottom_cursor = self.render_to_pane(
                    editor_state,
                    &editor_frame.percent_rows_shift(split.first_percent, -1),
                    pane_tree,
                    split.second,
                )?;
                self.render_frame_h_gap(&top_frame, &bottom_frame)?;

                Ok(top_cursor.or(bottom_cursor))
            }
        }
    }

    fn render_leaf_pane(
        &mut self,
        pane: &Pane,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
    ) -> io::Result<Option<(u16, u16)>> {
        let Some(buffer) = &editor_state.buffers.get(pane.buffer_id) else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to find buffer id {} associated with pane",
                    pane.buffer_id
                ),
            ));
        };
        if !buffer.is_dirty { return Ok(None) }

        let mut chars = buffer.content.chars().peekable();
        let mut char_count = 0;

        let mut line_count = 0;
        while let Some(char) = chars.peek() {
            if line_count == pane.top_line {
                break;
            }

            if handle_newline(*char, &mut char_count, &mut chars) {
                line_count += 1;
            }
        }

        queue!(
            self.stdout,
            cursor::MoveTo(editor_frame.x_col, editor_frame.y_row),
        )?;
        let mut cursor_position: Option<(u16, u16)> = None;
        for row in editor_frame.y_row..(editor_frame.y_row + editor_frame.rows) {
            let mut did_end_line = false;

            'col_loop: for col in editor_frame.x_col..(editor_frame.x_col + editor_frame.cols) {
                if char_count == buffer.cursor_index && cursor_position.is_none() {
                    cursor_position = Some((row, col));
                }

                let Some(char) = chars.peek() else {
                    for _ in col..(editor_frame.x_col + editor_frame.cols) {
                        queue!(self.stdout, style::Print(" "),)?;
                    }
                    break 'col_loop;
                };
                let char = *char;

                let is_newline = handle_newline(char, &mut char_count, &mut chars);
                if is_newline {
                    did_end_line = true;
                    for _ in col..(editor_frame.x_col + editor_frame.cols) {
                        queue!(self.stdout, style::Print(" "),)?;
                    }
                    break 'col_loop;
                } else {
                    _ = chars.next();
                    char_count += 1;
                    queue!(self.stdout, style::Print(char),)?;
                }
            }

            if !did_end_line {
                while let Some(peeked) = chars.peek() {
                    if handle_newline(*peeked, &mut char_count, &mut chars) {
                        break;
                    } else {
                        _ = chars.next()
                    }
                }
            }
            queue!(
                self.stdout,
                cursor::MoveDown(1),
                cursor::MoveToColumn(editor_frame.x_col)
            )?;
        }

        Ok(cursor_position)
    }

    fn render_frame_v_gap(&mut self, left_frame: &EditorFrame, right_frame: &EditorFrame) -> io::Result<()> {
        queue!(
            self.stdout,
            style::SetBackgroundColor(Color::DarkGreen),
        )?;

        for col in (left_frame.x_col+left_frame.cols)..right_frame.x_col {
            queue!(
                self.stdout,
                cursor::MoveTo(col, left_frame.y_row,),
            )?;
            for _ in left_frame.y_row..(left_frame.y_row + left_frame.rows) {
                queue!(
                    self.stdout,
                    style::Print(" "),
                    cursor::MoveLeft(1),
                    cursor::MoveDown(1),
                )?;
            }
        }

        queue!(self.stdout, style::ResetColor)?;

        Ok(())
    }

    fn render_frame_h_gap(&mut self, top_frame: &EditorFrame, bottom_frame: &EditorFrame) -> io::Result<()> {
        queue!(
            self.stdout,
            style::SetBackgroundColor(Color::DarkGreen),
        )?;

        for row in (top_frame.y_row+top_frame.rows)..bottom_frame.y_row {
            queue!(
                self.stdout,
                cursor::MoveTo(top_frame.x_col, row),
                style::SetBackgroundColor(Color::DarkGreen),
            )?;
            for _ in top_frame.x_col..(top_frame.x_col + top_frame.cols) {
                queue!(
                    self.stdout,
                    style::Print(" "),
                )?;
            }
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
        _ = chars.next();
        *char_count += 1;
        if chars.peek() == Some(&'\n') {
            *char_count += 1;
            _ = chars.next();
        }
        true
    } else if char == '\n' {
        *char_count += 1;
        _ = chars.next();
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
