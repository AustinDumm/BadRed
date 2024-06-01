local Buffer = {}
function Buffer:new(id)
    local buffer = {id = id}
    setmetatable(buffer, self)
    self.__index = self
    return buffer
end

function Buffer:current()
    id = coroutine.yield(red.call.current_buffer_id())
    return Buffer:new(id)
end

function Buffer:insert_at_cursor(content)
    if self.id == nil then return end
    coroutine.yield(red.call.buffer_insert(self.id, content))
end

red.buffer = Buffer

coroutine.yield(red.call.set_hook("key_event", function(event)
    red.buffer.current():insert_at_cursor(event)
end))

