// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use std::{
    collections::{vec_deque, VecDeque},
    iter,
    ops::Index,
    slice::SliceIndex,
};

type Iter<'a, T> = std::collections::vec_deque::Iter<'a, T>;

/// A contiguous, growable gap buffer holding lements of type T.
///
/// Intended for efficient insertion of elements before and after the buffer's moveable cursor.
///
/// # Examples
/// ```
/// use bad_gap::GapBuffer;
///
/// let mut gap_buffer = GapBuffer::new();
///
/// gap_buffer.push_before_cursor(0);
/// gap_buffer.push_before_cursor(1);
///
/// gap_buffer.push_after_cursor(3);
/// gap_buffer.push_after_cursor(2);
///
/// let collected: Vec<_> = gap_buffer.iter().collect();
/// assert_eq!(
///     collected,
///     [&0, &1, &2, &3]
/// );
///
/// let collected: Vec<_> = gap_buffer.precursor_iter().collect();
/// assert_eq!(
///     collected,
///     [&0, &1]
/// );
///
/// let collected: Vec<_> = gap_buffer.postcursor_iter().collect();
/// assert_eq!(
///     collected,
///     [&2, &3]
/// );
///
/// assert_eq!(
///     gap_buffer.cursor_index(),
///     2
/// );
///
/// gap_buffer.set_cursor(0);
/// gap_buffer.push_before_cursor(-2);
/// gap_buffer.push_after_cursor(-1);
///
/// let collected: Vec<_> = gap_buffer.into_iter().collect();
/// assert_eq!(
///     collected,
///     [-2, -1, 0, 1, 2, 3]
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
/// use bad_gap::GapBuffer;
///
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
/// use bad_gap::GapBuffer;
///
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
    start_index: usize,
}

impl<T> GapBuffer<T> {
    /// Creates a new empty GapBuffer with cursor at 0
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let buffer = GapBuffer::<i32>::new();
    ///
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    ///
    /// let collected: Vec<i32> = buffer.into_iter().collect();
    /// let empty: [_; 0] = [];
    ///
    /// assert_eq!(
    ///     collected,
    ///     &empty
    /// );
    /// ```
    pub fn new() -> Self {
        Self {
            deque: VecDeque::new(),
            start_index: 0,
        }
    }

    /// Adds a value to the GapBuffer at the index immediately after the cursor. Does not move
    /// the cursor itself.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new();
    /// buffer.push_after_cursor(0);
    /// buffer.push_after_cursor(1);
    ///
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [1, 0]
    /// );
    /// ```
    pub fn push_after_cursor(&mut self, item: T) {
        self.deque.push_front(item);
        self.start_index += 1;
    }

    /// Adds a value to the GapBuffer at the index immediately before the cursor. Moves the cursor
    /// one element forward to stay ahead of the newly inserted element.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new();
    /// buffer.push_before_cursor(0);
    /// buffer.push_before_cursor(1);
    ///
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     2
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [0, 1]
    /// );
    /// ```
    pub fn push_before_cursor(&mut self, item: T) {
        self.deque.push_back(item);
    }

    /// Removes the value from the GapBuffer at the index immediately after the cursor. Does not
    /// move the cursor. Returns the popped value if one exists.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3]);
    /// buffer.set_cursor(2);
    ///
    /// buffer.pop_after_cursor();
    /// buffer.pop_after_cursor();
    ///
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     2
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [0, 1],
    /// );
    /// ```
    pub fn pop_after_cursor(&mut self) -> Option<T> {
        if self.start_index == 0 {
            None
        } else {
            let popped = self.deque.pop_front();
            self.start_index -= 1;
            popped
        }
    }

    /// Removes the value from the GapBuffer at the index immediately before the cursor. Moves the
    /// cursor back one index to take the place of the newly popped value. Returns the popped
    /// value if one exists.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3]);
    /// buffer.set_cursor(2);
    ///
    /// buffer.pop_before_cursor();
    /// buffer.pop_before_cursor();
    ///
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [2, 3],
    /// );
    /// ```
    pub fn pop_before_cursor(&mut self) -> Option<T> {
        if self.start_index == self.deque.len() {
            None
        } else {
            self.deque.pop_back()
        }
    }

