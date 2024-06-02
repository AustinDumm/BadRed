local P = {}

Keymap = P

function P:new_map()
    local instance = { parent = self }
    setmetatable(instance, self)
    self.__index = self
    return instance
end

function P.set_map(new_map)
    P.current = new_map
end

function P.pop_map()
    P.current = P.current.parent or P.current
end

function P.event(key_event)
    P.current[key_event](key_event)
end

local function root_map()
    local map = P:new_map()
    map.__index = function(_, _)
        return function(key)
            coroutine.yield(red.buffer:current():insert_at_cursor(key))
        end
    end
    map["Backspace"] = function() end
    map["Delete"] = function() end
    map.parent = nil
    setmetatable(map, map)
    return map
end

P.current = root_map()
return Keymap
