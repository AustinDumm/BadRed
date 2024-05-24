pub type Result<T> = std::result::Result<T, String>;

pub struct PaneTree {
    pub tree: Vec<PaneNode>,
}

impl PaneTree {
    pub fn new(initial_buffer_id: usize) -> Self {
        Self {
            tree: vec![PaneNode {
                node_type: PaneNodeType::Leaf(Pane::new(initial_buffer_id)),
            }],
        }
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

    pub fn vsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<()> {
        self.split(pane_id, new_pane_buffer, |top, bottom, split_percentage| {
            PaneNodeType::VSplit(Split {
                first: top,
                second: bottom,
                first_percent: split_percentage,
            })
        });

        Ok(())
    }

    pub fn hsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<()> {
        self.split(pane_id, new_pane_buffer, |left, right, split_percentage| {
            PaneNodeType::HSplit(Split {
                first: left,
                second: right,
                first_percent: split_percentage,
            })
        });

        Ok(())
    }

    fn split(
        &mut self,
        pane_id: usize,
        new_pane_buffer: usize,
        split_constructor: impl FnOnce(usize, usize, f32) -> PaneNodeType,
    ) -> Result<()> {
        let moved_content_pane_index = self.tree.len();
        let new_content_pane_index = self.tree.len() + 1;

        let new_content_pane = PaneNode {
            node_type: PaneNodeType::Leaf(Pane {
                top_line: 0,
                buffer_id: new_pane_buffer,
            }),
        };

        let new_split_pane = PaneNode {
            node_type: split_constructor(moved_content_pane_index, new_content_pane_index, 0.5),
        };
        self.tree.push(new_content_pane);
        self.tree.push(new_split_pane);
        self.tree.swap(pane_id, moved_content_pane_index);

        Ok(())
    }
}

pub struct PaneNode {
    pub node_type: PaneNodeType,
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
