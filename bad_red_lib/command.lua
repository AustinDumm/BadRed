-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
-- 
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

local P = {}
Command = P

local function command_keymap(origin_pane, root_pane, old_map, command_buffer)
    local new_map = red.keymap:new_map()
    new_map.__index = function(_, _)
        return function(key)
            coroutine.yield(red.buffer:current():insert_at_cursor(key))
        end
    end

    local function is_empty()
        local command = command_buffer:content()
        return string.len(command) == 0
    end
    local function exit_command()
        command_buffer:clear()
        root_pane:close_child(false)
        command_buffer:close()
        origin_pane:set_active()
        red.keymap.current = old_map
    end

    new_map["C+Delete"] = function()
        if is_empty() then
            exit_command()
        else
            red.buffer:clear()
        end
    end
    new_map["Delete"] = function()
        if is_empty() then
            exit_command()
        else
            _ = red.buffer:delete(1)
        end
    end
    new_map["Backspace"] = function()
        if is_empty() then
            exit_command()
        else
            command_buffer:cursor_left(1)
            _ = command_buffer:delete(1)
        end
    end
    new_map["Enter"] = function(_)
        local command = command_buffer:content()
        exit_command()
        coroutine.yield(red.call.run_script(command))
    end

    setmetatable(new_map, new_map)

    return new_map
end

function P.start_command()
    local pane = red.pane:current()

    local old_root_pane = red.pane:root()
    old_root_pane:h_split()

    local root_pane = red.pane:root()
    root_pane:fix_size(1, false)

    local command_pane = root_pane:child(false)
    command_pane:set_active()

    local command_buffer = red.buffer:open()
    command_pane:set_buffer(command_buffer)

    local old_map = red.keymap.current
    red.keymap.current = command_keymap(pane, root_pane, old_map, command_buffer)
end
return Command
