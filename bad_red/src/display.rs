// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use crossterm::{
    cursor, queue,
    style::{self, Color, Stylize},
    terminal::{self, *},
};
use regex::{Match, Regex};
use std::io::{self, ErrorKind, Stdout, Write};
use unicode_width::UnicodeWidthChar;

use crate::{
    buffer::{ContentBuffer, EditorBuffer},
    editor_frame::EditorFrame,
    editor_state::{Editor, EditorState},
    pane::{Pane, PaneNode, PaneNodeType, PaneTree, Split},
    styling::{self, Styling},
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
                    self.new_render_leaf_pane(node, pane, node_index, editor_state, editor_frame)?;
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
                editor_frame.rows.saturating_sub(size).saturating_sub(1),
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

    const DEFAULT_STYLE_MATCH: &str = r"^(\W)|(\w*)|(\S*\s*)";
    fn new_render_leaf_pane(
        &mut self,
        pane_node: &PaneNode,
        pane: &Pane,
        pane_id: usize,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
    ) -> io::Result<Option<(u16, u16)>> {
        let mut cursor_screen_location: Option<(u16, u16)> = None;
        let buffer = buffer_by_id(editor_state, pane.buffer_id)?;

        if !needs_render(buffer, pane_node, editor_state, pane_id) {
            return Ok(None);
        }

        let mut current_buffer_line_index = pane.top_line;
        let mut pane_lines_remaining = editor_frame.rows;

        let default_regex = Self::default_style_regex()?;

        crossterm::queue!(
            self.stdout,
            cursor::MoveTo(editor_frame.x_col, editor_frame.y_row)
        )?;

        while pane_lines_remaining > 0 {
            let mut column_index = editor_frame.x_col;
            if let Some(buffer_line_copy) = buffer.content_copy_line(current_buffer_line_index)
            {
                if let Some(mut current_byte_index) =
                    buffer.line_start_byte_index(current_buffer_line_index)
                {
                    self.render_line(
                        buffer_line_copy,
                        buffer,
                        &default_regex,
                        editor_state,
                        editor_frame,
                        pane,
                        &mut current_byte_index,
                        &mut cursor_screen_location,
                        &mut pane_lines_remaining,
                        &mut column_index,
                    )?;
                } else {
                    if cursor_screen_location.is_none() {
                        cursor_screen_location =
                            Some((editor_frame.rows - pane_lines_remaining, 0));
                    }
                };
            }

            crossterm::queue!(
                self.stdout,
                style::Print(
                    vec![
                        " ";
                        (editor_frame.x_col + editor_frame.cols)
                            .saturating_sub(column_index)
                            .into()
                    ]
                    .join("")
                ),
                cursor::MoveDown(1),
                cursor::MoveToColumn(editor_frame.x_col),
            )?;
            pane_lines_remaining = pane_lines_remaining.saturating_sub(1);
            current_buffer_line_index += 1;
        }

        return Ok(cursor_screen_location);
    }

    fn default_style_regex() -> io::Result<Regex> {
        Regex::new(Self::DEFAULT_STYLE_MATCH).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create default regex. {}", e),
            )
        })
    }

    fn render_line(
        &mut self,
        mut buffer_line_copy: String,
        buffer: &EditorBuffer,
        default_regex: &Regex,
        editor_state: &EditorState,
        editor_frame: &EditorFrame,
        pane: &Pane,
        current_byte_index: &mut usize,
        cursor_screen_location: &mut Option<(u16, u16)>,
        pane_lines_remaining: &mut u16,
        column_index: &mut u16,
    ) -> io::Result<()> {
        'line_render: while !buffer_line_copy.is_empty() {
            let mut matched_style: Option<(Match, &str)> = None;
            for style in buffer.styling.style_list.iter().rev() {
                if let Some(found) = style.regex.find(&buffer_line_copy) {
                    matched_style = Some((found, &style.name));
                }
            }
            let (found, style) = matched_style.unwrap_or_else(|| {
                (
                    default_regex.find(&buffer_line_copy).unwrap(),
                    Styling::DEFAULT_NAME,
                )
            });
            let text_style = editor_state.style_map.get(style);
            let rest = buffer_line_copy.split_off(found.end());
            let matched_text = buffer_line_copy;
            buffer_line_copy = rest;

            for matched_char in matched_text.chars() {
                if *current_byte_index == buffer.cursor_byte_index()
                    && cursor_screen_location.is_none()
                {
                    *cursor_screen_location = Some((
                        editor_frame.y_row + editor_frame.rows - *pane_lines_remaining,
                        *column_index,
                    ));
                }

                let char_width =
                    width_for(matched_char, *column_index, editor_state.options.tab_width);
                if char_width == 0 {
                    // Print as utf8 code point to handle display
                    let code_point_literal = matched_char.escape_unicode().to_string();
                    *column_index += code_point_literal
                        .chars()
                        .map(|c| c.width().unwrap_or(1) as u16)
                        .sum::<u16>();
                    crossterm::queue!(self.stdout, style::Print(code_point_literal))?;
                } else if matched_char == '\n' {
                    break 'line_render;
                } else {
                    *column_index += char_width as u16;
                    render_char(&mut self.stdout, char_width, matched_char, text_style)?;
                }

                *current_byte_index += matched_char.len_utf8();
                if *column_index >= (editor_frame.x_col + editor_frame.cols) {
                    if !pane.should_wrap {
                        break;
                    } else {
                        let Some(new_pane_lines_remaining) = pane_lines_remaining.checked_sub(1)
                        else {
                            break 'line_render;
                        };
                        *pane_lines_remaining = new_pane_lines_remaining;
                        *column_index = 0;
                    }
                }
            }
        }

        if *current_byte_index == buffer.cursor_byte_index() {
            *cursor_screen_location = Some((
                editor_frame.y_row + editor_frame.rows - *pane_lines_remaining,
                *column_index,
            ));
        }

        Ok(())
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

