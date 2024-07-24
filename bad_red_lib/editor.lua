local P = {}

P.exit = red.doc.build_fn(
function()
    coroutine.yield(red.call.editor_exit())
end,
"exit",
[[
Immediately close the editor.
]],
[[
Will not check for unsaved files or other reasons to not close the editor. Will immediately force the editor closed.
]],
[[
nil
]]
)

return P

