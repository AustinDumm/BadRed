local P = {}

local function normal_mode(command_handler, input_map)
    local map = red.keymap:new_map()
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
            red.buffer:cursor_right(1)
        end
        red.keymap.current = input_map
    end
    map["h"] = function(_) red.buffer:cursor_left(1, true) end
    map["l"] = function(_) red.buffer:cursor_right(1, true) end
    map["k"] = function(_) red.buffer:cursor_up(1, true) end
    map["j"] = function(_) red.buffer:cursor_down(1, true) end
    map["f"] = function(_) error("TEST ERROR") end
    map["d"] = (function()
        local delete_map = red.keymap:new_map()

        delete_map["!"] = function(_) red.buffer:clear() end

        return delete_map
    end)()

    map["_exit_char"] = nil
    function map:did_become_active()
        local line_content = red.buffer:cursor_line_content()
        if (map._exit_char == "a" or map._exit_char == "i") and line_content ~= "\n" then
            red.buffer:cursor_left(1)
        end

        map._exit_char = nil
    end

    map.parent = nil
    setmetatable(map, map)
    return map
end

-- Initializes a the modal input style into the provided keymap. Builds onto
-- keymap's current keymap to be used for default text input.
function P:InitMode(keymap)
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
end

return P
