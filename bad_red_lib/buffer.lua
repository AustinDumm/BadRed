-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
-- 
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

local P = {}
Buffer = {}
Buffer = P

function P:new(id)
    local instance = {id = id}
    setmetatable(instance, self)
    self.__index = self
    return instance
end

function P:current()
    id = coroutine.yield(red.call.current_buffer_id())
    return self:new(id)
end

function P:insert_at_cursor(content)
    coroutine.yield(red.call.buffer_insert(self.id, content))
end

function P:delete(count)
    coroutine.yield(red.call.buffer_delete(self.id, count))
end

function P:cursor_right(count)
    coroutine.yield(red.call.buffer_cursor_move_char(self.id, count, false))
end

function P:cursor_left(count)
    coroutine.yield(red.call.buffer_cursor_move_char(self.id, count, true))
end

function P:cursor_index()
    coroutine.yield(red.call.buffer_cursor_index(self.id))
end

function P:set_cursor_index(index)
    coroutine.yield(red.call.buffer_set_cursor_index(self.id, index))
end

function P:length()
    return coroutine.yield(red.call.buffer_length(self.id))
end

function P:content()
    return coroutine.yield(red.call.buffer_content(self.id))
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

return Buffer
