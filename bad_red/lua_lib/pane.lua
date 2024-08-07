-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
--
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

package.preload["pane"] = function(modname, _)
    local P = {}

    P.new = red.doc.build_fn(
        function(self, id)
            local instance = { _id = id }
            setmetatable(instance, self)
            self.__index = self
            return instance
        end,
        "new",
        [[
Creates a Lua Pane table representing a pane in the BadRed editor with the provided ID.
]],
        [[
Does not create the pane in the editor data. Provided ID must therefore already be a valid ID for an existing pane. New pane object will use this table as its metatable to inherit from.
]],
        [[
Pane - A Pane table representing the editor pane with provided ID.
]],
        [[
self: Pane - The table this new Pane should inherit from.
]],
        [[
id: non-negative integer - The ID of the pane this new Pane table should represent.
]]
    )

    P.id = red.doc.build_fn(
        function(self)
            return self._id or P:current()._id
        end,
        "id",
        [[
The Pane ID of this pane table.
]],
        [[
If called on the `Pane` class table directly, returns the ID of the current active pane in the editor.
]],
        [[
non-negative integer - ID of this pane or, if called statically against the Pane class table, the current pane's ID.
]],
        [[
self: Pane - A pane object table or the `Pane` class table to get the ID from.
]]
    )

    P.current = red.doc.build_fn(
        function(self)
            local id = coroutine.yield(red.call.active_pane_index())
            return self:new(id)
        end,
        "current",
        [[
Instantiate a pane table representing the currently active pane in the editor.
]],
        nil,
        [[
Pane - Table representing the currently active pane in the editor.
]],
        [[
self: Pane - A pane table for this current pane table to inherit from.
]]
    )

    P.root = red.doc.build_fn(
        function(self)
            local id = coroutine.yield(red.call.root_pane_index())
            return self:new(id)
        end,
        "root",
        [[
Instantiate a pane table representing the root pane in the editor.
]],
        nil,
        [[
Pane - Table representing the root pane.
]],
        [[
self: Pane - A pane table for the new root pane table to inherit from.
]]
    )

    P.set_active = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.set_active_pane(self:id()))
        end
        ,
        "set_active",
        [[
Makes this Pane the active pane in the editor.
]],
        nil,
        [[
nil
]],
        [[
self: Pane - The pane to make active in the editor.
]]
    )

    P.is_first_child = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.pane_is_first(self:id()))
        end,
        "is_first_child",
        [[
Returns true if this pane is its parent's first child.
]],
        [[
Returns false if this pane is its parent's second child. Returns nil if this is the root pane and has no parent.
]],
        [[
boolean? - True if this is its parent's first child. False if this is its parent's second child. Nil if this has no parent (i.e., it is the root pane).
]],
        [[
self: Pane - The pane whose child value is returned.
]]
    )

    P.sibling = red.doc.build_fn(
        function(self)
            local is_first_child = self:is_first_child()
            local parent = self:parent()
            return parent:child(not is_first_child)
        end,
        "sibling",
        [[
Returns a Pane table representing the sibling of this Pane.
]],
        [[
Returns nil if this is the root pane. If called against the static Pane class table, uses the active pane instead.
]],
        [[
Pane? - A pane table representing this pane's sibling. Nil if this pane is root and has no sibling.
]],
        [[
self: Pane - The pane whose sibling is returned.
]]
    )

    P.parent = red.doc.build_fn(
        function(self)
            local parent_id = coroutine.yield(red.call.pane_index_up_from(self:id()))
            return P:new(parent_id)
        end,
        "parent",
        [[
Returns a Pane table representing the parent of this Pane.
]],
        [[
Returns nil if this is the root pane. If called against the static Pane class table, uses the active pane instead.
]],
        [[
Pane? - A pane table representing this pane's parent. Nil of this pane is root and has no parent.
]],
        [[
self: Pane - The pane whose parent is returned.
]]
    )

    P.child = red.doc.build_fn(
        function(self, to_first)
            local child_id = coroutine.yield(red.call.pane_index_down_from(self:id(), to_first))
            return P:new(child_id)
        end,
        "child",
        [[
Returns a Pane table representing a child of this Pane as specified by the input flag.
]],
        [[
Returns nil if this pane is a leaf and has no children. If called agaisnt the static Pane class table, uses the active pane instead.
]],
        [[
Pane? - A pane table representing this pane's first or second child. Nil if this pane is a leaf and has no children.
]],
        [[
self: Pane - The pane whose child is returned.
]],
        [[
to_first: boolean - True if the first child should be returned. False if the second child should be returned. All panes are binary tree nodes or leaf nodes with either 2 or 0 children.
]]
    )

    P.on_close = red.doc.build_fn(
        function(self, run)
            coroutine.yield(red.call.set_hook("pane_closed", run, self:id()))
        end,
        "on_close",
        [[
Sets a function to be called as a new script immediately after this pane is closed.
]],
        [[
Will interrupt and run prior to the continuation of the script that triggered the close.
]],
        [[
nil
]],
        [[
self: Pane - The pane whose closing should trigger the immediate startup of the provided function.
]],
        [[
run: Function - The function that should be called when this pane closes. Is called with no arguments.
]]
    )

    P.type = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.pane_type(self:id()))
        end,
        "type",
        [[
Returns the type of this Pane. Is Leaf, HSplit, or VSplit.
]],
        [[
If called on the static Pane class table, returns the type of the active pane.
]],
        [[
Pane Type Table - Enum table with type "pane_node_type" and "variant" field "Leaf", "VSplit", or "HSplit"
]],
        [[
self: Pane - The pane whose type is returned.
]]
    )

    P.wrap = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.pane_wrap(self:id()))
        end,
        "wrap",
        [[
Returns the `wrap` flag for this pane.
]],
        [[
If called on the static Pane class table, returns the type of the active pane.
]],
        [[
self: Pane - The pane whose `wrap` flag is returned.
]],
        [[
boolean - True if the pane is set to text wrap. False if not.
]]
    )

    P.set_wrap = red.doc.build_fn(
        function(self, wrap)
            return coroutine.yield(red.call.pane_set_wrap(self:id(), wrap))
        end,
        "set_wrap",
        [[
Sets the `wrap` flag for this pane.
]],
        [[
If called on the static Pane class table, sets the `wrap` flag for the currenlty active pane.
]],
        [[
nil
]],
        [[
self: Pane - The pane whose `wrap` flag is set.
]],
        [[
wrap: boolean - The new `wrap` value to set
]]
    )

    P.v_split = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.pane_v_split(self:id()))
        end
        ,
        "v_split",
        [[
Creates a new parent pane node for this pane of type VSplit with this pane as the new parent's first child.
]],
        [[
Creates a new sibling node under the new parent as the new parent's second child. Sets the new sibling's buffer to be the same as this pane's buffer or, if this pane is not a leaf node, the buffer of this pane's nearest first leaf child.
]],
        [[
nil
]],
        [[
self: Pane - The pane to split.
]]
    )

    P.h_split = red.doc.build_fn(
        function(self)
            coroutine.yield(red.call.pane_h_split(self:id()))
        end,
        "h_split",
        [[
Creates a new parent pane node for this pane of type HSplit with this pane as the new parent's first child.
]],
        [[
Creates a new sibling node under the new parent as the new parent's second child. Sets the new sibling's buffer to be the same as this pane's buffer or, if this pane is not a leaf node, the buffer of this pane's nearest first leaf child.
]],
        [[
nil
]],
        [[
self: Pane - The pane to split.
]]
    )

    P.close = red.doc.build_fn(
        function(self)
            local is_first = self:is_first_child()
            if is_first == nil then
                return
            end

            self:parent():close_child(is_first)
        end,
        "close",
        [[
Close this pane and its parent, removing it from the pane tree.
]],
        [[
This pane's sibling replaces this pane's former parent in the pane tree. If called on the static Pane class table, closes the active pane.
]],
        [[
nil
]],
        [[
self: Pane - The pane to close along with its parent.
]]
    )

    P.close_child = red.doc.build_fn(
        function(self, first_child)
            coroutine.yield(red.call.pane_close_child(self:id(), first_child))
        end,
        "close_child",
        [[
Closes one of the children of this pane, removing it from the pane tree.
]],
        [[
Also removes this pane from the tree and replaces it with the unclosed child pane.
]],
        [[
nil
]],
        [[
self: Pane- The pane whose child should be closed.
]],
        [[
first_child: boolean - Which child should be closed. True if the first child should be closed. False if the second child should be closed.
]]
    )

    P.increase_size = red.doc.build_fn(
        function(self)
            local is_first_child = self:is_first_child()
            local parent = self:parent()
            local parent_type = parent:type()

            if parent_type.variant == "leaf" then
                return
            end

            local split = parent_type.values[1].values
            local split_type = split.split_type
            if split_type.variant == "percent" then
                local percent = split_type.values.first_percent
                local shift
                if is_first_child then
                    shift = 0.1
                else
                    shift = -0.1
                end
                local new_percent = percent + shift
                if new_percent < 0.0 then
                    new_percent = 0.0
                elseif new_percent > 1.0 then
                    new_percent = 1.0
                end

                coroutine.yield(red.call.pane_set_split_percent(parent.id, new_percent))
            elseif split.type == "first_fixed" then
            elseif split.type == "second_fixed" then
            end
        end,
        "increase_size",
        [[
Increases the size of this pane within its parent's split.
]],
        [[
If this pane is shown within a percentage split, increases its size by 10 percentage points. If this pane is shown within a fixed split, increases its size by 1 row or decreases its sibling's mandatory size by 1 row as defined by the split type.
]],
        [[
nil
]],
        [[
self: Pane - Pane object to increase the size of.
]]
    )

    P.decrease_size = red.doc.build_fn(
        function(self)
            local is_first_child = self:is_first_child()
            local parent = self:parent()
            local parent_type = parent:type()

            if parent_type.variant == "leaf" then
                return
            end

            local split = parent_type.values[1].values
            local split_type = split.split_type
            if split_type.variant == "percent" then
                local percent = split_type.values.first_percent
                local shift
                if is_first_child then
                    shift = -0.1
                else
                    shift = 0.1
                end
                local new_percent = percent + shift
                if new_percent < 0.0 then
                    new_percent = 0.0
                elseif new_percent > 1.0 then
                    new_percent = 1.0
                end

                coroutine.yield(red.call.pane_set_split_percent(parent.id, new_percent))
            elseif split.type == "first_fixed" then
            elseif split.type == "second_fixed" then
            end
        end,
        "decrease_size",
        [[
Decreases the size of this pane within its parent's split.
]],
        [[
If this pane is shown within a percentage split, decreases its size by 10 percentage points. If this pane is shown within a fixed split, decreases its size by 1 row or decreases its sibling's mandatory size by 1 row as defined by the split type.
]],
        [[
nil
]],
        [[
self: Pane - Pane object to decrease the size of.
]]
    )

    P.fix_size = red.doc.build_fn(
        function(self, size, on_first_child)
            coroutine.yield(red.call.pane_set_split_fixed(self:id(), size, on_first_child))
        end,
        "fix_size",
        [[
Changes this split pane's split type to be a fixed split with given size on a given child.
]],
        nil,
        [[
nil
]],
        [[
self: Pane - The Pane object to set its split type on. Must be a split pane type, does not work on leaf panes.
]],
        [[
size: non-negative integer - The number of rows/cols the given child should be fixed to be.
]],
        [[
on_first_child: boolean - True if the first child of this pane should be the one to have its size fixed. False if the second child should be the one to have its size fixed.
]]
    )

    P.flex_size = red.doc.build_fn(
        function(self, percent, on_first_child)
            coroutine.yield(red.call.pane_set_split_percent(self:id(), percent, on_first_child))
        end,
        "flex_size",
        [[
Changes this split pane's split type to be a percentage (or flex) split with the given percentage on a given child.
]],
        nil,
        [[
nil
]],
        [[
self: Pane - The Pane object to set its split type on. Must be a split pane type, does not work on leaf panes.
]],
        [[
percent: float from (0.0, 1.0) - The percentage of the parent's size the given child should be flexed to.
]],
        [[
on_first_child: boolean - True if the first child of this pane should be the one whose percentage is set. Fales if the second child should be the one whose percentage is set.
]]
    )

    P.buffer = red.doc.build_fn(
        function(self)
            local buffer_id = coroutine.yield(red.call.pane_buffer_index(self:id()))
            return red.buffer:new(buffer_id)
        end,
        "buffer",
        [[
Returns the buffer object that this pane is displaying.
]],
        [[
If called on the static Pane class table, returns the buffer linked to the active pane.
]],
        [[
Buffer - Buffer table that this pane is currently displaying.
]],
        [[
self: Pane - Table whose buffer is returned.
]]
    )

    P.set_buffer = red.doc.build_fn(
        function(self, buffer)
            coroutine.yield(red.call.pane_set_buffer(self:id(), buffer:id()))
        end,
        "set_buffer",
        [[
Changes the buffer that this pane is currently displaying.
]],
        [[
If called on the static Pane class table, sets the buffer for the current active pane.
]],
        [[
nil
]],
        [[
self: Pane - Table whose buffer should be changed.
]],
        [[
buffer: Buffer Table - A table representing the buffer that this pane should be set to display.
]]
    )

    P.top_line = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.pane_top_line(self:id()))
        end,
        "top_line",
        [[
Return top line index of this pane.
]],
        [[
If this pane has no pane_id, will return the top line of the currently active pane.
]],
        [[
non-negative integer (16-bit)
]],
        [[
self: Pane Table - The pane whose top line is returned. If self has no `pane_id` field, uses the currently active pane.
]]
    )

    P.set_top_line = red.doc.build_fn(
        function(self, top_line)
            coroutine.yield(red.call.pane_set_top_line(self:id(), top_line))
        end,
        "set_top_line",
        [[
Set a new top line for this pane.
]],
        [[
If this pane has no pane_id, will set the top ilne of the currently active pane.
]],
        [[
nil
]],
        [[
self: Pane Table - The pane whose top line is set. If self as no `pane_id` field, sets the currently active pane.
]],
        [[
top_line: non-negative 16-bit integer
]]
    )

    P.frame = red.doc.build_fn(
        function(self)
            return coroutine.yield(red.call.pane_frame(self:id())).values
        end,
        "frame",
        [[
Returns the editor frame table for this pane's current size.
]],
        nil,
        [[
Frame table - Holds all frame information for this pane. 'x_col', 'y_row', 'rows', 'cols'. All non-negative integers maxing at 16 bit values.
]],
        [[
self: Pane Table - The pane table whose frame should be returned. If this table has no pane_id, the active pane will be used.
]]
    )

    red.doc.document_table(
        P,
        "Pane",
        [[
Class table for pane related interaction with the BadRed editor.
]],
        [[
The class Pane table itself represents static access to the active pane in the editor. Calling all functions on the pane class table will lookup the currently active pane before executing the function on that pane object. Pane table functions which return new Pane objects represent specific panes in the editor and inherit from the static Pane class table's functions for calling specific functions on a specific pane.

An editor pane being closed will invalidate any matching pane objects help within Lua scripts and cause errors when trying to call functions on invalidated pane tables. Handling cleanup on pane close is done by registering a callback with a Pane's `on_close` function.
]],
        {},
        function(_, value_doc)
            return "== Class: Pane ==\n" .. value_doc
        end
    )

    _G[modname] = P
    return P
end
