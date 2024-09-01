package.preload["registers"] = function(modname, _)
    local P = {
        register_key = '"',
        content_map = {}
    }

    local motion = require("motion")
    local buffer = require("buffer")
    local keymap = require("keymap")

    local function delete_mode_map(register, map)
        local delete_map = motion.motion_keymap(
            map,
            function() return buffer:current() end,
            1,
            function(start, stop, is_inclusive)
                if stop < start then
                    start, stop = stop, start
                end

                buffer:set_cursor(start)
                local delete_count = stop - start

                if is_inclusive then
                    delete_count = delete_count + 1
                end
                local content = buffer:delete(delete_count)
                P.set_register(register, content)
            end
        )

        delete_map["!"] = function(_) red.buffer:clear() end

        return delete_map
    end

    local function action_map(key)
        if key == nil then
            key = P.register_key
        end

        local map = keymap.empty_map()
        local map_mt = {
            __index = function(_, _)
                return nil
            end
        }
        setmetatable(map, map_mt)

        map["d"] = delete_mode_map(key, map)

        map["p"] = function(_)
            P.append(red.buffer:current(), key)
        end

        map["P"] = function(_)
            P.insert(red.buffer:current(), key)
        end

        return map
    end

    function P.append(buffer, register)
        if register == nil then
            register = P.register_key
        end

        local stored = P.content_map[register]
        if stored == nil then
            return
        end

        buffer:set_cursor(motion.char_move(buffer:current(), buffer:cursor(), 1, false))
        buffer:insert(stored)
        buffer:set_cursor(motion.char_move(buffer:current(), buffer:cursor(), -1, false))
    end

    function P.insert(buffer, register)
        if register == nil then
            register = P.register_key
        end

        local stored = P.content_map[register]
        if stored == nil then
            return
        end

        buffer:insert(stored)
        buffer:set_cursor(motion.char_move(buffer:current(), buffer:cursor(), -1, false))
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

    function P.set_register(register, content)
        if register == nil then
            register = P.register_key
        end

        P.content_map[register] = content
    end

    _G[modname] = P
    return P
end
