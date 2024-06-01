coroutine.yield(red.call.set_hook("key_event", function(event)
    coroutine.yield(red.call.current_buffer_insert(event))
end))
