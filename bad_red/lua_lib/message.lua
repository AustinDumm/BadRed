package.preload["message"] = function(modname, _)
    local P = {}

    P.init = red.doc.build_fn(
        function(self)
            self.buffer = red.buffer:open()
            self.pane = nil
        end,
        "init",
        [[
Initializes the message system for the editor.
]],
        [[
Creates message buffer that remains the singular buffer for the message system unless reset.
]],
        [[
nil
]],
        [[
self: Message package table
]]
    )

    P.send = red.doc.build_fn(
        function(self, message_text)
            self.buffer:set_cursor(self.buffer:length())
            self.buffer:insert(message_text .. "\n")
        end,
        "send",
        [[
Send message text to end of message buffer.
]],
        nil,
        [[
nil
]],
        [[
self: Message package table
]],
        [[
message_text: String - The message to add to the buffer.
]]
    )

    P.open = red.doc.build_fn(
        function(self)
            if self.pane == nil then
                local root = red.pane:root()
                root:h_split()
                local new_root = root:parent()

                new_root:fix_size(10, false)
                self.pane = new_root:child(false)
                self.pane:set_wrap(true)
                self.pane:on_close(function()
                    self.pane = nil
                end)
            end

            self.pane:set_buffer(self.buffer)
            self.pane:set_active()
        end,
        "open",
        [[
Display the message buffer in a new pane for the user.
]],
        [[
Opens the message pane as a 3 row high buffer under the rest of the editor panes.
]],
        [[
nil
]],
        [[
self: Message package table
]]
    )

    red.doc.document_table(
        P,
        "Message",
        [[
Handles functions related to the message buffer and sending messages via the message buffer.
]],
        [[
Message buffer is used for any information sent directly to the user. This can be logging, errors, or other info that is sent to the user about what the editor is doing that does not require response from the user. For response from the user, a prompting and interactive package will later be added to the core BadRed lua packages.
]],
        {},
        function(_, val_doc)
            return "== Package: Message ==\n" .. val_doc
        end
    )

    _G[modname] = P
    return P
end
