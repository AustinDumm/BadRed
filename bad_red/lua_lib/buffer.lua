-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

package.preload["buffer"] = function(modname, _)
    local P = {}

    P.new = red.doc.build_fn(
        function(self, id)
            local instance = { _id = id }
            setmetatable(instance, self)
            self.__index = self
            return instance
        end,
        "new",
        [[
Creates a new script buffer object from a given id.
]],
        [[
Does not open or create a buffer within the editor with the matching id. Intended mostly for internal use.

Ex: `
    local existing_buffer_id = 2
    local new_buf = buffer:new(existing_buffer_id)
`
]],
        [[
Buffer - object with the provided buffer id
]],
        [[
self: Buffer - Object to instantiate the buffer table from. self is used as __index metatable for buffer inheritance.
]],
        [[
id: non-negative integer - The ID value the editor runtime uses to identify a particular buffer.
]]
    )

    P.id = red.doc.build_fn(
        function(self)
            return self._id or P:current()._id
        end,
        "id",
        [[
ID for the current buffer object.
]],
        [[
If no id is set on this buffer (i.e., receiver of this function call is the Buffer class table), returns the id of the buffer currently being edited.
]],
        [[
non-negative integer - The ID value for this buffer object or the ID of the currently active buffer in the editor.
]],
        [[
self: Buffer - Object to get the ID from.
]]
    )

    P.open = red.doc.build_fn(
        function(self)
            local id = coroutine.yield(red.call.buffer_open())
            return self:new(id)
        end,
        "open",
        [[
Creates a new buffer in the editor of default type and empty content.
]],
        nil,
        [[
Buffer - Object with the id of the new buffer.
]],
        [[
self: Buffer - Class to instantiate the new buffer from.
]]
    )

    P.close = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.buffer_close(self:id()))
        end,
        "close",
        [[
Deletes this buffer from editor state by the ID of this buffer object
]],
        nil,
        [[
nil
]],
        [[
self: Buffer - Object to close. If no buffer ID set on this object, will close the currently active buffer
]]
    )

    P.current = red.doc.build_fn(
        function(self)
            local id = coroutine.yield(red.call.current_buffer_id())
            return self:new(id)
        end,
        "current",
        [[
Retrieves the active buffer data from the editor.
]],
        nil,
        [[
Buffer - A new object representing the active buffer in the editor.
]],
        [[
self: Buffer - Class table to instantiate the new buffer table from.
]]
    )

    P.insert = red.doc.build_fn(
        function(self, content)
            coroutine.yield(red.call.buffer_insert(self:id(), content))
        end,
        "insert",
        [[
Inserts provided content text into this buffer.
]],
        nil,
        [[
nil
]],
        [[
self: Buffer - Buffer object to insert text into. If no buffer ID is set on this object, inserts into active buffer.
]],
        [[
content: string - The string content to insert. Will be inserted into `self` as utf8 encoded bytes.
]]
    )

    P.delete = red.doc.build_fn(
        function(self, count)
            return coroutine.yield(red.call.buffer_delete(self:id(), count))
        end,
        "delete",
        [[
Deletes a number of characters starting at the current cursor character.
]],
        nil,
        [[
string - The entire text that was deleted from the buffer.
]],
        [[
self: Buffer - Buffer object to delete text from. If no buffer ID is set on this object, deletes from active buffer.
]],
        [[
count: non-negative integer - The number of unicode code points to delete starting at the current cursor.
]]
    )

    P.cursor = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.buffer_cursor(self:id()))
        end,
        "cursor",
        [[
Get the byte index of the current buffer's cursor
]],
        nil,
        [[
non-negative integer - The byte index of the cursor. Cursor sits between two bytes in the buffer specifically to the left of the byte at the same index. In other words, cursor at byte index 0 sits between the start of the buffer and the character starting at byte 0. In normal function, cursor should always return a number sitting on a character boundary with characters in utf8 byte encoding,
]],
        [[
self: Buffer - Buffer object whose cursor is returned. If no buffer ID is set on this object, gets the cursor of the active buffer.
]]
    )

    P.cursor_right = red.doc.build_fn(
        function(self, count, skip_newlines)
            local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_char(self:id(), count))
            local cursor_char = self:content_at(new_cursor, 1)

            if skip_newlines then
                local line_content = self:line_content(self:cursor_line())

                if cursor_char == "\n" and line_content ~= "\n" then
                    return coroutine.yield(red.call.buffer_index_moved_by_char(self:id(), new_cursor, 1))
                end
            end

            return new_cursor
        end,
        "cursor_right",
        [[
Returns the byte index `count` number of characters to the right of the current cursor for this buffer.
]],
        [[
Provides options for dealing with newlines.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose cursor should be moved. If no buffer ID is set on this object, returns the byte index of cursor of the active buffer moved by `count` characters.
]],
        [[
count: integer - Number of characters to move. Note that it is possible the returned cursor byte index will increase by more bytes than the count provided if moving over utf8 characters that are encoded in more than 1 byte. For this reason, use cursor movement functions instead of manually adding to a cursor byte value. Will stop at the end of the buffer if count moves the cursor more than the remaining number of characters in the buffer.
]],
        [[
skip_newlines: bool = false - Should the cursor be allowed to stop over a newline character. If true, will move to the next character to the right if a newline was landed on.
]]
    )

    P.cursor_left = red.doc.build_fn(
        function(self, count, skip_newlines)
            local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_char(self:id(), -count))
            local cursor_char = self:content_at(new_cursor, 1)

            if skip_newlines then
                local line_content = self:line_content(self:cursor_line())

                if cursor_char == "\n" and line_content ~= "\n" then
                    return coroutine.yield(red.call.buffer_index_moved_by_char(self:id(), new_cursor, -1))
                end
            end

            return new_cursor
        end,
        "cursor_left",
        [[
Returns the byte index `count` number of characters to the left of the current cursor for this buffer.
]],
        [[
Provides options for dealing with newlines.
]],
        [[
non-negative integer - The byte index for the character preceeding the current cursor by `count` characters.
]],
        [[
self: Buffer - Buffer object whose cursor should be moved. If no buffer ID is set on this object, returns the byte index of the cursor of the active buffer moved by `count` characters.
]],
        [[
count: integer - Number of characters to move. Note that it is possible the cursor will decrease by more bytes than the count provided if moving over utf8 characters that are encoded in more than 1 byte. For this reason, use cursor movement functions instead of manually adding to a cursor byte value. Will stop at the beginning of the buffer if count would move the cursor past the beginning character.
]],
        [[
skip_newlines: bool = false - Should the cursor be allowed to stop over a newline character. If true, will move to the next character to the left if a newline was landed on.
]]
    )

    P.index_left = red.doc.build_fn(
        function(self, index, count)
            return coroutine.yield(red.call.buffer_index_moved_by_char(self:id(), index, -count))
        end,
        [[
Returns the byte index `count` number of characters to the left of the character at `index`.
[[
Errs if `index` is not on a valid character boundary in this buffer. It is recommended that `index` only be retrieved from the return of this function and other character-boundary-respecting functions. `0` is guaranteed to be a valid `index.`
]],
        [[
non-negative integer - The byte index for the character preceeding the character at `index` by `count` characters.
]],
        [[
self: Buffer - Buffer object for which the index should be moved. If no buffer ID is set on this object, runs this function with the active buffer.
]],
        [[
index: non-negative integer - The byte index for the character to start calculating with. Must be on a character boundary.
]],
        [[
count: integer - Number of characters to move. Note that it is possible the cursor will decrease by more bytes than the count provided if moving over utf8 characters that are encoding in more than 1 byte.
]]
    )

    P.index_right = red.doc.build_fn(
        function(self, index, count)
            return coroutine.yield(red.call.buffer_index_moved_by_char(self:id(), index, count))
        end,
        [[
Returns the byte index `count` number of characters to the right of the character at `index`.
[[
Errs if `index` is not on a valid character boundary in this buffer. It is recommended that `index` only be retrieved from the return of this function and other character-boundary-respecting functions. `0` is guaranteed to be a valid `index.`
]],
        [[
non-negative integer - The byte index for the character preceeding the character at `index` by `count` characters.
]],
        [[
self: Buffer - Buffer object for which the index should be moved. If no buffer ID is set on this object, runs this function with the active buffer.
]],
        [[
index: non-negative integer - The byte index for the character to start calculating with. Must be on a character boundary.
]],
        [[
count: integer - Number of characters to move. Note that it is possible the cursor will decrease by more bytes than the count provided if moving over utf8 characters that are encoding in more than 1 byte.
]]
    )

    P.cursor_up = red.doc.build_fn(
        function(self, count, skip_newlines)
            local current_line = self:cursor_line()
            local to_line = current_line - count

            if to_line < 0 then
                self:set_cursor(0)
            else
                self:set_cursor_line(current_line - 1)
            end

            if skip_newlines and self:cursor_content() == "\n" and self:cursor_line_content() ~= "\n" then
                self:set_cursor(self:cursor_left(1), true)
            end
        end,
        "cursor_up",
        [[
Moves the cursor a certain number of lines up.
]],
        [[
Retains the column index of the cursor if the new line landed on is shorter than the previous column index. Provides options for dealing with newlines.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose cursor should be moved. If no buffer ID is set on this object, moves the cursor of the active buffer.
]],
        [[
count: integer - Number of lines to move. Will stop at the beginning of the buffer if count would move the cursor past the beginning character.
]],
        [[
skip_newlines: bool = false - Should the cursor be allowed to stop over a newline character. If true, will move to the next character to the left if a newline was landed on.
]]
    )

    P.cursor_down = red.doc.build_fn(
        function(self, count, skip_newlines)
            local current_line = self:cursor_line()
            local to_line = current_line + count
            local line_count = self:lines()

            if to_line >= line_count then
                self:set_cursor(self:length())
            else
                self:set_cursor_line(current_line + 1)
            end

            if skip_newlines and self:cursor_content() == "\n" and self:cursor_line_content() ~= "\n" then
                self:set_cursor(self:cursor_left(1), true)
            end
        end,
        "cursor_down",
        [[
Moves the cursor a certain number of lines down.
]],
        [[
Retains the column index of the cursor if the new line landed on is shorter than the previous column index. Provides options for dealing with newlines.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose cursor should be moved. If no buffer ID is set on this object, moves the cursor of the active buffer.
]],
        [[
count: integer - Number of lines to move. Will stop at the beginning of the buffer if count would move the cursor past the beginning character.
]],
        [[
skip_newlines: bool = false - Should the cursor be allowed to stop over a newline character. If true, will move to the next character to the left if a newline was landed on.
]]
    )

    local function is_whitespace(char)
        return string.match(char, "%s") ~= nil
    end

    local function is_non_alphanumeric(char)
        return string.match(char, "%W") ~= nil
    end

    local function is_alphanumeric(char)
        return string.match(char, "%w") ~= nil
    end

    local function get_word_split(only_whitespace, is_alphanumeric_word)
        if only_whitespace then
            return is_whitespace
        else
            if is_alphanumeric_word then
                return is_non_alphanumeric
            else
                return is_alphanumeric
            end
        end
    end

    local function move_index_to_word(self, current_index, move_left)
        local shift_char
        if move_left then
            shift_char = -1
        else
            shift_char = 1
        end

        while true do
            local content = self:content_at(current_index, 1)
            if content ~= nil and string.match(content, "%s") == nil then
                return current_index
            end

            current_index = self:index_right(current_index, shift_char)
        end
    end

    P.cursor_word_start = red.doc.build_fn(
        function(self, only_whitespace)
            local current_index = self:cursor()
            current_index = self:index_left(current_index, 1)
            current_index = move_index_to_word(self, current_index, true)

            local is_alphanumeric_word = is_alphanumeric(self:content_at(current_index, 1))
            local is_split = get_word_split(only_whitespace, is_alphanumeric_word)

            local character_index = self:index_left(current_index, 1)
            while current_index ~= 0 and not is_split(self:content_at(character_index, 1)) do
                current_index = character_index
                character_index = self:index_left(current_index, 1)
            end

            return current_index
        end,
        "cursor_word_start",
        [[
Returns the byte index for the starting character of the word the cursor is on.
]],
        [[
Provides an option for whether words should be considered split by only whitespace, or by any change in character type. For example, if the word is alphanumeric, the first non-alphanumeric character found to the left marks the first non-word character and vice versa. If the cursor is already on the first character of this word, returns the start index of the preceeding word.
]],
        [[
non-negative integer - The index of the first character of the word the cursor is on.
]],
        [[
only_whitespace: bool = false - If false, any character of the opposite type (alphanumeric and non-alphanumeric) is considered to split words. If true, only whitespace characters are considered to split words.
]]
    )

    P.cursor_word_end = red.doc.build_fn(
        function(self, only_whitespace)
            local current_index = self:cursor()
            local content_size = self:length()

            current_index = self:index_right(current_index, 1)
            if current_index >= content_size then
                return current_index
            end

            current_index = move_index_to_word(self, current_index, false)

            local is_alphanumeric_word = is_alphanumeric(self:content_at(current_index, 1))
            local is_split = get_word_split(only_whitespace, is_alphanumeric_word)

            local character_index = self:index_right(current_index, 1)
            while character_index < content_size and not is_split(self:content_at(character_index, 1)) do
                current_index = character_index
                character_index = self:index_right(current_index, 1)
            end

            return current_index
        end,
        "cursor_word_end",
        [[
Returns the byte index for the ending character of the word the cursor is on.
]],
        [[
Provides an option for whether words should be considered split by only whitespace, or by any change in character type. For example, if the word is alphanumeric, the first non-alphanumeric character found to the right marks the first non-word character and vice versa. If the cursor is already on the last character of this word, returns the end index of the succeeding word.
]],
        [[
non-negative integer - The index of the ending character of the word the cursor is on.
]],
        [[
only_whitespace: bool = false - If false, any character of the opposite type (alphanumeric and non-alphanumeric) is considered to split words. If true, only whitespace characters are considered to split words).
]]
    )

    P.cursor_next_word_start = red.doc.build_fn(
        function(self, only_whitespace)
            local length = self:length()
            local index = self:cursor()
            if index >= length then
                return index
            end

            local is_alphanum = is_alphanumeric(self:content_at(index, 1))
            local is_split = get_word_split(only_whitespace, is_alphanum)

            index = self:index_right(index, 1)
            while index < length and not is_split(self:content_at(index, 1)) do
                index = self:index_right(index, 1)
            end

            while index < length and is_whitespace(self:content_at(index, 1)) do
                index = self:index_right(index, 1)
            end

            return index
        end,
        "cursor_next_word_start",
        [[
Returns the byte index for the starting character of the word following the word the cursor is currently on.
]],
        [[
Provides an option for whether words should be considered split by only whitespace, or by any change in character type. For example, if the word is alphanumeric, the first non-alphanumeric character found to the right marks the first non-word character and vice versa.
]],
        [[
non-negative integer - The index of the first character in the word following the word the cursor is on.
]],
        [[
only_whitespace: bool = false - If false, any character of the opposite type (alphanumeric and non-alphanumeric) is considered to split words. If true, only whitespace characters are considered to split words).
]]
    )

    P.cursor_line = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.buffer_cursor_line(self:id()))
        end,
        "cursor_line",
        [[
Gets the current line index of the cursor.
]],
        nil,
        [[
nil
]],
        [[
self: Buffer - Buffer object whose cursor line is returned. If no buffer ID is set on this object, gets the line index of the active buffer.
]]
    )

    P.set_cursor = red.doc.build_fn(
        function(self, index, keep_col_index)
            coroutine.yield(red.call.buffer_set_cursor(self:id(), index, keep_col_index))
        end,
        "set_cursor",
        [[
Sets the byte index of thee cursor for this buffer.
]],
        [[
Cursor index must be at a character boundary as defined by utf8 encoding. This requirement is not checked or enforced by the set_cursor call. Meant for use with a cursor value retrieved from related cursor functions or for internal use. For moving the cursor by character, see: `cursor_left`, `cursor_right`, `cursor_up`, and `cursor_down`.

One exception to this is setting the cursor index to 0 or to `length()` will safely set the cursor to the beginning or end of the buffer, respectively. This is guaranteed to be at a character boundary.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose cursor should be set. If no buffer ID is set on this object, gets the line index of the active buffer.
]],
        [[
index: non-negative integer - The byte index the cursor should be set to. Note that this is expected to land on a character boundary. Setting the index to a value which splits a multi-byte utf8 character will lead to undefined behavior.
]],
        [[
keep_col_index: bool = false - Should the set of the cursor retain the column index. Useful for when a line is shorter than the ideal column index the cursor should be set to.
]]
    )

    P.set_cursor_line = red.doc.build_fn(
        function(self, line)
            coroutine.yield(red.call.buffer_set_cursor_line(self:id(), line))
        end,
        "set_cursor_line",
        [[
Moves the cursor to the given line number.
]],
        [[
Attempts to set the cursor as close as possible to its column index before the move was made. Does not update the column index for the cursor.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose cursor should be set. If no buffer ID is set on this object, sets the line index of the active buffer.
]],
        [[
line: non-negative integer - The line number the cursor should be set to.
]]
    )

    P.length = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.buffer_length(self:id()))
        end,
        "length",
        [[
Gets the length in bytes of the content in this buffer.
]],
        nil,
        [[
non-negative integer - The length in bytes of this buffer.
]],
        [[
self: Buffer - Buffer object whose length is returned. If no buffer ID is set on this object, returns the length of the active buffer.
]]
    )

    P.lines = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.buffer_line_count(self:id()))
        end,
        "lines",
        [[
Gets the number of lines of the content of this buffer.
]],
        nil,
        [[
non-negative integer - The number of lines in this buffer.
]],
        [[
self: Buffer - Buffer object whose number of lines is returned. If no buffer ID is set on this object, returns the number of lines in the active buffer.
]]
    )

    P.content = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.buffer_content(self:id()))
        end,
        "content",
        [[
Gets a Lua String copy of the content of this buffer.
]],
        [[
Note: As this is a copy, calling this function is O(n) where n is the number of bytes in the buffer. Calling this too frequently on a large buffer will slow down the editor considerably.
]],
        [[
string - A copy of the content of this buffer.
]],
        [[
self: Buffer - Buffer object whose content should be copied and returned. If no buffer ID is set on this object, returns a copy of the content of the active buffer.
]]
    )

    P.content_at = red.doc.build_fn(
        function(self, byte_index, char_length)
            return coroutine.yield(red.call.buffer_content_at(self:id(), byte_index, char_length))
        end,
        "content_at",
        [[
Gets a Lua String copy of a substring of this buffer.
]],
        [[
Note: As this is a copy, calling this function is O(n) where n is the number of bytes required to copy `char_length` number of chars. Calling this too frequently with a large number of characters will slow down the editor considerably.
]],
        [[
string - A copy of a substring of the content of this buffer.
]],
        [[
self: Buffer - Buffer object whose content should be copied and returned. If no buffer ID is set on this object, returns a copy of the content of the active buffer.
]],
        [[
byte_index: non-negative integer - Starting index at which to copy the substring. Must be on a character boundary of the utf8-encoded bytes within the buffer. Refrain from providing numbers for byte_index unless they are 0, `length()` or provided from the buffer functions which return known byte indices at character boundaries.
]],
        [[
char_length: non-negative integer - Number of characters to copy.
]]
    )

    P.line_content = red.doc.build_fn(
        function(self, line_index)
            return coroutine.yield(red.call.buffer_line_content(self:id(), line_index))
        end,
        "line_content",
        [[
Get a Lua String copy of the characters of a certain line in this buffer.
]],
        nil,
        [[
string - A Lua String copy of the content at the line index provided for this buffer.
]],
        [[
self: Buffer - Buffer object whose line content should be copied and returned. If no buffer ID is set on this object, returns a copy of the line content from the active buffer.
]],
        [[
line_index: non-negative integer - The line index of the line whose content should be copied. Must be [0, `self:lines()`].
]]
    )

    P.cursor_line_content = red.doc.build_fn(
        function(self)
            return self:line_content(self:cursor_line())
        end,
        "cursor_line_content",
        [[
Get a Lua String copy of the line the cursor is currently on.
]],
        nil,
        [[
string - A Lua String copy of the content of the cursor's current line.
]],
        [[
self: Buffer - Buffer object whose cursor line content should be copied and returned. If no buffer ID is set on this object, returns a copy of the line content from the active buffer at its current cursor line.
]]
    )

    P.cursor_content = red.doc.build_fn(
        function(self)
            return self:content_at(self:cursor(), 1)
        end,
        "cursor_content",
        [[
Get the character at the current cursor for this buffer.
]],
        nil,
        [[
string - A Lua String copy of the character at this buffer's current cursor.
]],
        [[
self: Buffer - Buffer object whose cursor character should be copied and returned. If no buffer ID is set on this object, returns a copy of the character in the active buffer under its current cursor.
]]
    )

    P.clear = red.doc.build_fn(
        function(self)
            local content_length = self:length()
            self:set_cursor(0)
            return self:delete(content_length)
        end,
        "clear",
        [[
Clears all content out of this buffer
]],
        nil,
        [[
string - A Lua String of all of the content that was removed from the buffer.
]],
        [[
self: Buffer - Buffer object whose content should be cleared. If no buffer ID is set on this object, clears the content from the active buffer and returns cleared content as a Lua String.
]]
    )

    P.run_as_script = red.doc.build_fn(
        function(self)
            local content = self:content()
            coroutine.yield(red.call.run_script(content))
        end,
        "run_as_script",
        [[
Spawns a new Lua script with this buffer's content as the script text.
]],
        [[
Note: Makes a copy of this buffer's content to run.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose content should be used to spawn the new script. If no buffer ID is set on this object, copies and spawns the active buffer's content as the Lua script.
]]
    )

    P.execute = red.doc.build_fn(
        function(self)
            self:run_as_script()
            self:clear()
        end,
        "execute",
        [[
Spawns a new Lua script with this buffer's content as the script text.
]],
        [[
Consumes and clears the content out of this buffer.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose content should be consumed and used to spawn the new script. If no buffer ID is set on this object, consumes and spawns the active buffer's content as the Lua script.
]]
    )

    P.link_file = red.doc.build_fn(
        function(self, file, preserve_buffer)
            coroutine.yield(red.call.buffer_link_file(self:id(), file:id(), not preserve_buffer))
        end,
        "link_file",
        [[
Connects this buffer to an open file object managed by the editor.
]],
        [[
Optionally overwrites this buffer with the file's current content on disk. Fails if this buffer is already linked with a file.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object to be linked with the given file. If no buffer ID is set on this object, attempts to link the active buffer to the file instead.
]],
        [[
file: File - File object to link this buffer with. Must be file as created from the `red.file` package.
]],
        [[
preserve_buffer: bool = false - Should this buffer content be kept during link. If false, will clear this buffer's content and populate with the given File's content as it is on disk.
]]
    )

    P.unlink_file = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.buffer_unlink_file(self:id()))
        end,
        "unlink_file",
        [[
Disconnects this buffer from its open file object managed by the editor.
]],
        [[
Fails if this buffer is not linked with a file already.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object to be unlinked from its current file. If no buffer ID is set on this object, attempts to unlink the active buffer from its file instead.
]]
    )

    P.write_to_file = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.buffer_write_to_file(self:id()))
        end,
        "write_to_file",
        [[
Overwrites the on-disk content of this Buffer's linked file with the current content of this Buffer.
]],
        [[
Fails if this buffer is not linked with a file already.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose content is to be written out to its linked file. If no buffer ID is set on this object, attempts to write the active buffer's content to its linked file instead.
]]
    )

    P.type = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.buffer_type(self:id()))
        end,
        "type",
        [[
Gets the buffer type of this Buffer object.
]],
        nil,
        [[
table - buffer_type RedEnum:
    - `buffer.naive`
    - `buffer.gap`
]],
        [[
self: Buffer - Buffer object whose type is returned. If no buffer ID is set on this object, returns the type of the active buffer instead.
]]
    )

    P.set_type = red.doc.build_fn(
        function(self, type)
            return coroutine.yield(red.call.buffer_set_type(self:id(), type))
        end,
        "set_type:",
        [[
Sets the buffer type of this Buffer object.
]],
        [[
If type provided is different than the current buffer type, current buffer content is copied into the data structure for the new buffer type. All other buffer information such as ID is preserved. Copying may take O(n) or worse.
]],
        [[
nil
]],
        [[
self: Buffer - Buffer object whose type is set. If no buffer ID is set on this object, sets the type of the active buffer instead.
]],
        [[
type: EditorBufferType table (RedEnum) - Type of buffer to change this buffer to.
    See:
        - `buffer.naive`
        - `buffer.gap`
]]
    )

    P.naive = {
        type = "EditorBufferType",
        variant = "naive"
    }

    P.gap = {
        type = "EditorBufferType",
        variant = "gap"
    }

    red.doc.document_table(
        P,
        "Buffer",
        [[
Class table for buffer related interaction with the BadRed editor.
]],
        [[
The class Buffer table itself represents static access to the active buffer in the editor. Calling all functions on the buffer class table will lookup the currently active buffer before executing the function on that buffer object. Buffer table functions which return new Buffer objects represent specific buffers in the editor and inherit from the static Buffer class table's functions for calling specific functions on a specific buffer.

An editor buffer being closed will invalidate any matching buffer objects held within the Lua runtime and cause errors when trying to call functions on invalidated buffer tables. Handling cleanup on buffer close will be added in the future as a callback to register on a buffer object.
]],
        {},
        function(_, value_doc)
            return "== Class: Buffer ==\n" .. value_doc
        end
    )

    _G[modname] = P
    return P
end
