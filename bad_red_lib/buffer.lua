-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

local P = {}
Buffer = {}
Buffer = P

function P:new(id)
    local instance = { _id = id }
    setmetatable(instance, self)
    self.__index = self
    return instance
end

function P:id()
    return self._id or P:current()._id
end

function P:open()
    local id = coroutine.yield(red.call.buffer_open())
    return P:new(id)
end

function P:close()
    coroutine.yield(red.call.buffer_close(self:id()))
end

function P:current()
    local id = coroutine.yield(red.call.current_buffer_id())
    return self:new(id)
end

function P:insert_at_cursor(content)
    coroutine.yield(red.call.buffer_insert(self:id(), content))
end

function P:delete(count)
    coroutine.yield(red.call.buffer_delete(self:id(), count))
end

function P:cursor()
    return coroutine.yield(red.call.buffer_cursor(self:id()))
end

function P:cursor_right(count, skip_newlines, keep_col_index)
    local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_char(self:id(), count))
    local cursor_char = self:content_at(new_cursor, 1)
    self:set_cursor_index(new_cursor, keep_col_index)

    if skip_newlines then
        local line_content = self:line_content(self:cursor_line())

        if cursor_char == "\n" and line_content ~= "\n" then
            self:cursor_right(1)
        end
    end
end

function P:cursor_left(count, skip_newlines, keep_col_index)
    local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_char(self:id(), -count))
    local cursor_char = self:content_at(new_cursor, 1)
    self:set_cursor_index(new_cursor, keep_col_index)

    if skip_newlines then
        local line_content = self:line_content(self:cursor_line())

        if cursor_char == "\n" and line_content ~= "\n" then
            self:cursor_left(1)
        end
    end
end

function P:cursor_up(count, skip_newlines)
    local current_line = self:cursor_line()
    local to_line = current_line - count

    if to_line < 0 then
        self:set_cursor_index(0)
    else
        self:set_cursor_line(current_line - 1)
    end

    if skip_newlines and self:cursor_content() == "\n" and self:cursor_line_content() ~= "\n" then
        self:cursor_left(1, false, true)
    end
end

function P:cursor_down(count, skip_newlines)
    local current_line = self:cursor_line()
    local to_line = current_line + count
    local line_count = self:lines()

    if to_line >= line_count then
        self:set_cursor_index(self:length())
    else
        self:set_cursor_line(current_line + 1)
    end

    if skip_newlines and self:cursor_content() == "\n" and self:cursor_line_content() ~= "\n" then
        self:cursor_left(1, false, true)
    end
end

function P:cursor_index()
    return coroutine.yield(red.call.buffer_cursor(self:id()))
end

function P:cursor_line()
    return coroutine.yield(red.call.buffer_cursor_line(self:id()))
end

function P:set_cursor_index(index, keep_col_index)
    coroutine.yield(red.call.buffer_set_cursor(self:id(), index, keep_col_index))
end

function P:set_cursor_line(line)
    coroutine.yield(red.call.buffer_set_cursor_line(self:id(), line))
end

function P:length()
    return coroutine.yield(red.call.buffer_length(self:id()))
end

function P:lines()
    return coroutine.yield(red.call.buffer_line_count(self:id()))
end

function P:content()
    return coroutine.yield(red.call.buffer_content(self:id()))
end

function P:content_at(byte_index, char_length)
    return coroutine.yield(red.call.buffer_content_at(self:id(), byte_index, char_length))
end

function P:line_content(line_index)
    return coroutine.yield(red.call.buffer_line_content(self:id(), line_index))
end

function P:cursor_line_content()
    return self:line_content(self:cursor_line())
end

function P:cursor_content()
    return self:content_at(self:cursor(), 1)
end

function P:clear()
    local content_length = self:length()
    self:set_cursor_index(0)
    self:delete(content_length)
end

function P:run_as_script()
    local content = self:content()
    coroutine.yield(red.call.run_script(content))
end

function P:execute()
    self:run_as_script()
    self:clear()
end

function P:link_file(file, preserve_buffer)
    coroutine.yield(red.call.buffer_link_file(self:id(), file:id(), not preserve_buffer))
end

function P:unlink_file()
    coroutine.yield(red.call.buffer_unlink_file(self:id()))
end

function P:write_to_file()
    coroutine.yield(red.call.buffer_write_to_file(self:id()))
end

function P:type()
    return coroutine.yield(red.call.buffer_type(self:id()))
end

function P:set_type(type)
    return coroutine.yield(red.call.buffer_set_type(self:id(), type))
end

P.naive = {
    type = "EditorBufferType",
    variant = "naive"
}

P.gap = {
    type = "EditorBufferType",
    variant = "gap"
}

return Buffer
