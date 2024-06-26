
local P = {}

function P:open(path)
    local id = coroutine.yield(red.call.file_open(path))
    return self:from_id(id)
end

function P:close()
    coroutine.yield(red.call.file_close(self:id()))
end

function P:from_buffer(buffer)
    local file_id = coroutine.yield(red.call.buffer_current_file(buffer:id()))
end

function P:from_id(id)
    local instance = { _id = id }
    setmetatable(instance, self)
    self.__index = self
    return instance
end

function P:id()
    return self._id
end

return P

