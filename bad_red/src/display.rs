// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal::{self, *},
};
use std::{
    io::{self, ErrorKind, Stdout, Write},
    iter::Peekable,
};
use unicode_width::UnicodeWidthChar;

use crate::{
    buffer::ContentBuffer,
    editor_frame::EditorFrame,
    editor_state::{Editor, EditorState},
    pane::{Pane, PaneNode, PaneNodeType, PaneTree, Split},
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

    pub fn cleanup_display(&mut self) -> io::Result<()> {
        queue!(self.stdout, LeaveAlternateScreen, cursor::Show)?;

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

        queue!(self.stdout, cursor::SavePosition, cursor::Hide)?;
        let cursor = self.render_to_pane(
            editor_state,
            &editor_frame,
            &editor_state.pane_tree,
            editor_state.pane_tree.root_index(),
        )?;
        queue!(self.stdout, cursor::RestorePosition)?;
        if let Some((row, col)) = cursor {
            queue!(self.stdout, cursor::MoveTo(col, row), cursor::Show)?;
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
        let node = pane_tree
            .tree
            .get(node_index)
            .map(|ni| ni.as_ref())
            .flatten()
            .ok_or(io::Error::new(
                ErrorKind::Other,
                format!("Failed to find pane at index: {}", node_index),
            ))?;

        match &node.node_type {
            PaneNodeType::Leaf(ref pane) => {
                let pane_cursor =
                    self.render_leaf_pane(node, pane, node_index, editor_state, editor_frame)?;
                if editor_state.active_pane_index == node_index {
                    Ok(pane_cursor)
                } else {
                    Ok(None)
                }
            }
            PaneNodeType::VSplit(split) => {
                self.render_v_split(node_index, pane_tree, editor_state, editor_frame, split)
            }
            PaneNodeType::HSplit(split) => {
                self.render_h_split(node_index, pane_tree, editor_state, editor_frame, split)
            }
        }
    }

    fn render_v_split(
        &mut self,
        node_index: usize,
        pane_tree: &PaneTree,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
        split: &Split,
    ) -> io::Result<Option<(u16, u16)>> {
        match split.split_type {
            crate::pane::SplitType::Percent { first_percent } => {
                let left_frame = editor_frame.percent_cols(first_percent, -1);
                let right_frame = &editor_frame.percent_cols_shift(first_percent, -1);

                let left_cursor =
                    self.render_to_pane(editor_state, &left_frame, pane_tree, split.first)?;

                let right_cursor =
                    self.render_to_pane(editor_state, right_frame, pane_tree, split.second)?;
                self.render_frame_v_gap(
                    editor_state.active_pane_index == node_index,
                    &left_frame,
                    &right_frame,
                )?;

                Ok(left_cursor.or(right_cursor))
            }
            crate::pane::SplitType::FirstFixed { size } => self.render_fixed_v_split(
                node_index,
                pane_tree,
                editor_state,
                editor_frame,
                split,
                size,
            ),
            crate::pane::SplitType::SecondFixed { size } => self.render_fixed_v_split(
                node_index,
                pane_tree,
                editor_state,
                editor_frame,
                split,
                editor_frame.cols - size - 1,
            ),
        }
    }

    fn render_fixed_v_split(
        &mut self,
        node_index: usize,
        pane_tree: &PaneTree,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
        split: &Split,
        first_fixed: u16,
    ) -> io::Result<Option<(u16, u16)>> {
        let left_frame = editor_frame.with_cols(first_fixed);
        let right_frame = &editor_frame
            .with_cols(editor_frame.cols - first_fixed - 1)
            .with_x_col(editor_frame.x_col + first_fixed + 1);

        let left_cursor = self.render_to_pane(editor_state, &left_frame, pane_tree, split.first)?;

        let right_cursor =
            self.render_to_pane(editor_state, right_frame, pane_tree, split.second)?;
        self.render_frame_v_gap(
            editor_state.active_pane_index == node_index,
            &left_frame,
            &right_frame,
        )?;

        Ok(left_cursor.or(right_cursor))
    }

    fn render_h_split(
        &mut self,
        node_index: usize,
        pane_tree: &PaneTree,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
        split: &Split,
    ) -> io::Result<Option<(u16, u16)>> {
        match split.split_type {
            crate::pane::SplitType::Percent { first_percent } => {
                let top_frame = editor_frame.percent_rows(first_percent, -1);
                let bottom_frame = editor_frame.percent_rows_shift(first_percent, -1);

                let top_cursor =
                    self.render_to_pane(editor_state, &top_frame, pane_tree, split.first)?;
                let bottom_cursor = self.render_to_pane(
                    editor_state,
                    &editor_frame.percent_rows_shift(first_percent, -1),
                    pane_tree,
                    split.second,
                )?;
                self.render_frame_h_gap(
                    editor_state.active_pane_index == node_index,
                    &top_frame,
                    &bottom_frame,
                )?;

                Ok(top_cursor.or(bottom_cursor))
            }
            crate::pane::SplitType::FirstFixed { size } => self.render_fixed_h_split(
                node_index,
                pane_tree,
                editor_state,
                editor_frame,
                split,
                size,
            ),
            crate::pane::SplitType::SecondFixed { size } => self.render_fixed_h_split(
                node_index,
                pane_tree,
                editor_state,
                editor_frame,
                split,
                editor_frame.rows - size - 1,
            ),
        }
    }

    fn render_fixed_h_split(
        &mut self,
        node_index: usize,
        pane_tree: &PaneTree,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
        split: &Split,
        first_fixed: u16,
    ) -> io::Result<Option<(u16, u16)>> {
        let top_frame = editor_frame.with_rows(first_fixed);
        let bottom_frame = &editor_frame
            .with_rows(editor_frame.rows - first_fixed - 1)
            .with_y_row(editor_frame.y_row + first_fixed + 1);

        let top_cursor = self.render_to_pane(editor_state, &top_frame, pane_tree, split.first)?;

        let bottom_cursor =
            self.render_to_pane(editor_state, bottom_frame, pane_tree, split.second)?;
        self.render_frame_h_gap(
            editor_state.active_pane_index == node_index,
            &top_frame,
            &bottom_frame,
        )?;

        Ok(top_cursor.or(bottom_cursor))
    }

    fn scan_to_first_line<I>(&self, byte_count: &mut usize, pane: &Pane, chars: &mut Peekable<I>)
    where
        I: Iterator<Item = char>,
    {
        let mut line_count = 0;
        while let Some(char) = chars.peek() {
            let char_bytes = char.len_utf8();
            if line_count == pane.top_line {
                break;
            }

            if handle_newline(*char, byte_count, chars) {
                line_count += 1;
            } else {
                *byte_count += char_bytes;
                _ = chars.next();
            }
        }
    }

    fn render_leaf_pane(
        &mut self,
        pane_node: &PaneNode,
        pane: &Pane,
        pane_id: usize,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
    ) -> io::Result<Option<(u16, u16)>> {
        let buffer = editor_state.buffer_by_id(pane.buffer_id).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Failed to find buffer id {} associated with leaf pane",
                    pane.buffer_id
                ),
            )
        })?;

        if !buffer.is_render_dirty
            && !pane_node.is_dirty
            && editor_state.active_pane_index != pane_id
        {
            return Ok(None);
        }

        let mut chars = buffer.chars().peekable();
        let mut byte_count = 0;
        self.scan_to_first_line(&mut byte_count, pane, &mut chars);

        queue!(
            self.stdout,
            cursor::MoveTo(editor_frame.x_col, editor_frame.y_row)
        )?;

        let mut is_cursor_offscreen = false;
        let mut cursor_position: Option<(u16, u16)> = None;
        for row in editor_frame.y_row..(editor_frame.y_row + editor_frame.rows) {
            let mut col = editor_frame.x_col;
            while col < editor_frame.x_col + editor_frame.cols {
                if byte_count == buffer.cursor_byte_index() && cursor_position.is_none() {
                    cursor_position = Some((row, col));
                }

                let Some(peeked) = chars.peek().map(|c| *c) else {
                    break;
                };

                if handle_newline(peeked, &mut byte_count, &mut chars) {
                    break;
                }

                let char_width = width_for(peeked, col, editor_state.options.tab_width);
                if char_width == 0 {
                    // Print as utf8 code point to handle display
                    let code_point_literal = peeked.escape_unicode().to_string();
                    col += code_point_literal
                        .chars()
                        .map(|c| c.width().unwrap_or(1) as u16)
                        .sum::<u16>();
                    queue!(self.stdout, style::Print(code_point_literal))?;
                } else {
                    col += char_width as u16;
                    render_char(&mut self.stdout, char_width, peeked)?;
                }

                byte_count += peeked.len_utf8();
                _ = chars.next();
            }

            let line_clear = if col < (editor_frame.x_col + editor_frame.cols) {
                vec![" "; (editor_frame.x_col + editor_frame.cols - col).into()].join("")
            } else {
                if !pane.should_wrap {
                    if byte_count == buffer.cursor_byte_index() {
                        is_cursor_offscreen = true;
                    }
                    while let Some(peeked) = chars.peek().map(|c| *c) {
                        if handle_newline(peeked, &mut byte_count, &mut chars) {
                            break;
                        }

                        if byte_count + 1 == buffer.cursor_byte_index() {
                            is_cursor_offscreen = true;
                        }

                        byte_count += peeked.len_utf8();
                        _ = chars.next();
                    }
                }
                "".to_string()
            };
            queue!(
                self.stdout,
                style::Print(line_clear),
                cursor::MoveDown(1),
                cursor::MoveToColumn(editor_frame.x_col)
            )?;
        }

        drop(chars);

        if is_cursor_offscreen {
            Ok(None)
        } else {
            Ok(cursor_position)
        }
    }

    fn render_frame_v_gap(
        &mut self,
        is_active: bool,
        left_frame: &EditorFrame,
        right_frame: &EditorFrame,
    ) -> io::Result<()> {
        let color = if is_active {
            Color::DarkBlue
        } else {
            Color::DarkGreen
        };
        queue!(self.stdout, style::SetBackgroundColor(color))?;

        for col in (left_frame.x_col + left_frame.cols)..right_frame.x_col {
            queue!(self.stdout, cursor::MoveTo(col, left_frame.y_row,),)?;
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

    fn render_frame_h_gap(
        &mut self,
        is_active: bool,
        top_frame: &EditorFrame,
        bottom_frame: &EditorFrame,
    ) -> io::Result<()> {
        let color = if is_active {
            Color::DarkBlue
        } else {
            Color::DarkGreen
        };
        queue!(self.stdout, style::SetBackgroundColor(color),)?;

        for row in (top_frame.y_row + top_frame.rows)..bottom_frame.y_row {
            queue!(self.stdout, cursor::MoveTo(top_frame.x_col, row),)?;
            for _ in top_frame.x_col..(top_frame.x_col + top_frame.cols) {
                queue!(self.stdout, style::Print(" "),)?;
            }
        }
        queue!(self.stdout, style::ResetColor)?;

        Ok(())
    }
}

fn handle_newline<I>(char: char, byte_count: &mut usize, chars: &mut Peekable<I>) -> bool
where
    I: Iterator<Item = char>,
{
    if char == '\r' {
        _ = chars.next();
        *byte_count += 1;
        if chars.peek() == Some(&'\n') {
            *byte_count += 1;
            _ = chars.next();
        }
        true
    } else if char == '\n' {
        *byte_count += 1;
        _ = chars.next();
        true
    } else {
        false
    }
}

fn width_for(character: char, at_col: u16, tab_width: u16) -> usize {
    if character == '\t' {
        (tab_width - at_col % tab_width).into()
    } else {
        character.width().unwrap_or(1)
    }
}

fn render_char(stdout: &mut Stdout, width: usize, character: char) -> io::Result<()> {
    if character == '\t' {
        queue!(stdout, style::Print(" ".repeat(width)))?;
    } else {
        queue!(stdout, style::Print(character))?;
    }

    Ok(())
}

impl Drop for Display {
    fn drop(&mut self) {
        let _ = self.cleanup_display();
    }
}
