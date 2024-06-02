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

return Buffer
