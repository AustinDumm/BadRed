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

function P:set_active()
    coroutine.yield(red.call.set_active_pane(self.id))
end

function P:is_first_child()
    return coroutine.yield(red.call.pane_is_first(self.id))
end

function P:sibling()
    local is_first_child = self:is_first_child()
    local parent = self:parent()
    return parent:child(not is_first_child)
end

function P:parent()
    local parent_id = coroutine.yield(red.call.pane_index_up_from(self.id))
    return P:new(parent_id)
end

function P:child(to_first)
    local child_id = coroutine.yield(red.call.pane_index_down_from(self.id, to_first))
    return P:new(child_id)
end

function P:type()
    return coroutine.yield(red.call.pane_type(self.id))
end

function P:v_split()
    coroutine.yield(red.call.pane_v_split(self.id))
    self.id = coroutine.yield(red.call.pane_index_down_from(self.id, is_first_child))
end

function P:h_split()
    coroutine.yield(red.call.pane_h_split(self.id))
    self.id = coroutine.yield(red.call.pane_index_down_from(self.id, is_first_child))
end

function P:increase_size()
    local is_first_child = self:is_first_child()
    local parent = self:parent()
    local parent_type = parent:type()

    if parent_type.type == "Leaf" then
        return
    end

    local split = parent_type.split
    if split.type == "Percent" then
        local percent = split.first_percent
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
        coroutine.yield(red.call.pane_set_split_percent(parent_id, new_percent))

    elseif split.type == "FirstFixed" then
    elseif split.type == "SecondFixed" then
    end
end

function P:decrease_size()
    local is_first_child = self:is_first_child()
    local parent = self:parent()
    local parent_type = parent:type()

    if parent_type.type == "Leaf" then
        return
    end

    local split = parent_type.split
    if split.type == "Percent" then
        local percent = split.first_percent
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
        coroutine.yield(red.call.pane_set_split_percent(parent_id, new_percent))

    elseif split.type == "FirstFixed" then
    elseif split.type == "SecondFixed" then
    end
end

return Pane
