-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
-- 
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

local P = {}

Keymap = P

local MetaKeymap = {
    __index = function(keymap, event)
        return keymap.parent[event]
    end
}

function P:new_map()
    local instance = { parent = self }
    instance.new_map = P.new_map
    setmetatable(instance, MetaKeymap)
    return instance
end

function P.set_map(new_map)
    P.current = new_map
end

function P.pop_map()
    P.current = P.current.parent or P.current
end

function P.push_map(update_function)
    local map = P.current:new_map()
    update_function(map)
    P.current = map
end

function P.event(key_event)
    local map = P.sequence or P.current
    local event_handler = map[key_event]

    if type(event_handler) == "function" then
        event_handler(key_event)
        P.sequence = nil
    elseif type(event_handler) == "table" then
        P.sequence = event_handler
    elseif event_handler == nil then
        return
    else
        error("Can only treat function and table as key event handlers. Found: " .. tostring(event_handler))
    end
end

P.raw_input_map = (function()
    local map = P:new_map()
    map.__index = function(_, _)
        return function(key)
            red.buffer:insert_at_cursor(key)
        end
    end
    map.new = P.new_map
    setmetatable(map, map)

    map["Backspace"] = function(_)
        if red.buffer:cursor_index() == 0 then
            return
        end

        red.buffer:cursor_left(1)
        _ = red.buffer:delete(1)
    end
    map["Delete"] = function(_)
        _ = red.buffer:delete(1)
    end

    map["Left"] = function(_)
        red.buffer:cursor_left(1)
    end
    map["Right"] = function(_)
        red.buffer:cursor_right(1)
    end
    map["Enter"] = function(_)
        red.buffer:insert_at_cursor("\n")
    end
    map["C+e"] = function()
        red.buffer:current():execute()
    end

    return map
end)()

P.current = P.raw_input_map
P.sequence = nil
return Keymap
