-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
-- 
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

red.doc = require("doc")
red.opts = require("opts")
red.buffer = require("buffer")
red.keymap = require("keymap")
red.pane = require("pane")
red.command = require("command")
red.mode = require("mode")
red.file = require("file")
red.message = require("message")
red.styling = require("styling")

for k,v in pairs(require("editor")) do
    red[k] = v
end

red.mode:init(red.keymap)
coroutine.yield(red.call.set_hook("key_event", function(event)
    red.keymap.event(event)
end))

red.message:init()
coroutine.yield(red.call.set_hook("secondary_error", function(message)
    red.buffer:new(0):insert(message)
end))

coroutine.yield(red.call.set_hook("error", function(message)
    red.message:send(message)
    red.message:open()
end))

red.styling:init()

red.doc.document_table(
    red,
    "red",
    [[
Top-level table containing all built-in interfaces with the BadRed text editor
]],
    nil,
    {},
    function(_, value_doc) return value_doc end
)

red.command:set_shortcut("w", function() red.buffer:write() end)
red.command:set_shortcut("q", function() red.exit() end)
red.command:set_shortcut("wq", function()
    red.buffer:write()
    red.exit()
end)

