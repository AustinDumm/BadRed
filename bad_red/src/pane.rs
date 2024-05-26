pub type Result<T> = std::result::Result<T, String>;

pub struct PaneTree {
    pub tree: Vec<PaneNode>,
}

impl PaneTree {
    pub fn new(initial_buffer_id: usize) -> Self {
        Self {
            tree: vec![PaneNode {
                node_type: PaneNodeType::Leaf(Pane::new(initial_buffer_id)),
                parent_index: None,
            }],
        }
    }

    pub fn pane_node_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a PaneNode> {
        self.tree.get(pane_index)
    }

    pub fn pane_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a Pane> {
        self.tree
            .get(pane_index)
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
                first_percent: split_percentage,
            })
        })
    }

    pub fn hsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<usize> {
        self.split(pane_id, new_pane_buffer, |top, bottom, split_percentage| {
            PaneNodeType::HSplit(Split {
                first: top,
                second: bottom,
                first_percent: split_percentage,
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
        let moved_content_pane_index = self.tree.len() + 1;

        let current = self.tree.get_mut(pane_id).ok_or_else(|| {
            format!(
                "Failed to find pane for current id while splitting: {}",
                pane_id
            )
        })?;
        let current_parent = current.parent_index;
        current.parent_index = Some(pane_id);

        let new_content_pane = PaneNode {
            node_type: PaneNodeType::Leaf(Pane {
                top_line: 0,
                buffer_id: new_pane_buffer,
            }),
            parent_index: Some(pane_id),
        };

        let new_split_pane = PaneNode {
            node_type: split_constructor(moved_content_pane_index, new_content_pane_index, 0.5),
            parent_index: current_parent,
        };
        self.tree.push(new_content_pane);
        self.tree.push(new_split_pane);
        self.tree.swap(pane_id, moved_content_pane_index);

        Ok(moved_content_pane_index)
    }
}

pub struct PaneNode {
    pub node_type: PaneNodeType,
    pub parent_index: Option<usize>,
}

pub enum PaneNodeType {
    Leaf(Pane),
    VSplit(Split),
    HSplit(Split),
}

pub struct Split {
    pub first: usize,
    pub second: usize,
    pub first_percent: f32,
}

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