    /// Returns an iterator over the gap buffer with respect to the buffers intended order, not
    /// relative to any cursor location.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2]);
    ///
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// drop(iter);
    ///
    /// buffer.set_cursor(2);
    /// let mut iter = buffer.iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        self.precursor_iter().chain(self.postcursor_iter())
    }

    /// Returns an iterator over only the elements before the cursor in the gap buffer.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2]);
    /// buffer.set_cursor(0);
    ///
    /// let mut iter = buffer.precursor_iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     None,
    /// );
    /// drop(iter);
    ///
    /// buffer.set_cursor(2);
    /// let mut iter = buffer.precursor_iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// ```
    pub fn precursor_iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        self.deque.iter().skip(self.start_index)
    }

    /// Returns an iterator over only the elements after the cursor in the buffer.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2]);
    /// buffer.set_cursor(0);
    ///
    /// let mut iter = buffer.postcursor_iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&0)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&1)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None,
    /// );
    /// drop(iter);
    ///
    /// buffer.set_cursor(2);
    /// let mut iter = buffer.postcursor_iter();
    /// assert_eq!(
    ///     iter.next(),
    ///     Some(&2)
    /// );
    /// assert_eq!(
    ///     iter.next(),
    ///     None
    /// );
    /// ```
    pub fn postcursor_iter(&self) -> impl Iterator<Item = &'_ T> + '_ {
        self.deque.iter().take(self.start_index)
    }

    /// Returns the number of elements currently stored in the gap buffer.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let buffer = GapBuffer::from([0, 1, 2, 3]);
    ///
    /// assert_eq!(
    ///     buffer.len(),
    ///     4
    /// );
    /// ```
    pub fn len(&self) -> usize {
        self.deque.len()
    }

    /// Changes the cursor location. Runs in O(|I-N|) where I is the current cursor index of the
    /// gap buffer and N is the new index.
    ///
    /// Panics if the index provided is an invalid cursor index (i.e., is strictly greater than
    /// length of the buffer).
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0]);
    ///
    /// buffer.push_before_cursor(1);
    ///
    /// let collected: Vec<_> = buffer.iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [&1, &0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     1
    /// );
    ///
    /// buffer.set_cursor(0);
    /// buffer.push_before_cursor(2);
    ///
    /// let collected: Vec<_> = buffer.iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [&2, &1, &0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     1
    /// );
    /// ```
    pub fn set_cursor(&mut self, index: usize) {
        if index > self.deque.len() {
            panic!("Expected cursor index ({}) for set_cursor to be within the bounds of GapBuffer (len: {})", index, self.len());
        }

        let current_cursor = self.cursor_index();

        if index == current_cursor {
            return;
        } else if index > current_cursor {
            // Move cursor towards end of buffer
            let cursor_diff = index - current_cursor;
            self.deque.rotate_left(cursor_diff);
            self.start_index -= cursor_diff;
        } else {
            // Move cursor towards the start of buffer
            let cursor_diff = current_cursor - index;
            self.deque.rotate_right(cursor_diff);
            self.start_index += cursor_diff;
        }
    }

    /// Returns the current cursor index.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0]);
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    ///
    /// buffer.push_after_cursor(1);
    /// let collected: Vec<_> = buffer.iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [&1, &0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     0
    /// );
    ///
    /// buffer.push_before_cursor(2);
    /// let collected: Vec<_> = buffer.iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [&2, &1, &0]
    /// );
    /// assert_eq!(
    ///     buffer.cursor_index(),
    ///     1
    /// );
    /// ```
    pub fn cursor_index(&self) -> usize {
        self.deque.len() - self.start_index
    }

    /// Returns the value immediately before the cursor if one exists
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
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
    ///     Some(&1)
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [1, 0]
    /// );
    /// ```
    pub fn precursor(&self) -> Option<&T> {
        self.get_precursor(0)
    }

    /// Returns the value indexed from N starting at the cursor moving towards the start of the
    /// buffer. Given that the cursor sits between two values in the gap buffer, the value
    /// immediately before the cursor is considered precursor element 0. This makes
    /// `precursor(CursorIndex - 1)` the first element in the gap buffer. `precursor(CursorIndex)`
    /// will always be `None`, analogous to `vec.len() - 1` being the index just off the end of the
    /// vector and always indexing to None.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new();
    /// assert_eq!(
    ///     buffer.get_precursor(0),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(0);
    /// assert_eq!(
    ///     buffer.get_precursor(0),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(1);
    /// assert_eq!(
    ///     buffer.get_precursor(0),
    ///     Some(&1)
    /// );
    ///
    /// buffer.push_before_cursor(2);
    /// assert_eq!(
    ///     buffer.get_precursor(0),
    ///     Some(&2)
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [1, 2, 0]
    /// );
    /// ```
    pub fn get_precursor(&self, index: usize) -> Option<&T> {
        let cursor_index = self.cursor_index();
        if index >= cursor_index {
            None
        } else {
            self.get(cursor_index - index - 1)
        }
    }

    /// Returns the value immediately after the cursor if one exists
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
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
    ///     Some(&1)
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [0, 1]
    /// );
    /// ```
    pub fn postcursor(&self) -> Option<&T> {
        self.get_postcursor(0)
    }

    /// Returns the value indexed from N starting at the cursor moving towards the end of the
    /// buffer. Given that the cursor sits between two values in the gap buffer, the value
    /// immediately following the cursor is considered postcursor element 0. This makes
    /// `postcursor(buffer.len() - CursorIndex - 1)` the last element in the gap buffer.
    /// `precursor(buffer.len() - CursorIndex)` will always be `None`, analogous to
    /// `vec.len() - 1` being the index just off the end of the vector and always indexing to None.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::new();
    /// assert_eq!(
    ///     buffer.get_postcursor(0),
    ///     None
    /// );
    ///
    /// buffer.push_before_cursor(0);
    /// assert_eq!(
    ///     buffer.get_postcursor(0),
    ///     None
    /// );
    ///
    /// buffer.push_after_cursor(1);
    /// assert_eq!(
    ///     buffer.get_postcursor(0),
    ///     Some(&1)
    /// );
    ///
    /// buffer.push_after_cursor(2);
    /// assert_eq!(
    ///     buffer.get_postcursor(0),
    ///     Some(&2)
    /// );
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [0, 2, 1]
    /// );
    /// ```
    pub fn get_postcursor(&self, index: usize) -> Option<&T> {
        let cursor_index = self.cursor_index();
        if index >= self.start_index {
            None
        } else {
            self.get(cursor_index + index)
        }
    }

    /// Returns a reference to an element at the given index, or None if index out of bounds.
    ///
    /// Index is with respect to the beginning of the gap buffer data, not to the cursor.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3]);
    /// buffer.set_cursor(2);
    ///
    /// assert_eq!(
    ///     buffer.get(2),
    ///     Some(&2)
    /// );
    ///
    /// assert_eq!(
    ///     buffer.get(4),
    ///     None
    /// );
    /// ```
    pub fn get(&self, index: usize) -> Option<&T> {
        self.deque_index_from_buffer_index(index)
            .map(|i| self.deque.get(i))
            .flatten()
    }

    /// Returns a mutable reference to an element at the given index, or None if index is out of
    /// bounds.
    ///
    /// Index is with respect to the beginning of the gap buffer data, not to the cursor.
    ///
    /// ### Examples
    /// ```
    /// use bad_gap::GapBuffer;
    ///
    /// let mut buffer = GapBuffer::from([0, 1, 2, 3, 4]);
    /// buffer.set_cursor(3);
    ///
    /// if let Some(element) = buffer.get_mut(1) {
    ///     *element = 10;
    /// }
    ///
    /// if let Some(element) = buffer.get_mut(5) {
    ///     panic!("Will not run. Index provided to get_mut was out of bounds");
    /// }
    ///
    /// let collected: Vec<_> = buffer.into_iter().collect();
    /// assert_eq!(
    ///     collected,
    ///     [0, 10, 2, 3, 4]
    /// );
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.deque_index_from_buffer_index(index)
            .map(|i| self.deque.get_mut(i))
            .flatten()
    }
}

