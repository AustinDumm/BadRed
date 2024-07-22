# TODO

1. Address issue where editor can't find init.lua when not run with pwd=./bad_red
    a. Look into build step of compiling init.lua as a built-in dependency
    b. Optionally allow user to provide their own init path that overrides precompiled lua init?
2. Undo/Redo
3. More VIMish bindings
4. Class-level docs
5. Text color maps?
    a. Look into how other editors handle this
    b. Ranges with colors?
6. Text linking maps?
    a. Look into how other editors handle this
    b. Ranges with destinations?
    c. Add this functionality to docs to allow hypertext in docs
7. Onboarding help?
8. Editor-level help
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
16. Table/data documentation
    a. Use for naive/gap buffer objects? Could move to a ScriptObject from within Rust?
17. Fix pane split to correctly set buffer of new child to be the firstmost leaf node's buffer of the node being split
