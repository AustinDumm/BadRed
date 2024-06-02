red.buffer = require("buffer")
red.keymap = require("keymap")

coroutine.yield(red.call.set_hook("key_event", function(event)
    red.keymap.event(event)
end))

