# TODO

2. Undo/Redo
3. More VIMish bindings
5. Text color maps?
    a. Look into how other editors handle this
    b. Ranges with colors?
6. Text linking maps?
    a. Look into how other editors handle this
    b. Ranges with destinations?
    c. Add this functionality to docs to allow hypertext in docs
7. Onboarding help?
9. Shortcut mode for command input
    a. If a script returns a callable, call it?
    b. Alternatively could have a special lookup for shortcuts?
    c. Could make this specific only to command input to allow for easier calling of arg-less functions
10. Better tab/spaces support
11. Invisible character render mode?
12. Concept-level documentation
13. Interactive style functions (could hook in nicely with #9)
    a. Wrap in a callable table with a interactive runtime that uses a list of parameter names to prompt the user for
14. Macros
15. Built-ins Documentation
    a. Built in functions should be doc tables
       i. Could update these to be fully lua? Or keep partially implemented in Rust from ScriptObject trait/macro?
    b. Hook doc tables
17. Fix pane split to correctly set buffer of new child to be the firstmost leaf node's buffer of the node being split
18. Add hook for non-nil script return data
    a. Used to print the result of a script that evaluates to printable data
