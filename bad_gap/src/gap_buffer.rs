// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{collections::VecDeque, ops::Index, slice::SliceIndex};

type Iter<'a, T> = std::collections::vec_deque::Iter<'a, T>;

/// A contiguous, growable gap buffer holding lements of type T.
///
/// Intended for efficient insertion of elements before and after the buffer's moveable cursor.
///
/// # Examples
/// ```
/// let mut gap_buffer = GapBuffer::new();
///
/// gap_buffer.push_before_cursor(0);
/// gap_buffer.push_before_cursor(1);
///
/// gap_buffer.push_after_cursor(2);
/// gap_buffer.push_after_cursor(3);
///
/// assert_eq!(
///     gap_buffer.iter().collect(),
///     [0, 1, 2, 3]
/// );
///
/// assert_eq!(
///     gap_buffer.precursor_iter().collect(),
///     [0, 1]
/// );
/// assert_eq!(
///     gap_buffer.precursor_iter().collect(),
///     [2, 3]
/// );
///
/// assert_eq!(
///     gap_buffer.cursor_index(),
///     2
/// );
///
/// gap_buffer.set_cursor(0);
/// gap_buffer.push_before_cursor(4);
/// gap_buffer.push_after_cursor(-1);
///
/// assert_eq!(
///     gap_buffer.iter().collect(),
///     [-1, 0, 1, 2, 3, 4]
/// );
/// ```
///
/// # Cursor
///
/// GapBuffer supports O(1) insertion only at the cursor. The cursor can be moved and set to be any
/// location within the GapBuffer with a cost of O(|I-N|) where I is the current cursor of the buffer
/// and N is the new cursor index. This ability to move the cursor location is what makes GapBuffer
/// efficient for applications where text is repeatedly inserted at the same point in the buffer
/// and where insertions at the cursor is more frequent than moving the cursor. A clear example of
/// this is a text editor where text insertions tend to happen repeatedly at the cursor while
/// entering text. The same efficiency is achieved for deleting at the cursor as well.
///
/// The cursor itself can be thought of as sitting between two elements in the buffer. The
/// convention of this crate is that the buffer at index `I` sits between element `I` and `I-1`.
/// This means the cursor can be at position: `buffer.len()` while still supporting insertion both
/// before and after the cursor.
///
/// For example:
/// ```
/// let mut buf1 = GapBuffer::from([0, 1, 2, 3]);
/// buf1.set_cursor(0);
///
/// let mut buf2 = GapBuffer::from([0, 1, 2, 3]);
/// buf2.set_cursor(2);
/// ```
///
/// With "_" indicating the cursor location
///
/// For buf1:
/// ```text
/// Cursor
///  |
///  v
/// [_ 0, 1, 2, 3]
/// ```
///
/// For buf2:
/// ```text
///       Cursor
///        |
///        v
/// [0, 1, _ 2, 3]
/// ```
///
/// Elements can be pushed before and after the cursor with [push_before_cursor](GapBuffer::push_before_cursor) and
/// [push_after_cursor](GapBuffer::push_after_cursor).
///
/// Elements can be deleted before and after the cursor with [pop_before_cursor](GapBuffer::pop_before_cursor) and
/// [pop_after_cursor](GapBuffer::pop_after_cursor).
///
/// # Indexing
///
/// GapBuffer allows accessing values by index with respect to the buffer's start point, not its
/// cursor.
///
/// ### Examples
/// ```
/// let mut gap_buffer = GapBuffer::from([0, 1, 2, 3]);
/// assert_eq!(gap_buffer[0], 0);
/// gap_buffer.set_cursor(2);
/// assert_eq!(gap_buffer[0], 0);
///
/// ```
/// Use [get](GapBuffer::get) and [get_mut](GapBuffer::get_mut) if you want to check whether the index is within the GapBuffer.
///
pub struct GapBuffer<T> {
    deque: VecDeque<T>,
}

impl<T> GapBuffer<T> {
    /// Creates a new empty GapBuffer with cursor at 0
    ///
    /// ### Examples
    /// ```
    /// let buffer = GapBuffer::new();
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     []
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    /// ```
    pub fn new() -> Self {
        todo!()
    }