impl<T> GapBuffer<T> {
    /// Returns the index into the deque's current state that matches the index into the buffer's
    /// content expected state (i.e. indexed from 0 in the GapBuffer's content).
    fn deque_index_from_buffer_index(&self, buffer_index: usize) -> Option<usize> {
        if buffer_index >= self.deque.len() {
            None
        } else {
            Some((buffer_index + self.start_index) % self.len())
        }
    }
}

impl<T> Index<usize> for GapBuffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Out of bounds index provided to GapBuffer")
    }
}

impl<T> From<VecDeque<T>> for GapBuffer<T> {
    fn from(value: VecDeque<T>) -> Self {
        let start_index = value.len();
        Self {
            deque: value,
            start_index,
        }
    }
}

impl<T> From<Vec<T>> for GapBuffer<T> {
    fn from(value: Vec<T>) -> Self {
        let start_index = value.len();
        Self {
            deque: VecDeque::from(value),
            start_index,
        }
    }
}

impl<T> From<&[T]> for GapBuffer<T>
where
    T: Clone,
{
    fn from(value: &[T]) -> Self {
        let start_index = value.len();
        Self {
            deque: VecDeque::from(Vec::from(value)),
            start_index,
        }
    }
}

impl<T, const N: usize> From<[T; N]> for GapBuffer<T> {
    fn from(value: [T; N]) -> Self {
        let start_index = value.len();
        Self {
            deque: VecDeque::from(value),
            start_index,
        }
    }
}

impl<'a, T> IntoIterator for &'a GapBuffer<T> {
    type Item = &'a T;

    type IntoIter =
        iter::Chain<iter::Skip<vec_deque::Iter<'a, T>>, iter::Take<vec_deque::Iter<'a, T>>>;

    fn into_iter(self) -> Self::IntoIter {
        let first_section = self.deque.iter().skip(self.start_index);
        let second_section = self.deque.iter().take(self.len() - self.start_index);

        first_section.chain(second_section)
    }
}

impl<T> IntoIterator for GapBuffer<T> {
    type Item = T;

    type IntoIter = vec_deque::IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        // Cursor of len() puts the elements in expected order
        self.set_cursor(self.len());

        self.deque.into_iter()
    }
}
