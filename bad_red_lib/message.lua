local P = {}

function P:init()
    self.buffer = red.buffer:open()
    self.pane = nil
end

function P:send(message_text)
    self.buffer:set_cursor_index(self.buffer:length())
    self.buffer:insert_at_cursor(message_text .. "\n")
end

function P:open()
    if self.pane == nil then
        local root = red.pane:root()
        root:h_split()
        local new_root = root:parent()

        new_root:fix_size(3, false)
        self.pane = new_root:child(false)
        self.pane:set_wrap(true)
    end

    self.pane:set_buffer(self.buffer)
    self.pane:set_active()
end

return P
