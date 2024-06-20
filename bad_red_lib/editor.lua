local P = {}

function P.exit()
    coroutine.yield(red.call.editor_exit())
end

return P
