package.preload["mode"] = function(modname, _)
    local P = {}

    local keymap = require("keymap")
    local buffer = require("buffer")
    local motion = require("motion")

    local function normal_mode(command_handler, input_map)
        local map = keymap:new_map()

        map = motion.motion_keymap(
            map,
            function() return buffer:current() end,
            1,
            function(_, stop)
                buffer:set_cursor(stop)
            end
        )
        
        map["C+w"] = (function()
            local pane_map = map:new_map()
            pane_map["v"] = function(_)
                red.pane:v_split()
                red.pane:child(true):set_active()
            end
            pane_map["s"] = function(_)
                red.pane:h_split()
                red.pane:child(true):set_active()
            end
            pane_map["q"] = function(_)
                red.pane:close()
            end
            return pane_map
        end)()
        map[":"] = function(key) command_handler:start_command(key) end
        map["i"] = function(key)
            map._exit_char = key
            red.keymap.current = input_map
        end
        map["a"] = function(key)
            map._exit_char = key
            if red.buffer:cursor_content() ~= "\n" then
                red.buffer:set_cursor(red.buffer:cursor_right(1))
            end
            red.keymap.current = input_map
        end
        map["k"] = function(_) red.buffer:cursor_up(1, true) end
        map["j"] = function(_) red.buffer:cursor_down(1, true) end

        map["C+e"] = function(_)
            local current_line = red.pane:top_line()
            if current_line + 1 >= 2^16 then
                return
            end

            red.pane:set_top_line(current_line + 1)
        end
        map["C+y"] = function(_)
            local current_line = red.pane:top_line()
            if current_line <= 0 then
                return
            end

            red.pane:set_top_line(current_line - 1)
        end

        map["C+d"] = function(_)
            local height = red.pane:root():frame().rows
            local top_line = red.pane:top_line()

            local new_top_line = top_line + math.floor(height / 2)

            if new_top_line >= 2^16 then
                new_top_line = 2^16 - 1
            end

            red.pane:set_top_line(new_top_line)
        end

        map["C+u"] = function(_)
            local height = red.pane:root():frame().rows
            local top_line = red.pane:top_line()

            local new_top_line = top_line - math.floor(height / 2)

            if new_top_line < 0 then
                new_top_line = 0
            end

            red.pane:set_top_line(new_top_line)
        end

        map["d"] = (function()
            local delete_map = motion.motion_keymap(
                map,
                function() return buffer:current() end,
                1,
                function(start, stop, is_inclusive)
                    buffer:set_cursor(start)

                    local delete_count = stop - start
                    if is_inclusive then
                        delete_count = delete_count + 1
                    end
                    buffer:delete(delete_count)
                end
            )

            delete_map["!"] = function(_) red.buffer:clear() end

            return delete_map
        end)()

        map["_exit_char"] = nil
        function map:did_become_active()
            local line_content = red.buffer:cursor_line_content()
            if (map._exit_char == "a" or map._exit_char == "i") and line_content ~= "\n" then
                red.buffer:set_cursor(red.buffer:cursor_left(1))
            end

            map._exit_char = nil
        end

        map.parent = nil
        setmetatable(map, map)
        return map
    end

    P.init = red.doc.build_fn(
        function(self, keymap)
            local base_map = keymap.current

            self.command_handler = red.command.command_handler(base_map)
            self.input_map = base_map:new()
            self.normal_map = normal_mode(self.command_handler, self.input_map)

            self.command_handler:set_exit(self.normal_map)
            self.input_map["Esc"] = function(_)
                keymap.current = self.normal_map
                self.normal_map:did_become_active()
            end

            red.keymap.current = self.normal_map
        end,
        "init",
        [[
Initializes a the modal input style into the provided keymap.
]],
        [[
Builds onto keymap's current keymap to be used for default text input.
]],
        [[
nil
]],
        [[
self: Mode package table
]],
        [[
keymap: Keymap package table with its current map used as the base map to build modal input off of.
]]
    )

    red.doc.document_table(
        P,
        "mode",
        [[
Package containing functions for setting up modal editing via Keymap
]],
        nil,
        {},
        function(_, val_doc)
            return "== Package: Mode ==\n" .. val_doc
        end
    )

    _G[modname] = P
    return P
end
