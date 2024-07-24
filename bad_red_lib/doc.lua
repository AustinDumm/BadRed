local P = {}

P.doc_type_fn = "documented Function"
P.doc_type_table = "documented Table"

local function doc_table(doc_type, name, short_description, long_description, spec_table, doc_str_fn)
    local doc_table_val = {
        type = doc_type,
        name = name,
        short_description = short_description,
        long_description = long_description,
        doc_string = doc_str_fn,
    }

    for k,v in pairs(spec_table) do
        doc_table_val[k] = v
    end

    return doc_table_val
end

local function fn_doc_spec(return_doc, ...)
    return {
        return_doc = return_doc,
        args_docs = { ... }
    }
end

local function build_fn_doc_string(doc_table)
    local long_desc
    if doc_table.long_description then
        long_desc = doc_table.long_description .. "\n\n"
    else
        long_desc = ""
    end

    return doc_table.name .. ": function\n\n" ..
        doc_table.short_description .. "\n\n" ..
        long_desc ..
        "Returns: " .. doc_table.return_doc ..
        "\nArgs\n    " .. table.concat(doc_table.args_docs, "    ")
end

local function build_table_doc_string(doc_table)
    local elements_doc = doc_table.elements
    local by_key_docs = {}
    for _,v in pairs(elements_doc) do
        local element_doc_string
        if string.sub(v, -1) == "\n" then
            element_doc_string = v
        else
            element_doc_string = v .."\n"
        end

        table.insert(by_key_docs, element_doc_string)
    end
    table.sort(by_key_docs)

    local long_desc
    if doc_table.long_description then
        long_desc = doc_table.long_description .. "\n\n"
    else
        long_desc = ""
    end

    return doc_table.name .. ": table\n\n" ..
        doc_table.short_description .. "\n\n" ..
        long_desc ..
        "Elements:\n    " .. table.concat(by_key_docs, "    ") .. "\n\n"
end

local function build_fn(fn, name, short_fn_doc, long_fn_doc, return_doc, ...)
    local fn_table = {
        fn = fn,
        type = P.doc_type_fn,
    }
    local fn_meta = {
        __doc = doc_table(
            P.doc_type_fn,
            name,
            short_fn_doc,
            long_fn_doc,
            fn_doc_spec(
                return_doc,
                ...
            ),
            build_fn_doc_string
        ),
        __call = function(_, ...)
            return fn(...)
        end
    }
    setmetatable(fn_table, fn_meta)

    return fn_table
end

P.is_documented = build_fn(
    function(value)
        if type(value) == "table" then
            local mt = getmetatable(value)
            if mt then
                return mt.__doc ~= nil
            else
                return false
            end
        else
            return false
        end
    end,
    "is_documented",
    [[
Returns whether or not the given value is documented by the BadRed documentation system.
]],
    nil,
    [[
boolean - True if the given value is documented by the documentation system.
]],
    [[
value: Any - The value to be checked for documentation.
]]
)

P.build_fn = build_fn(
    build_fn,
    "build_fn",
    [[
Builds a documented function for use in the BadRed doc system.
]],
    [[
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
short_description: string - A description and documentation string for the function provided. Should be ideally be one or two sentences that fit on roughly one line.
]],
    [[
long_description: string - A detailed description and documentation string for the function provided. Include details about the function's use, edge cases, concepts, and usage examples.
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

local function documented_type(value)
    if P.is_documented(value) then
        local mt = getmetatable(value)
        return mt.__doc.type
    else
        return type(value)
    end
end

local function table_key_doc(key, value, key_doc)
    local key_doc_string
    local key_doc_lookup = key_doc[key]
    if key_doc_lookup then
        key_doc_string = " - " .. key_doc[key]
    elseif P.is_documented(value) then
        key_doc_string = " - " .. getmetatable(value).__doc.short_description
    else
        key_doc_string = ""
    end

    return key .. ": " .. documented_type(value) .. key_doc_string
end

local function table_doc_spec(table_to_doc, key_doc)
    local spec = {}
    for k,v in pairs(table_to_doc) do
        table.insert(spec, table_key_doc(k, v, key_doc))
    end
    return { elements = spec }
end

P.document_table = build_fn(
    function(table_to_doc, name, short_description, long_description, key_doc, value_doc_reformat)
        local doc = doc_table(
            P.doc_type_table,
            name,
            short_description,
            long_description,
            table_doc_spec(table_to_doc, key_doc),
            build_table_doc_string
        )

        local mt = getmetatable(table_to_doc)
        if mt then
            mt.__doc = doc
        else
            setmetatable(table_to_doc, { __doc = doc })
        end

        for k,v in pairs(table_to_doc) do
            if P.is_documented(v) then
                local value_doc_table = getmetatable(v).__doc
                local value_doc_fn = value_doc_table.doc_string
                value_doc_table.doc_string = function(doc_t)
                    local value_doc_str = value_doc_fn(doc_t)
                    return value_doc_reformat(k, value_doc_str)
                end
            end
        end

        return table_to_doc
    end,
    "document_table",
    [[
Adds documentation data to the metatable of the table provided. Returns provided table.
]],
    [[
Documentation information includes name, description, documentation lookup table for each of its keys, and a reformat function for each of its documented values.

If no documentation lookup entry exists for a given key and the value for that key is a documented item, the item's short description will be used.

The reformat function is called whenever one of its documented value's documentations is retrieved. The function is given the key or index of the value being reformatted and the entire doc string its child value creates. It returns the new documentation string to use instead. This can be used to override documentation of its child value by returning an entirely new string, or can add and adjust to its child value's doc string instead.
]],
    [[
documented Table - The original table along with an updated metatable including __doc information.
]]
)

local function primitive_doc(primitive)
    return type(primitive)
end

P._help_pane = nil
P._help_buffer = nil
P.help = build_fn(
    function(doc_to_show)
        if P._help_buffer == nil then
            P._help_buffer = red.buffer:open()
        end

        local doc_string
        local obj_type = type(doc_to_show)
        if obj_type == "table" then
            local mt = getmetatable(doc_to_show)
            local doc = mt.__doc
            if doc then
                doc_string = doc:doc_string()
            else
                doc_string = primitive_doc(doc_to_show)
            end
        else
            doc_string = primitive_doc(doc_to_show)
        end

        P._help_buffer:clear()
        P._help_buffer:insert(doc_string)

        if P._help_pane == nil then
            local root = red.pane:root()
            root:h_split()
            local new_root = root:parent()
            new_root:fix_size(25, false)
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
    nil,
    [[
nil
]],
    [[
red_fn: red_function - The function doc table to show the help information for. To create a red_function, call `build_fn` from the "doc" package.
]]
)

return P
