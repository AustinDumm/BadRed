package.preload["motion"] = function(modname, _)
    local doc = require("doc")
    local P = {}

    local function signum(value)
        if value < 0 then
            return -1
        elseif value > 0 then
            return 1
        else
            return 0
        end
    end

    local function handle_newline(buffer, index, count)
        local index_char = buffer:content_at(index, 1)
        local line_content = buffer:line_content(buffer:line_for_index(index))

        if index_char == "\n" and line_content ~= "\n" then
            local shift = signum(count)

            return coroutine.yield(red.call.buffer_index_moved_by_char(buffer:id(), index, shift))
        end

        return nil
    end

    local function is_whitespace(char)
        return char == nil or string.match(char, "%s") ~= nil
    end

    local function is_non_alphanumeric(char)
        return char == nil or string.match(char, "%W") ~= nil
    end

    local function is_alphanumeric(char)
        return char == nil or string.match(char, "%w") ~= nil
    end

    local function get_word_split(only_whitespace, is_alphanumeric_word)
        if only_whitespace then
            return is_whitespace
        else
            if is_alphanumeric_word then
                return function(char)
                    return is_non_alphanumeric(char) or is_whitespace(char)
                end
            else
                return function(char)
                    return is_alphanumeric(char) or is_whitespace(char)
                end
            end
        end
    end

    local function move_index_to_word(self, current_index, move_left)
        local shift_char
        if move_left then
            shift_char = -1
        else
            shift_char = 1
        end

        while true do
            local content = self:content_at(current_index, 1)
            if content ~= nil and string.match(content, "%s") == nil then
                return current_index
            end

            local new_current_index = P.char_move(self, current_index, shift_char)
            if new_current_index == current_index then
                -- Reached the edges of the buffer
                return new_current_index
            else
                current_index = new_current_index
            end
        end
    end

    P.char_move = doc.build_fn(
        function(buffer, start_index, count, skip_newlines)
            if count == 0 then
                return start_index
            end

            local new_index = coroutine.yield(red.call.buffer_index_moved_by_char(buffer:id(), start_index, count))

            if skip_newlines then
                return handle_newline(buffer, new_index, count) or new_index
            else
                return new_index
            end
        end,
        "char_move",
        [[
Returns the byte index `count` number of characters to the right of the current cursor for this buffer.
]],
        [[
Negative `count` given moves the index to the left. Provides options for dealing with newlines.
]],
        [[
non-negative integer - The byte index moved by `count` chars.
]],
        [[
buffer: Buffer - Buffer object whose moved cursor index should be returned. If no buffer ID is set on this object, returns the byte index of cursor of the active buffer moved by `count` characters.
]],
        [[
count: integer - Number of characters to move. Note that it is possible the returned cursor byte index will increase by more bytes than the count provided if moving over utf8 characters that are encoded in more than 1 byte. For this reason, use cursor movement functions instead of manually adding to a cursor byte value. Will stop at the end of the buffer if count moves the cursor more than the remaining number of characters in the buffer.
]],
        [[
skip_newlines: bool = false - Should the cursor be allowed to stop over a newline character. If true, will move to the next character to the right if a newline was landed on.
]]
    )

    local function to_word_boundary_helper(buffer, current_index, count, only_whitespace)
        if count == 0 then
            return current_index
        end

        local shift = signum(count)

        current_index = move_index_to_word(buffer, current_index, shift < 0)

        local is_alphanumeric_word = is_alphanumeric(buffer:content_at(current_index, 1))
        local is_split = get_word_split(only_whitespace, is_alphanumeric_word)


        local character_index = P.char_move(buffer, current_index, shift)
        while not is_split(buffer:content_at(character_index, 1)) do
            current_index = character_index
            character_index = P.char_move(buffer, current_index, shift)

            if current_index == character_index then
                -- Reached an edge of the buffer
                break
            end
        end

        return to_word_boundary_helper(
            buffer,
            current_index,
            count - shift,
            only_whitespace
        )
    end

    P.to_word_boundary = doc.build_fn(
        function(buffer, start_index, count, only_whitespace)
            return to_word_boundary_helper(
                buffer,
                P.char_move(buffer, start_index, signum(count)),
                count,
                only_whitespace
            )
        end,
        "to_word_boundary",
        [[
Returns the byte index for the starting letter of the word `count` words away from the provided index.
]],
        [[
If `count` is 0, will return `start_index`. If `count` is negative, will return `count`th word's starting letter moving to the left from `start_index`. If `count` is positive, will return the `count`th word's starting letter moving to the right from `start_index`. If `start_index` is on a word's starting letter, this word is not counted as part of `count`.
]],
        [[
non-negative integer - The byte index of the first character of the word `count` away from `start_index`.
]],
        [[
buffer: Buffer table - The buffer object to do the word index search on.
]],
        [[
start_index: non-negative integer - The byte index within the buffer table to start the search. Must be on a character byte index.
]],
        [[
count: integer - Number of boundaries to move past before stopping.
]],
        [[
only_whitespace: bool = false - If false, any character of the opposite type of the character on `start_index` is considered to split words. If true, only whitespace characters are considered to split words.
]]
    )

    P.past_word_boundary = doc.build_fn(
        function(buffer, start_index, count, only_whitespace)
            local boundary = to_word_boundary_helper(buffer, start_index, count, only_whitespace)

            local shift = signum(count)
            local length = buffer:length()
            local new_index = boundary
            repeat
                new_index = P.char_move(buffer, new_index, shift)
            until new_index >= length or not is_whitespace(buffer:content_at(new_index, 1))

            return new_index
        end,
        "past_word_boundary",
        [[
Returns the byte index for the first letter following the word boundary `count` boundaries away from the provided index.
]],
        [[
If `count` is 0, will return `start_index`. If `count` is negative, will return the first non-whitespace character preceeding the `count`th word boundary moving to the left from `start_index`. If `count` is positive, will return the first non-whitespace character succeeding the `count`th word boundary moving ot the right from `start_index`.
]],
        [[
non-negative integer - The byte index of the first non-whitespace character after the `count`th word boundary from `start_index`.
]],
        [[
buffer: Buffer table - The buffer object to do the word index search on.
]],
        [[
start_index: non-negative integer - The byte index within the buffer table to start the search. Must be on a character byte index.
]],
        [[
count: integer - Number of boundaries to move past before stopping.
]],
        [[
only_whitespace: bool = false - If false, any character of the opposite type of the character on `start_index` is considered to split words. If true, only whitespace characters are considered to split words.
]]
    )

    P.to_line_boundary = doc.build_fn(
        function(buffer, start_index, count, include_whitespace)
            local shift = signum(count)
            local current_line = buffer:line_for_index(start_index)

            local new_byte_index
            if shift < 0 then
                new_byte_index = buffer:line_start_index(current_line)
            else
                new_byte_index = buffer:line_end_index(current_line)
            end

            local result
            if include_whitespace then
                result = new_byte_index
            else
                result = move_index_to_word(buffer, new_byte_index, shift > 0)
            end

            if buffer:content_at(result, 1) == "\n" or result == buffer:length() then
                result = result - 1
            end
            local next_count = count - shift

            if next_count == 0 then
                return result
            else
                return P.to_line_boundary(
                    buffer,
                    move_index_to_word(buffer, result, shift < 0),
                    count - shift,
                    include_whitespace
                )
            end
        end,
        "to_line_boundary",
        [[
Return byte index for line_boundary `count` line boundaries away.
]],
        [[
If already on line boundary, does not include that boundary in the count. Negative `count` looks for boundaries to the left. Positive `count` moves for boundaries to the right.
]],
        [[
non-negative integer - The byte index on the line boundary `count` boundaries away.
]],
        [[
buffer: Buffer table - Buffer to calculate motion on.
]],
        [[
start_index: non-negative integer - Byte index to start searching from.
]],
        [[
count: integer - Number of boundaries to move. Negative indicates movement to the left, positive indicates movement to the right.
]],
        [[
include_whitespace - If false, does not consider whitespace valid line boundary, treats only non-whitespace characters as boundaries.
]]
    )

    P.to_character_find = doc.build_fn(
        function(buffer, start, search_char, count, search_right)
            local shift
            if search_right then
                shift = 1
            else
                shift = -1
            end

            local index = P.char_move(buffer, start, shift, false)
            while 0 <= index and index < buffer:length() do
                local search_length = string.len(search_char)

                local content = buffer:content_at(index, search_length)
                if content == search_char then
                    count = count - 1
                end
                if content == "\n" then
                    return nil
                end
                if count == 0 then
                    break
                end

                index = P.char_move(buffer, index, shift, false)
            end

            return index
        end,
        "to_character_find",
        [[
Returns the character index for the `count`'th occurrence of `search_char` on line containing `start`.
]],
        nil,
        [[
optional non-negative integer - The character byte index for the `count`'th occurrence of `search_char`. Nil if none found]],
        [[
buffer: Buffer table - The buffer to search for character in.
]],
        [[
start: non-negative integer - The byte char index to begin the search from.
]],
        [[
search_char: character - The character to match on.
]],
        [[
count: positive integer - The number of occurrences of `search_char` to skip before returning.
]],
        [[
search_right: boolean - True if search should move to the right, towards the end of the buffer. False if search should move left towards the start of the buffer.
]]
    )

    P.motion_keymap = doc.build_fn(
        function(parent_map, buffer_get, count, callback)
            local map = parent_map:new_map()
            local function run_motion(motion, is_inclusive)
                if motion == nil then
                    return
                end

                local buffer = buffer_get()
                local start = buffer:cursor()
                local stop = motion(buffer, start)

                if stop == nil then
                    return
                else
                    callback(start, stop, is_inclusive)
                end
            end

            map["h"] = function(_)
                run_motion(function(buffer, start)
                    return P.char_move(buffer, start, -count, true)
                end)
            end
            map["Left"] = map["h"]
            map["l"] = function(_)
                run_motion(function(buffer, start)
                    return P.char_move(buffer, start, count, true)
                end)
            end
            map["Right"] = map["l"]

            map["w"] = function(_)
                run_motion(function(buffer, start)
                    return P.past_word_boundary(buffer, start, count, false)
                end)
            end
            map["W"] = function(_)
                run_motion(function(buffer, start)
                    return P.past_word_boundary(buffer, start, count, true)
                end)
            end

            map["b"] = function(_)
                run_motion(function(buffer, start)
                    return P.to_word_boundary(buffer, start, -count, false)
                end)
            end
            map["B"] = function(_)
                run_motion(function(buffer, start)
                    return P.to_word_boundary(buffer, start, -count, true)
                end)
            end

            map["e"] = function(_)
                run_motion(
                    function(buffer, start)
                        return P.to_word_boundary(buffer, start, count, false)
                    end,
                    true
                )
            end
            map["E"] = function(_)
                run_motion(
                    function(buffer, start)
                        return P.to_word_boundary(buffer, start, count, true)
                    end,
                    true
                )
            end

            map["0"] = function(_)
                run_motion(
                    function(buffer, start)
                        return P.to_line_boundary(buffer, start, -1, true)
                    end,
                    false
                )
            end

            map["^"] = function(_)
                run_motion(
                    function(buffer, start)
                        return P.to_line_boundary(buffer, start, -1, false)
                    end,
                    false
                )
            end

            map["_"] = map["^"]

            map["$"] = function(_)
                run_motion(
                    function(buffer, start)
                        return P.to_line_boundary(buffer, start, 1, true)
                    end,
                    true
                )
            end

            map["f"] = (function(_)
                local keymap = {}
                local mt = {
                    __index = function(_, _)
                        return function(search_key)
                            run_motion(
                                function(buffer, start)
                                    return P.to_character_find(buffer, start, search_key, 1, true)
                                end,
                                true
                            )
                        end
                    end
                }
                setmetatable(keymap, mt)

                return keymap
            end)()
            map["F"] = (function(_)
                local keymap = {}
                local mt = {
                    __index = function(_, _)
                        return function(search_key)
                            run_motion(
                                function(buffer, start)
                                    return P.to_character_find(buffer, start, search_key, 1, false)
                                end,
                                true
                            )
                        end
                    end
                }
                setmetatable(keymap, mt)

                return keymap
            end)()

            map["t"] = (function(_)
                local keymap = {}
                local mt = {
                    __index = function(_, _)
                        return function(search_key)
                            run_motion(
                                function(buffer, start)
                                    local found = P.to_character_find(buffer, start, search_key, 1, true)
                                    if found == nil then
                                        return nil
                                    end
                                    return P.char_move(buffer, found, -1, false)
                                end,
                                true
                            )
                        end
                    end
                }
                setmetatable(keymap, mt)

                return keymap
            end)()
            map["T"] = (function(_)
                local keymap = {}
                local mt = {
                    __index = function(_, _)
                        return function(search_key)
                            run_motion(
                                function(buffer, start)
                                    local found = P.to_character_find(buffer, start, search_key, 1, false)
                                    if found == nil then
                                        return nil
                                    end
                                    return P.char_move(buffer, found, 1, false)
                                end,
                                true
                            )
                        end
                    end
                }
                setmetatable(keymap, mt)

                return keymap
            end)()

            map["Esc"] = function(_)
                run_motion(
                    function(_, _)
                        return nil
                    end
                )
            end


            return map
        end,
        "motion_keymap",
        [[
Returns a keymap object for standard motion keys to run a given action.
]],
        [[
On each motion key, a callback function is given the start and end byte index for the motion.
]],
        [[
Keymap table
]],
        [[
parent_map: Keymap table - The keymap this new motion map should inherit from
]],
        [[
buffer_get: Function() -> Buffer Table - Returns the buffer the motion should be played against
]],
        [[
count: positive integer - The number of times to run the motion.
]],
        [[
callback: Function(start: non-negative integer, stop: non-negative integer, is_inclusive: bool) - run when a motion is entered. is_inclusive is true only if the motion provided includes the character at index `stop`.
]]
    )


    return doc.document_table(
        P,
        modname,
        [[
Contains functions for retrieving and calculating motion distances on a buffer.
]],
        [[
Meant to emulate vim-style motions where possible. Contains a "motion" keymap table that can be used as a child table which triggers a callback with the motion's start and end byte index for the given buffer.
]],
        {},
        function(value_doc) return "== Motion ==\n" .. value_doc end
    )
end
