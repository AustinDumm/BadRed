package.preload["file"] = function(modname, _)
    local P = {}

    local function from_id(id)
        local instance = { _id = id }
        setmetatable(instance, P)
        P.__index = P
        return instance
    end


    P.open = red.doc.build_fn(
        function(self, path)
            local id = coroutine.yield(red.call.file_open(path))
            return from_id(id)
        end,
        "open",
        [[
Opens a file on the host's file system for the editor to manage.
]],
        [[
Does not initialize or create directories or paths to the file. Only opens if the path to the file exists. Will create a new file if the file name is not found in the directory provided.
]],
        [[
File table
]],
        [[
self: File Class table - The table provided by the File Package
]],
        [[
path: string - The path to the file that should be opened.
]]
    )

    P.close = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.file_close(self:id()))
        end,
        "close",
        [[
Closes a file and removes it from the editor's internal data.
]],
        [[
Will fail if the file object this table represents does not exist or is no longer part of the editor.
]],
        [[
nil
]],
        [[
self: File table - the table representing the file that needs closing.
]]
    )

    P.from_buffer = red.doc.build_fn(
        function(self, buffer)
            local file_id = coroutine.yield(red.call.buffer_current_file(buffer:id()))
            return from_id(file_id)
        end,
        "from_buffer",
        [[
Initializes a File table from the file currently linked with the provided buffer.
]],
        [[
Fails if the buffer provided does not have a valid ID or is not linked with a file.
]],
        [[
File table - Represents the file that the provided buffer is linked with.
]],
        [[
self: File Class table - The File class table that should instantiate the new File object table.
]],
        [[
buffer: Buffer table - The buffer table whose file should be retrieved and returned. If the Buffer class is provided, will return the file object table representing the file linked to the currently active buffer.
]]
    )

    P.id = red.doc.build_fn(
        function(self)
            return self._id
        end,
        "id",
        [[
Get the File id that this file object table represents.
]],
        nil,
        [[
non-negative integer
]],
        [[
self: File Object table - The object whose id should be returned
]]
    )

    red.doc.document_table(
        P,
        "File",
        [[
The Class table for all File-related interaction with the BadRed editor.
]],
        [[
Contains functions for instantiating new File object tables which represent files the BadRed editor has open. Contains functions for interacting with specific files through the File object tables.
]],
        {},
        function(_, value_doc)
            return "== Class: File ==\n" .. value_doc
        end
    )

    _G[modname] = P
    return P
end
