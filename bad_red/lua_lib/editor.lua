package.preload["editor"] = function(modname, _)
    local P = {}

    P.exit = red.doc.build_fn(
        function()
            coroutine.yield(red.call.editor_exit())
        end,
        "exit",
        [[
Immediately close the editor.
]],
        [[
Will not check for unsaved files or other reasons to not close the editor. Will immediately force the editor closed.
]],
        [[
nil
]]
    )

    P.set_text_style = red.doc.build_fn(
        function(name, background, foreground)
            coroutine.yield(red.call.set_text_style(name, background, foreground))
        end,
        "set_text_style",
        [[
Sets the text styling for a named text style. The given styling will be used whenever a buffer's style stack returns a name for a given style regex.
]],
        nil,
        [[
nil
]],
        [[
name: String - The style name that should be set with this background and foreground.
]],
        [[
background: Color Table - The rgb color table for the text's background. See: red.rgb
]],
        [[
foreground: Color Table - The rgb color table for the text's foreground. See: red.rgb
]]
    )

    P.rgb = red.doc.build_fn(
        function(r, g, b)
            return {
                type = "Color",
                values = {
                    r = r,
                    g = g,
                    b = b
                }
            }
        end,
        "rgb",
        [[
Builds a color table given rgb values
]],
        nil,
        [[
Color Table - Table representing the given color. To be used with `red` functions that expect colors.
]],
        [[
r: Integer Byte - Red component of the color [0, 255]
]],
        [[
g: Integer Byte - Green component of the color [0, 255]
]],
        [[
b: Integer Byte - Blue component of the color [0, 255]
]]
    )

    _G[modname] = P
    return P
end
