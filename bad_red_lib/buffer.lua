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

return Buffer