    /// Adds a value to the GapBuffer at the index immediately after the cursor. Does not move
    /// the cursor itself.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::new();
    /// buffer.push_after_cursor(0);
    /// buffer.push_after_cursor(1);
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [1, 0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    /// ```
    pub fn push_after_cursor(&mut self, item: T) {
        todo!()
    }

    /// Adds a value to the GapBuffer at the index immediately before the cursor. Moves the cursor
    /// one element forward to stay ahead of the newly inserted element.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::new();
    /// buffer.push_before_cursor(0);
    /// buffer.push_before_cursor(1);
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [0, 1]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     2
    /// );
    /// ```
    pub fn push_before_cursor(&mut self, item: T) {
        todo!()
    }

    /// Removes the value from the GapBuffer at the index immediately after the cursor. Does not
    /// move the cursor. Returns the popped value if one exists.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3])
    /// buffer.set_cursor(2);
    ///
    /// buffer.pop_after_cursor();
    /// buffer.pop_after_cursor();
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [0, 1],
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     2
    /// );
    /// ```
    pub fn pop_after_cursor(&mut self) -> Option<T> {
        todo!()
    }

    /// Removes the value from the GapBuffer at the index immediately before the cursor. Moves the
    /// cursor back one index to take the place of the newly popped value. Returns the popped
    /// value if one exists.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3])
    /// buffer.set_cursor(2);
    ///
    /// buffer.pop_before_cursor();
    /// buffer.pop_before_cursor();
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [2, 3],
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    /// ```
    pub fn pop_before_cursor(&mut self) -> Option<T> {
        todo!()
    }

    /// Returns an iterator over the gap buffer with respect to the buffers intended order, not
    /// relative to any cursor location.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0, 1, 2]);
    ///
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    ///
    /// buffer.set_cursor(2);
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        self.deque.iter()
    }

