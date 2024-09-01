-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

package.preload["command"] = function(modname, _)
    local doc = require("doc")

    local P = {
        shortcuts = {}
    }
    
    local function trim_trailing(s)
        return s:gsub("%s+$", "")
    end

    P.set_shortcut = doc.build_fn(
        function(self, shortcut, call)
            self.shortcuts[trim_trailing(shortcut)] = call
        end,
        "set_shortcut",
        [[
Creates a new command shortcut entry.
]],
        [[
Command shortcuts are strings set to be equivalent to running a provided Lua function. If a shortcut is provided to the command buffer, the associated callback is called iwth no arguments. This happens instead of the buffer contents being ran as a Lua script.
]],
        [[
nil
]],
        [[
self: Command package table - The command system to set the shortcut on.
]],
        [[
shortcut: String - The string that must be found in the command entry to run this shortcut. Trailing whitespace will be trimmed from `shortcut` and from entries to the command.
]],
        [[
call: Function() - The function to run if `shortcut` is triggered.
]]
    )

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

            if string.sub(command, 1, trigger_key:len()) == trigger_key then
                command = string.sub(command, trigger_key:len() + 1)
            end

            local shortcut = P.shortcuts[trim_trailing(command)]
            if shortcut then
                exit_command()
                shortcut()
            else
                exit_command()
                coroutine.yield(red.call.run_script(command))
            end
        end

        return new_map
    end

    P.command_handler = red.doc.build_fn(
        function(command_entry_map)
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
        end,
        "command_handler",
        [[
Builds a table for handling command input via a command buffer/pane.
]],
        [[
On start, the command handler opens a buffer and creates a 1-high pane at the bottom of the screen for input of the scrips. Keymap to use after command exit is set via set_exit function. This is provided after initialization as often the map to exit into is also the map that has an entry point into the command handler's start meaning that there is a circular reference with one link needed to be set later.
]],
        [[
CommandHandler table
]],
        [[
command_entry_map: Keymap - The Keymap to use while the user enters text into the command buffer. Overrides Esc to cancel command entry, cleanup the command buffer and pane, and return to the exit_map. Overrides Enter to submit and run the command in the buffer then cleanup the command buffer and pane.
]]
    )

    red.doc.document_table(
        P,
        "Command",
        [[
Package that contains the command handler functions.
]],
        nil,
        {},
        function(_, value_doc)
            return "== Class: Command ==\n" .. value_doc
        end
    )

    _G[modname] = P
    return P
end
