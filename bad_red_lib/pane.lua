-- This file is part of BadRed.

-- BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
-- 
-- BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

local P = {}
Pane = P

function P:new(id)
    local instance = { id = id }
    setmetatable(instance, self)
    self.__index = self
    return instance
end

function P:current()
    local id = coroutine.yield(red.call.active_pane_index())
    return self:new(id)
end

function P:to_sibling()
    local is_left = coroutine.yield(red.call.pane_is_first(self.id))
    local parent_id = coroutine.yield(red.call.pane_index_up_from(self.id))
    local sibling_id = coroutine.yield(red.call.pane_index_down_from(parent_id, not is_left))
    coroutine.yield(red.call.set_active_pane(sibling_id))
    self.id = sibling_id
end

function P:to_parent()
    local parent_id = coroutine.yield(red.call.pane_index_up_from(self.id))
    coroutine.yield(red.call.set_active_pane(parent_id))
    self.id = parent_id
end

function P:to_child(is_left)
    local child_id = coroutine.yield(red.call.pane_index_down_from(self.id, is_left))
    coroutine.yield(red.call.set_active_pane(child_id))
    self.id = child_id
end

function P:v_split()
    coroutine.yield(red.call.pane_v_split(self.id))
    self.id = coroutine.yield(red.call.pane_index_down_from(self.id, is_left))
end

function P:h_split()
    coroutine.yield(red.call.pane_h_split(self.id))
    self.id = coroutine.yield(red.call.pane_index_down_from(self.id, is_left))
end

return Pane
