local P = {}

local function build_fn(func, name, fn_doc, return_doc, ...)
    local doc_table = {}
    local doc_meta = { __call = function (_, ...)
        local args = {...}
        return func(table.unpack(args))
    end }
    setmetatable(doc_table, doc_meta)

    doc_table.name = name
    doc_table.fn_doc = fn_doc
    doc_table.return_doc = return_doc
    doc_table.args_docs = {...}

    return doc_table
end

P.build_fn = build_fn(
build_fn,
"build_fn",
[[
Builds a documented function for use in the BadRed doc system.

Ex: `build_fn(function(a, b) ... end, "Example function documentation", "a: int - Argument 1", "b: boolean - Argument 2")`
]],
[[
red_function - A table object matching the function and documentation strings passed in.
]],
[[
func: function - The function that the returned table is documenting. The returned table, when called as a function, runs this function.
]],
[[
name: string - The name the function is referenced with in the editor scripts. Do not include package, table, or class names as part of the function name.
]],
[[
fn_doc: string - A description and documentation string for the function provided. Include details about the function's use as well as usage examples.
]],
[[
return_doc: string - A description of the return type of the function. Structure documentation matching:
<return-type> - <return-description>
]],
[[
...: variadic strings - A description of each parameter of the function. Structure documentation matching:
<arg-name>: <arg-type> - <arg-description>
]]
)

P._help_pane = nil
P._help_buffer = nil
P.help = build_fn(
    function(fn_to_show)
        if P._help_buffer == nil then
            P._help_buffer = red.buffer:open()
        end

        local doc_string =
            fn_to_show.name .. "\n\n" ..
            fn_to_show.fn_doc .. "\n\n" ..
            "Returns: " .. fn_to_show.return_doc ..
            "\nArgs\n    " .. table.concat(fn_to_show.args_docs, "    ")

        P._help_buffer:clear()
        P._help_buffer:insert(doc_string)

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
nil
]],
[[
red_fn: red_function - The function doc table to show the help information for. To create a red_function, call `build_fn` from the "doc" package.
]]
)

return P
