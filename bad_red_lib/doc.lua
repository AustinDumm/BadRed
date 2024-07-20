local P = {}

local function build(func, name, fn_doc, ...)
    local doc_table = {}
    local doc_meta = { __call = function (_, ...)
        local args = {...}
        return func(table.unpack(args))
    end }
    setmetatable(doc_table, doc_meta)

    doc_table.name = name
    doc_table.fn_doc = fn_doc
    doc_table.args_docs = {...}

    return doc_table
end

P.build = build(
    build,
    "build",
    [[
Builds a documented function for use in the BadRed doc system.

Ex: `build(function(a, b) ... end, "Example function documentation", "a: int - Argument 1", "b: boolean - Argument 2")`
]],
    [[
func: function - The function that the returned table is documenting. The returned table, when called as a function, runs this function.
]],
    [[
fn_doc: string - A description and documentation string for the function provided. Include details about the function's use as well as usage examples.
]],
    [[
...: variadic strings - A description of each parameter of the function. Structure documentation matching:
<arg-name>: <arg-type> - <arg-description>
]]
)

P._help_pane = nil
P._help_buffer = nil
P.help = build(
    function(fn_to_show)
        if P._help_buffer == nil then
            P._help_buffer = red.buffer:open()
        end

        local doc_string = fn_to_show.fn_doc .. "\nArgs\n    " .. table.concat(fn_to_show.args_docs, "    ")

        P._help_buffer:clear()
        P._help_buffer:insert_at_cursor(doc_string)

        if P._help_pane == nil then
            local root = red.pane:root()
            root:h_split()
            local new_root = root:parent()
            new_root:fix_size(15, false)
            P._help_pane = new_root:child(false)
            P._help_pane:set_wrap(true)
            P._help_pane:on_close(function()
                P._help_pane = nil
            end)
            P._help_pane:set_buffer(P._help_buffer)
        end

        P._help_pane:set_active()
    end,
    "help",
    [[
Displays the help information for a given RedFn.
]],
    [[
red_fn: red_function - The function doc table to show the help information for. To create a red_function, call `build` from the "doc" package.
]]
)

return P
