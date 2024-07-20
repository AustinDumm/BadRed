-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

local P = {}
Command = P

local function command_keymap(base_map, origin_pane, root_pane, old_map, command_buffer, trigger_key)
    local new_map = base_map:new_map()

    local function exit_command()
        command_buffer:clear()
        root_pane:close_child(false)
        command_buffer:close()
        origin_pane:set_active()
        red.keymap.current = old_map
    end

    new_map["Esc"] = function(_)
        exit_command()
    end
    new_map["Enter"] = function(_)
        local command = command_buffer:content()
        command = string.sub(command, trigger_key:len() + 1)

        exit_command()
        coroutine.yield(red.call.run_script(command))
    end

    return new_map
end

function P.command_handler(command_entry_map)
    local handler = {}
    function handler:start_command(trigger_key)
        local pane = red.pane:current()

        local old_root_pane = red.pane:root()
        old_root_pane:h_split()

        local root_pane = red.pane:root()
        root_pane:fix_size(1, false)

        local command_pane = root_pane:child(false)
        command_pane:set_active()

        local command_buffer = red.buffer:open()
        command_buffer:insert(trigger_key)
        command_pane:set_buffer(command_buffer)

        red.keymap.current = command_keymap(
            command_entry_map,
            pane,
            root_pane,
            self.exit_map,
            command_buffer,
            trigger_key
        )
    end

    function handler:set_exit(exit_map)
        self.exit_map = exit_map
    end

    return handler
end

return Command
