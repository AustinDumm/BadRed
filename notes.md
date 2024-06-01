Introduce hooks for key events. Let Lua register functions for
hook events. Can then get rid of the KeyMap complexity within
the rust code and instead let the keymap live entirely within
lua. Also need to create an init.lua call that lets the system
set up the keymap for the key event hook.