    /// Returns an iterator over only the elements before the cursor in the gap buffer.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0, 1, 2]);
    ///
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     None,
    /// );
    ///
    /// buffer.set_cursor(2);
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// ```
    pub fn precursor_iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        self.deque.iter()
    }

    /// Returns an iterator over only the elements after the cursor in the buffer.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0, 1, 2]);
    ///
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(0)
    /// )
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(1)
    /// )
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(2)
    /// )
    /// assert_eq!(
    ///     iter.next(),
    ///     None,
    /// );
    ///
    /// buffer.set_cursor(2);
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// ```
    pub fn postcursor_iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        self.deque.iter()
    }

    /// Returns the number of elements currently stored in the gap buffer.
    ///
    /// ### Examples
    /// ```
    /// let buffer = GapBuffer::from([0, 1, 2, 3]);
    ///
    /// assert_eq!(
    ///     buffer.len(),
    ///     4
    /// );
    /// ```
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Changes the cursor location. Runs in O(|I-N|) where I is the current cursor index of the
    /// gap buffer and N is the new index.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0]);
    ///
    /// buffer.push_before_cursor(1);
    /// assert_eq!(
    ///     buffer.iter().collect();
    ///     [1, 0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     1
    /// );
    ///
    /// buffer.set_cursor(0);
    /// buffer.push_before_cursor(2);
    /// assert_eq!(
    ///     buffer.iter().collect();
    ///     [2, 1, 0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     1
    /// );
    /// ```
    pub fn set_cursor(&mut self, index: usize) {
        todo!()
    }

    /// Returns the current cursor index.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0]);
    /// assert_eq!(
    ///     buffer.cursor_index,
    ///     0
    /// );
    ///
    /// buffer.push_after_cursor(1);
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [1, 0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index,
    ///     0
    /// );
    ///
    /// buffer.push_before_cursor(2);
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [2, 1, 0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index,
    ///     1
    /// );
    /// ```
    pub fn cursor_index(&self) -> usize {
        todo!()
    }

    /// Returns the value immediately before the cursor if one exists
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::new();
    /// assert_eq!(
    ///     buffer.precursor(),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(0);
    /// assert_eq!(
    ///     buffer.precursor(),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(1);
    /// assert_eq!(
    ///     buffer.precursor(),
    ///     Some(1)
    /// );
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [1, 0]
    /// );
    /// ```
    pub fn precursor(&self) -> Option<T> {
        self.get_precursor(0)
    }

    /// Returns the value N values before the cursor if one exists
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::new();
    /// assert_eq!(
    ///     buffer.get_precursor(1),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(0);
    /// assert_eq!(
    ///     buffer.get_precursor(1),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(1);
    /// assert_eq!(
    ///     buffer.get_precursor(1),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(2);
    /// assert_eq!(
    ///     buffer.get_precursor(1),
    ///     Some(2)
    /// );
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [2, 1, 0]
    /// );
    /// ```
    pub fn get_precursor(&self, index: usize) -> Option<T> {
        todo!()
    }

    /// Returns the value immediately after the cursor if one exists
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::new();
    /// assert_eq!(
    ///     buffer.postcursor(),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(0);
    /// assert_eq!(
    ///     buffer.postcursor(),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(1);
    /// assert_eq!(
    ///     buffer.postcursor(),
    ///     Some(1)
    /// );
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [0, 1]
    /// );
    /// ```
    pub fn postcursor(&self) -> Option<T> {
        self.get_postcursor(0)
    }

    /// Returns the value N values after the cursor if one exists
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::new();
    /// assert_eq!(
    ///     buffer.get_postcursor(1),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(0);
    /// assert_eq!(
    ///     buffer.get_postcursor(1),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(1);
    /// assert_eq!(
    ///     buffer.get_postcursor(1),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(2);
    /// assert_eq!(
    ///     buffer.get_postcursor(1),
    ///     Some(2)
    /// );
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [0, 1, 2]
    /// );
    /// ```
    pub fn get_postcursor(&self, index: usize) -> Option<T> {
        todo!()
    }

    /// Returns a reference to an element or subslice depending on type of index.
    ///
    /// - If given a position, returns a reference to the element at that position or `None` if out
    /// of bounds
    /// - If given a range, returns the subslice corresponding to that range, or `None` if out of
    /// bounds.
    ///
    /// ### Examples
    /// ```
    /// let buffer = GapBuffer::from([0, 1, 2, 3]);
    ///
    /// assert_eq!(
    ///     buffer.get(0),
    ///     Some(&0)
    /// );
    /// assert_eq!(
    ///     buffer.get(1..3),
    ///     Some(&[1, 2])
    /// );
    /// assert_eq!(
    ///     buffer.get(4),
    ///     None,
    /// );
    /// assert_eq!(
    ///     buffer.get(3..5),
    ///     None
    /// );
    /// ```
    pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output> 
    where I: SliceIndex<[T]> {
        todo!()
    }

    /// Returns a mutable reference to an element or subslice depending on the type of index (see
    /// [get](GapBuffer::get)) or `None` if index is out of bounds.
    ///
    /// ### Examples
    /// ```
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3, 4]);
    ///
    /// if let Some(element) = buffer.get_mut(1) {
    ///     *element = 10;
    /// }
    ///
    /// if let Some(slice) = buffer.get_mut(2..4) {
    ///     *slice[0] = 20;
    ///     *slice[1] = 30;
    /// }
    ///
    /// assert_eq!(
    ///     buffer.iter().collect(),
    ///     [0, 10, 20, 30, 4]
    /// );
    /// ```
    pub fn get_mut<I>(&self, index: I) -> Option<&mut <I as SliceIndex<[T]>>::Output>
    where I: SliceIndex<[T]> {
        todo!()
    }
}

impl<T> Index<usize> for GapBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        todo!()
    }
}

impl<T> From<VecDeque<T>> for GapBuffer<T> {
    fn from(value: VecDeque<T>) -> Self {
        todo!()
    }
}

impl<T> From<Vec<T>> for GapBuffer<T> {
    fn from(value: Vec<T>) -> Self {
        todo!()
    }
}

impl<T> From<&[T]> for GapBuffer<T>
where
    T: Clone
{
    fn from(value: &[T]) -> Self {
        todo!()
    }
}

impl<T, const N: usize> From<[T; N]> for GapBuffer<T> {
    fn from(value: [T; N]) -> Self {
        todo!()
    }
}
