-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

package.preload["styling"] = function(modname, _)
    local P = { style_map = {} }

    local function register_lua(styling)
        styling:register(
            "lua",
            {
                { name = "keyword",     regex = "(and|break|do|else|elseif|end|false|for|function|if|in|local|nil|not|or|repeat|return|then|true|until|while)\\s" },
                { name = "comment",     regex = "--.*" },
                { name = "string",      regex = "(\".*\")|\\[\\[|\\]\\]" },
                { name = "dot-indexed", regex = "(\\.\\w*)|(\\:\\w*)" },
                { name = "symbol",      regex = "\\W" },
            }
        )
    end

    P.help_style = "bad_red_editor_help"
    local function register_help(styling)
        styling:register(
            styling.help_style,
            {
                { name = "keyword", regex = ".*\\:" },
                { name = "type", regex = ".* -"},
            }
        )
    end

    P.init = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.set_hook("buffer_file_linked", function(change)
                local buffer = red.buffer:new(change.values.buffer_id)
                local file = red.file.new(change.values.file_id)
                local extension = file:extension()

                self:set_buffer_type(buffer, extension)
            end))
            register_lua(self)
            register_help(self)
        end,
        "init",
        [[
Initializes the styling system for the editor.
]],
        nil,
        [[
nil
]],
        [[
self: Styling Module Table
]]
    )

    P.set_buffer_type = red.doc.build_fn(
        function(self, buffer, extension)
            local style_list = self.style_map[extension]
            buffer:clear_styles()

            if style_list == nil then
                return
            end

            for _, style in ipairs(style_list) do
                buffer:push_style(style.name, style.regex)
            end
        end,
        "set_buffer_type",
        [[
Sets the given buffer's style based on the given file type extension.
]],
        nil,
        "nil",
        [[
self: Styling Package Table
]],
        [[
buffer: Buffer Object Table - The table representing the buffer whose style should be set.
]],
        [[
extension: String - The file extension to treat this buffer as.
]]
    )

    P.register = red.doc.build_fn(
        function(self, extension, style_list)
            self.style_map[extension] = style_list
        end,
        "register",
        [[
Sets a list of style name and regexes for a given file extension.
]],
        nil,
        [[
nil
]],
        [[
self: Styling Module Table
]],
        [[
extension: String - The file extension on which this set of styles should be applied.
]],
        [[
style_list: Array({ name = String, regex = String }) - List of name/regex pairs that make up this file extension's stylings.
]]
    )

    P.keyword = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("keyword", background, foreground)
        end,
        "keyword",
        "Set color styling for keyword text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    P.comment = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("comment", background, foreground)
        end,
        "comment",
        "Set color styling for comment text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    P.string = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("string", background, foreground)
        end,
        "string",
        "Set color styling for string text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    P.symbol = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("symbol", background, foreground)
        end,
        "symbol",
        "Set color styling for symbol text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    P.dot_indexed = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("dot_indexed", background, foreground)
        end,
        "dot_indexed",
        "Set color styling for dot-indexed text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    P.default = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("default", background, foreground)
        end,
        "default",
        "Set color styling for default text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    P.type = red.doc.build_fn(
        function(background, foreground)
            red.set_text_style("type", background, foreground)
        end,
        "type",
        "Set color styling for type text",
        nil,
        "nil",
        "background: Color",
        "foreground: Color"
    )

    _G[modname] = red.doc.document_table(
        P,
        "styling",
        [[
Functions and data related to the Regex-based text styling system
]],
        nil,
        {},
        function(_, val_doc)
            return "== Package: Styling ==\n" .. val_doc
        end
    )
    return P
end
