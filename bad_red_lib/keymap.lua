-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
-- 
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

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
    map["Backspace"] = function()
        red.buffer:current():cursor_left(1)
        _ = red.buffer:current():delete(1)
    end
    map["C+Delete"] = function()
        red.buffer:current():clear()
    end
    map["Delete"] = function()
        _ = red.buffer:current():delete(1)
    end
    map["Enter"] = function()
        red.buffer:current():insert_at_cursor("\n")
    end
    map["Left"] = function()
        red.buffer:current():cursor_left(1)
    end
    map["Right"] = function()
        red.buffer:current():cursor_right(1)
    end
    map["C+e"] = function()
        local content = red.buffer:current():content()
        red.buffer:current():execute()
    end
    map.parent = nil
    setmetatable(map, map)
    return map
end

P.current = root_map()
return Keymap
