# TODO

1. Undo/Redo
2. More VIMish bindings
4. Text linking maps?
    a. Look into how other editors handle this
    b. Ranges with destinations?
    c. Add this functionality to docs to allow hypertext in docs
5. Onboarding help?
6. Shortcut mode for command input
    a. If a script returns a callable, call it?
    b. Alternatively could have a special lookup for shortcuts?
    c. Could make this specific only to command input to allow for easier calling of arg-less functions
8. Invisible character render mode?
9. Concept-level documentation
10. Interactive functions (could hook in nicely with #9)
    a. Wrap in a callable table with a interactive runtime that uses a list of parameter names to prompt the user for
11. Macros
12. Built-ins Documentation
    a. Built in functions should be doc tables
       i. Could update these to be fully lua? Or keep partially implemented in Rust from ScriptObject trait/macro?
    b. Hook doc tables
13. Fix pane split to correctly set buffer of new child to be the firstmost leaf node's buffer of the node being split
14. Add hook for non-nil script return data
    a. Used to print the result of a script that evaluates to printable data
15. Add better script spawn/join support.
    a. Is needed for better command shortcut support. The result value of scripts ran in command need to be
        reacted to. This would allow scripts that return literal values where command uses the resulting
        value to run a following script.
16. Add arrow key motions while in insert mode.
17. Add more pane render modes
    a. Floating
    b. Stacked
19. Screen offset follow cursor
20. Decide on how to do associated pane/buffer connections supporting:
    a. Line number gutter
    b. Modeline
    c. Bufferline
    d. Something like a "virtual buffer" that calls a lua delegate for its line-by-line content? How to know when to update? Manual updates to start via hook?
21. Copy/Paste
22. Grep/Search/Replace
23. Profiling mode with logs for RedCall count and times.
    a. Use for picking which lua calls need to be moved into their own Built-in implementation
    b. Cleanup built-in implementation to make it not the world's largest match