-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

package.preload["keymap"] = function(modname, _)
    local P = {}

    local MetaKeymap = {
        __index = function(keymap, event)
            return keymap.parent[event]
        end
    }

    local motion = require("motion")
    local opts = require("opts")

    P.new_map = red.doc.build_fn(
        function(self)
            local instance = { parent = self }
            instance.new_map = P.new_map
            setmetatable(instance, MetaKeymap)
            return instance
        end,
        "new_map",
        [[
Create a keymap child of this keymap.
]],
        [[
Child keymaps pass unhandled key events up to their parent keymap. If no parent handles the keymap, will err due to no handler found. Override a map's metatable's __index function to handle default functionality for unhandled key events.
]],
        [[
Keymap table
]],
        [[
self: Keymap table - The table to use as a parent for this new table.
]]
    )

    P.set_map = red.doc.build_fn(
        function(new_map)
            P.current = new_map
        end,
        "set_map",
        [[
Set the global keymap to a new provided map object.
]],
        nil,
        [[
nil
]],
        [[
new_map: Keymap table - The map to set as the global used keymap on key events.
]]
    )

    P.pop_map = red.doc.build_fn(
        function()
            P.current = P.current.parent or P.current
        end,
        "pop_map",
        [[
Change the global keymap to the parent of the current global keymap.
]],
        nil,
        [[
nil
]]
    )

    P.push_map = red.doc.build_fn(
        function(update_function)
            local map = P.current:new_map()
            update_function(map)
            P.current = map
        end,
        "push_map",
        [[
Call provided update_function with current global map and set the current map to the update_function return value.
]],
        nil,
        [[
nil
]],
        [[
update_function: (Keymap table) -> Keymap table - Callback build the new map as a child of the provided map.
]]
    )

    P.event = red.doc.build_fn(
        function(key_event)
            local map = P.sequence or P.current
            local event_handler = map[key_event]

            if type(event_handler) == "function" then
                event_handler(key_event)
                P.sequence = nil
            elseif type(event_handler) == "table" then
                P.sequence = event_handler
            elseif event_handler == nil then
                P.sequence = nil
            else
                error("Can only treat function and table as key event handlers. Found: " .. tostring(event_handler))
            end
        end,
        "event",
        [[
Handle provided key_event from the current set keymap.
]],
        [[
If the current keymap contains a function at the key event, calls the function and clears out the sequence of input currently stored. If the current keymap contains a table at the key event, uses the new table as the current sequence handler to process the next input with.
]],
        [[
nil
]],
        [[
key_event: string - The KeyEvent string from the BadRed editor hook being handled.
]]
    )

    function P.empty_map()
        local map = P:new_map()
        map.__index = function(_, _)
            return function(key)
                red.buffer:insert(key)
            end
        end
        map.new = P.new_map
        setmetatable(map, map)
        return map
    end

    P.raw_input_map = red.doc.document_table((function()
            local map = P:new_map()
            map.__index = function(_, _)
                return function(key)
                    red.buffer:insert(key)
                end
            end
            map.new = P.new_map
            setmetatable(map, map)

            map["Backspace"] = function(_)
                if red.buffer:cursor() == 0 then
                    return
                end

                red.buffer:set_cursor(
                    motion.char_move(
                        red.buffer:current(),
                        red.buffer:cursor(),
                        -1
                    )
                )
                _ = red.buffer:delete(1)
            end
            map["Delete"] = function(_)
                _ = red.buffer:delete(1)
            end

            map["Enter"] = function(_)
                red.buffer:insert("\n")
            end

            map["Tab"] = function(_)
                if opts.expand_tabs then
                    local text_insert = ""
                    for _ = 1, opts.tab_width do
                        text_insert = text_insert .. " "
                    end
                    red.buffer:insert(text_insert)
                else
                    red.buffer:insert("\t")
                end
            end

            map["Left"] = function(_)
                local index = motion.char_move(red.buffer:current(), red.buffer:cursor(), -1, true)
                red.buffer:set_cursor(index)
            end
            map["Right"] = function(_)
                local index = motion.char_move(red.buffer:current(), red.buffer:cursor(), 1, true)
                red.buffer:set_cursor(index)
            end

            return map
        end)(),
        "raw_input_map",
        [[
Keymap for standard raw input.
]],
        [[
Supports Backspace, Delete, Left, Right, and Enter as expected deletion/movement/newline. All other characters inputs the keycode itself into the current buffer.
]],
        {},
        function(_, i) return i end
    )

    P.current = P.raw_input_map
    P.sequence = nil

    red.doc.document_table(
        P,
        "Keymap",
        [[
Package for creating keymaps and managing the globally current keymap.
]],
        nil,
        {},
        function(_, val_doc)
            return "== Package: Keymap ==\n" .. val_doc
        end
    )

    _G[modname] = P
    return P
end