fn width_for(character: char, at_col: u16, tab_width: u16) -> usize {
    if character == '\t' {
        (tab_width - at_col % tab_width).into()
    } else {
        character.width().unwrap_or(1)
    }
}

fn render_char(
    stdout: &mut Stdout,
    width: usize,
    character: char,
    text_style: Option<&styling::TextStyle>,
) -> io::Result<()> {
    if character == '\t' {
        if let Some(text_style) = text_style {
            let spacing = " ".repeat(width).with(Color::from(&text_style.foreground));

            let spacing = if let Some(ref background) = &text_style.background {
                spacing.on(Color::from(background))
            } else {
                spacing
            };

            queue!(stdout, style::PrintStyledContent(spacing))?;
        } else {
            queue!(stdout, style::Print(" ".repeat(width)))?;
        }
    } else {
        if let Some(text_style) = text_style {
            let character = character.with(Color::from(&text_style.foreground));
            if let Some(background) = &text_style.background {
                character.on(Color::from(background));
            }

            queue!(stdout, style::PrintStyledContent(character))?;
        } else {
            queue!(stdout, style::Print(character))?;
        }
    }

    Ok(())
}

fn buffer_by_id(editor_state: &EditorState, buffer_id: usize) -> io::Result<&EditorBuffer> {
    editor_state.buffer_by_id(buffer_id).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to find buffer id {} associated with leaf pane",
                buffer_id
            ),
        )
    })
}

fn needs_render(
    buffer: &EditorBuffer,
    pane_node: &PaneNode,
    editor_state: &EditorState,
    pane_id: usize,
) -> bool {
    buffer.is_render_dirty || pane_node.is_dirty || editor_state.active_pane_index == pane_id
}

impl Drop for Display {
    fn drop(&mut self) {
        let _ = self.cleanup_display();
    }
}
