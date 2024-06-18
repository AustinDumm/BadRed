// This file is part of BadRed.

// BadRed is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//
// BadRed is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

use bad_red_proc_macros::auto_lua;

pub type Result<T> = std::result::Result<T, String>;

pub struct PaneTree {
    root_index: usize,
    pub tree: Vec<Option<PaneNode>>,
}

impl PaneTree {
    pub fn new(initial_buffer_id: usize) -> Self {
        Self {
            root_index: 0,
            tree: vec![Some(PaneNode {
                node_type: PaneNodeType::Leaf(Pane::new(initial_buffer_id)),
                parent_index: None,
                is_dirty: true,
            })],
        }
    }

    pub fn root_index(&self) -> usize {
        self.root_index
    }

    pub fn root_pane<'a>(&'a self) -> Option<&'a PaneNode> {
        self.pane_node_by_index(self.root_index)
    }

    pub fn pane_node_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a PaneNode> {
        self.tree.get(pane_index).map(|i| i.as_ref()).flatten()
    }

    pub fn pane_node_mut_by_index<'a>(&'a mut self, pane_index: usize) -> Option<&'a mut PaneNode> {
        self.tree.get_mut(pane_index).map(|i| i.as_mut()).flatten()
    }

    pub fn pane_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a Pane> {
        self.tree
            .get(pane_index)
            .map(|i| i.as_ref())
            .flatten()
            .map(|node| match &node.node_type {
                PaneNodeType::Leaf(pane) => Some(pane),
                PaneNodeType::VSplit(_) | PaneNodeType::HSplit(_) => None,
            })
            .flatten()
    }

    pub fn vsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<usize> {
        self.split(pane_id, new_pane_buffer, |left, right, split_percentage| {
            PaneNodeType::VSplit(Split {
                first: left,
                second: right,
                split_type: SplitType::Percent {
                    first_percent: split_percentage,
                },
            })
        })
    }

    pub fn hsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<usize> {
        self.split(pane_id, new_pane_buffer, |top, bottom, split_percentage| {
            PaneNodeType::HSplit(Split {
                first: top,
                second: bottom,
                split_type: SplitType::Percent {
                    first_percent: split_percentage,
                },
            })
        })
    }

    fn split(
        &mut self,
        pane_id: usize,
        new_pane_buffer: usize,
        split_constructor: impl FnOnce(usize, usize, f32) -> PaneNodeType,
    ) -> Result<usize> {
        let new_content_pane_index = self.tree.len();
        let split_root_index = self.tree.len() + 1;

        let current = self.pane_node_mut_by_index(pane_id).ok_or_else(|| {
            format!(
                "Failed to find pane for current id while splitting: {}",
                pane_id
            )
        })?;
        let current_parent = current.parent_index;
        current.parent_index = Some(split_root_index);

        let new_content_pane = PaneNode {
            node_type: PaneNodeType::Leaf(Pane {
                top_line: 0,
                buffer_id: new_pane_buffer,
            }),
            parent_index: Some(split_root_index),
            is_dirty: true,
        };

        let new_split_pane = PaneNode {
            node_type: split_constructor(pane_id, new_content_pane_index, 0.5),
            parent_index: current_parent,
            is_dirty: true,
        };
        self.tree.push(Some(new_content_pane));
        self.tree.push(Some(new_split_pane));

        if let Some(current_parent) = current_parent {
            let parent_node = self.pane_node_mut_by_index(current_parent).ok_or_else(|| {
                format!(
                    "Failed to find expected parent node at index: {}",
                    current_parent
                )
            })?;
            match parent_node.node_type {
                PaneNodeType::Leaf(_) => Err(format!(
                    "Found parent node to be a leaf at index: {}",
                    current_parent
                )),
                PaneNodeType::VSplit(ref mut split) | PaneNodeType::HSplit(ref mut split) => {
                    if split.first == pane_id {
                        split.first = split_root_index;
                        Ok(())
                    } else if split.second == pane_id {
                        split.second = split_root_index;
                        Ok(())
                    } else {
                        Err(format!("Found parent node for index {} at index {} that did not have correct child indices: (first: {}, second: {})", pane_id, current_parent, split.first, split.second))
                    }
                }
            }?;
        } else if self.root_index == pane_id {
            self.root_index = split_root_index;
        } else {
            return Err(format!(
                "Found pane node {} with no parent that is not the root node ({})",
                pane_id, self.root_index
            ));
        }

        Ok(split_root_index)
    }

    pub fn close_child(
        &mut self,
        parent_index: usize,
        first_child: bool,
        active_pane_index: usize,
    ) -> Result<Option<usize>> {
        let parent_node = self.pane_node_by_index(parent_index).ok_or_else(|| {
            format!(
                "Attempted to close child of pane node at invalid index: {}",
                parent_index
            )
        })?;

        let grandparent_index = parent_node.parent_index;

        let (child_to_keep, child_to_close) = match &parent_node.node_type {
            PaneNodeType::Leaf(_) => Err(format!(
                "Attempted to close child for pane node index that is leaf node: {}",
                parent_index
            )),
            PaneNodeType::VSplit(split) | PaneNodeType::HSplit(split) => {
                if first_child {
                    Ok((split.second, split.first))
                } else {
                    Ok((split.first, split.second))
                }
            }
        }?;

        self.pane_node_mut_by_index(child_to_keep)
            .ok_or_else(|| format!("Failed to find child at index: {}", child_to_keep))?
            .parent_index = grandparent_index;

        if let Some(grandparent_index) = grandparent_index {
            let grandparent_node =
                self.pane_node_mut_by_index(grandparent_index)
                    .ok_or_else(|| {
                        format!(
                            "Failed to find grandparent node at index: {}",
                            grandparent_index
                        )
                    })?;
            match grandparent_node.node_type {
                PaneNodeType::Leaf(_) => Err(format!(
                    "Found parent node that is leaf node at index: {}",
                    grandparent_index
                ))?,
                PaneNodeType::VSplit(ref mut split) | PaneNodeType::HSplit(ref mut split) => {
                    if split.first == parent_index {
                        split.first = child_to_keep
                    } else if split.second == parent_index {
                        split.second = child_to_keep
                    } else {
                        Err(format!("Found grandparent node at {} that does not point to child node {} in its split. (first: {}, second: {})", grandparent_index, parent_index, split.first, split.second))?
                    }
                }
            }
        } else if self.root_index == parent_index {
            self.root_index = child_to_keep
        } else {
            Err(format!(
                "Tried to close child of node {} without a parent that is not the root index {}.",
                parent_index, self.root_index
            ))?
        }

        let active_pane_closed = self.close_with_children(child_to_close, active_pane_index);

        if active_pane_closed {
            Ok(Some(child_to_keep))
        } else {
            Ok(None)
        }
    }

    fn close_with_children(&mut self, index: usize, active_pane_index: usize) -> bool {
        true
    }
}

pub struct PaneNode {
    pub node_type: PaneNodeType,
    pub parent_index: Option<usize>,
    pub is_dirty: bool,
}

impl PaneNode {
    pub fn set_type(&mut self, node_type: PaneNodeType) {
        self.node_type = node_type
    }
}

#[auto_lua]
#[derive(Clone, Debug)]
pub enum PaneNodeType {
    Leaf(Pane),
    VSplit(Split),
    HSplit(Split),
}

#[auto_lua]
#[derive(Clone, Debug)]
pub struct Split {
    pub first: usize,
    pub second: usize,
    pub split_type: SplitType,
}

#[auto_lua]
#[derive(Clone, Debug)]
pub enum SplitType {
    Percent { first_percent: f32 },
    FirstFixed { size: u16 },
    SecondFixed { size: u16 },
}

#[auto_lua]
#[derive(Clone, Debug)]
pub struct Pane {
    pub top_line: u16,
    pub buffer_id: usize,
}

impl Pane {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            top_line: 0,
            buffer_id,
        }
    }
}
