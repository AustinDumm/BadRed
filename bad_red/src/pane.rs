
pub type Result<T> = std::result::Result<T, String>;

pub struct PaneTree {
    pub tree: Vec<PaneNode>,
}


impl PaneTree {
    pub fn new(initial_buffer_id: usize) -> Self {
        Self {
            tree: vec![PaneNode::Leaf(Pane::new(initial_buffer_id))],
        }
    }

    pub fn pane_by_index<'a>(&'a self, pane_index: usize) -> Option<&'a Pane> {
        self.tree.get(pane_index)
            .map(|node| match node {
                PaneNode::Leaf(pane) => Some(pane),
                PaneNode::VSplit(_) |
                PaneNode::HSplit(_) => None
            })
            .flatten()
    }

   // pub fn vsplit(&mut self, pane_id: usize, new_pane_buffer: usize) -> Result<()> {
   //     let pane = self.panes.get_mut(pane_id)
   //         .ok_or(format!("Failed to find pane at id: {}", pane_id))?;
   //     pane.pane_path.push(false);

   //     let split_root_path = pane.pane_path.clone();

   //     let mut new_pane_path = split_root_path.clone();
   //     new_pane_path.push(true);
   //     self.panes.push(Pane::new(new_pane_buffer));

   //     Ok(())
   // }
}

pub enum PaneNode {
    Leaf(Pane),
    VSplit(Split),
    HSplit(Split),
}

pub struct Split {
    pub first: usize,
    pub second: usize,
    pub first_char_size: u16,
}

pub struct Pane {
    pub top_line: u16,
    pub buffer_id: usize,
    pub pane_path: Vec<bool>,
}

impl Pane {
    pub fn new(buffer_id: usize) -> Self {
        Self {
            top_line: 0,
            buffer_id,
            pane_path: vec![],
        }
    }
}
