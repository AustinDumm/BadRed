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
    setmetatable(instance, MetaKeymap)
    return instance
end

function P.set_map(new_map)
    P.current = new_map
end

function P.pop_map()
    P.current = P.current.parent or P.current
end

function P.event(key_event)
    local map = P.sequence or P.current
    local event_handler = map[key_event]

    if type(event_handler) == "function" then
        event_handler(key_event)
        P.sequence = nil
    elseif type(event_handler) == "table" then
        P.sequence = event_handler
    else
        error("Can only treat function and table as key event handlers")
    end
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
    map["C+r"] = (function()
        local nested = map:new_map()
        nested["_loop_count"] = 0
        setmetatable(nested, nested)

        nested.__index = function(_, key_event)
            local number = tonumber(key_event)
            if number == nil then
                return function(k_e)
                    for _=1,nested._loop_count do
                        nested.parent[k_e](k_e)
                    end
                end
            else
                nested._loop_count = nested._loop_count * 10 + number
                return nested
            end
        end

        return nested
    end)()
    map["C+w"] = (function()
        local pane_map = map:new_map()
        pane_map["v"] = function(_) red.pane:current():v_split() end
        pane_map["h"] = function(_) red.pane:current():h_split() end
        pane_map["u"] = function(_) red.pane:current():parent():set_active() end
        pane_map["l"] = function(_) red.pane:current():child(true):set_active() end
        pane_map["r"] = function(_) red.pane:current():child(false):set_active() end
        pane_map["s"] = function(_) red.pane:current():sibling():set_active() end
        pane_map["+"] = function(_) red.pane:current():increase_size() end
        pane_map["-"] = function(_) red.pane:current():decrease_size() end
        return pane_map
    end)()
    map["C+e"] = function()
        local content = red.buffer:current():content()
        red.buffer:current():execute()
    end
    map.parent = nil
    setmetatable(map, map)
    return map
end

P.current = root_map()
P.sequence = nil
return Keymap
