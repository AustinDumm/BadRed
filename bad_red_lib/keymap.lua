local P = {}

Keymap = P

function P:new()
    local instance = { default = nil, map = {} }
    setmetatable(instance, self)
    self.__index = self
    return instance
end

return Keymap
