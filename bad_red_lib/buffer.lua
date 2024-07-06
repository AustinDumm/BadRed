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

function P:cursor_right(count)
    local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_char(self:id(), count))
    coroutine.yield(red.call.buffer_set_cursor(self:id(), new_cursor))
end

function P:cursor_left(count)
    local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_char(self:id(), -count))
    coroutine.yield(red.call.buffer_set_cursor(self:id(), new_cursor))
end

function P:cursor_up(count)
    local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_line(self:id(), count, true))
    coroutine.yield(red.call.buffer_set_cursor(self:id(), new_cursor))
end

function P:cursor_down(count)
    local new_cursor = coroutine.yield(red.call.buffer_cursor_moved_by_line(self:id(), count, false))
    coroutine.yield(red.call.buffer_set_cursor(self:id(), new_cursor))
end

function P:cursor_index()
    return coroutine.yield(red.call.buffer_cursor(self:id()))
end

function P:set_cursor_index(index)
    coroutine.yield(red.call.buffer_set_cursor(self:id(), index))
end

function P:length()
    return coroutine.yield(red.call.buffer_length(self:id()))
end

function P:content()
    return coroutine.yield(red.call.buffer_content(self:id()))
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

return Buffer
