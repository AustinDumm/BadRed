red.buffer = require("buffer")

coroutine.yield(red.call.set_hook("key_event", function(event)
    red.buffer:current():insert_at_cursor(event)
end))

