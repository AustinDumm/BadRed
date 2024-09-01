package.preload["registers"] = function(modname, _)
    local P = {
        register_key = '"',
        char_map = {}
    }

    local keymap = require("keymap")
    local motion = require("motion")

    local function action_map(key)
        if key == nil then
            key = P.register_key
        end

        local map = {}
        local map_mt = {
            __index = function(_, _)
                return nil
            end
        }
        setmetatable(map, map_mt)

        map["p"] = function(_)
            local stored = P.char_map[key]
            if stored == nil then
                return
            end

            red.buffer:set_cursor(motion.char_move(red.buffer:current(), red.buffer:cursor(), 1, false))
            red.buffer:insert(stored)
            red.buffer:set_cursor(motion.char_move(red.buffer:current(), red.buffer:cursor(), -1, false))
        end

        map["P"] = function(_)
            local stored = P.char_map[key]
            if stored == nil then
                return
            end

            red.buffer:insert(stored)
            red.buffer:set_cursor(motion.char_move(red.buffer:current(), red.buffer:cursor(), -1, false))
        end

        return map
    end

    function P.register_map()
        local map = {}
        local map_mt = {
            __index = function(_, k)
                return action_map(k)
            end
        }
        setmetatable(map, map_mt)

        return map
    end

    _G[modname] = P
    return P
end
