use crate::ViewId;

/// Layout direction for splits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical,
}

/// A node in the layout tree
#[derive(Debug)]
pub enum Node {
    /// A leaf node containing a view
    Leaf(ViewId),
    /// A container with children
    Container {
        layout: Layout,
        children: Vec<Node>,
        /// Size ratios for children (should sum to 1.0)
        ratios: Vec<f32>,
    },
}

impl Node {
    /// Create a new leaf node
    pub fn leaf(view_id: ViewId) -> Self {
        Node::Leaf(view_id)
    }

    /// Create a new container
    pub fn container(layout: Layout, children: Vec<Node>) -> Self {
        let n = children.len();
        let ratio = 1.0 / n as f32;
        Node::Container {
            layout,
            children,
            ratios: vec![ratio; n],
        }
    }

    /// Get all view IDs in this subtree
    pub fn views(&self) -> Vec<ViewId> {
        match self {
            Node::Leaf(id) => vec![*id],
            Node::Container { children, .. } => {
                children.iter().flat_map(|c| c.views()).collect()
            }
        }
    }

    /// Check if this node contains a view
    pub fn contains(&self, view_id: ViewId) -> bool {
        match self {
            Node::Leaf(id) => *id == view_id,
            Node::Container { children, .. } => {
                children.iter().any(|c| c.contains(view_id))
            }
        }
    }

    /// Find the sibling view in a given direction
    pub fn find_sibling(&self, view_id: ViewId, direction: Direction) -> Option<ViewId> {
        match self {
            Node::Leaf(_) => None,
            Node::Container { layout, children, .. } => {
                // Check if view_id is a direct child
                let idx = children.iter().position(|c| {
                    matches!(c, Node::Leaf(id) if *id == view_id)
                });

                if let Some(idx) = idx {
                    // Found it, look for sibling
                    let target_idx = match direction {
                        Direction::Left | Direction::Up => idx.checked_sub(1),
                        Direction::Right | Direction::Down => {
                            if idx + 1 < children.len() {
                                Some(idx + 1)
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(target_idx) = target_idx {
                        // Get first view in the sibling subtree
                        return children[target_idx].views().first().copied();
                    }
                } else {
                    // Recurse into children
                    for child in children {
                        if let Some(sibling) = child.find_sibling(view_id, direction) {
                            return Some(sibling);
                        }
                    }
                }

                None
            }
        }
    }
}

/// Direction for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Layout tree managing splits
#[derive(Debug)]
pub struct Tree {
    /// Root node of the tree
    root: Node,
    /// Currently focused view
    focus: ViewId,
}

impl Tree {
    /// Create a new tree with a single view
    pub fn new(view_id: ViewId) -> Self {
        Self {
            root: Node::leaf(view_id),
            focus: view_id,
        }
    }

    /// Get the focused view ID
    pub fn focus(&self) -> ViewId {
        self.focus
    }

    /// Set the focused view
    pub fn set_focus(&mut self, view_id: ViewId) {
        if self.root.contains(view_id) {
            self.focus = view_id;
        }
    }

    /// Get all view IDs
    pub fn views(&self) -> Vec<ViewId> {
        self.root.views()
    }

    /// Split the focused view
    pub fn split(&mut self, new_view_id: ViewId, layout: Layout) {
        let old_focus = self.focus;
        self.split_view(old_focus, new_view_id, layout);
        self.focus = new_view_id;
    }

    /// Split a specific view
    fn split_view(&mut self, view_id: ViewId, new_view_id: ViewId, layout: Layout) {
        self.root = self.split_node(&self.root, view_id, new_view_id, layout);
    }

    fn split_node(&self, node: &Node, view_id: ViewId, new_view_id: ViewId, layout: Layout) -> Node {
        match node {
            Node::Leaf(id) if *id == view_id => {
                // Found the view to split
                Node::container(
                    layout,
                    vec![Node::leaf(*id), Node::leaf(new_view_id)],
                )
            }
            Node::Leaf(id) => Node::leaf(*id),
            Node::Container {
                layout: node_layout,
                children,
                ratios,
            } => {
                let new_children: Vec<_> = children
                    .iter()
                    .map(|c| self.split_node(c, view_id, new_view_id, layout))
                    .collect();

                Node::Container {
                    layout: *node_layout,
                    children: new_children,
                    ratios: ratios.clone(),
                }
            }
        }
    }

    /// Close a view
    pub fn close(&mut self, view_id: ViewId) -> Option<ViewId> {
        let views = self.views();
        if views.len() <= 1 {
            return None; // Can't close the last view
        }

        // Find a new focus before removing
        let new_focus = views.iter().find(|&&id| id != view_id).copied();

        self.root = self.remove_view(&self.root, view_id)?;

        if let Some(new_focus) = new_focus {
            if self.focus == view_id {
                self.focus = new_focus;
            }
            Some(new_focus)
        } else {
            None
        }
    }

    fn remove_view(&self, node: &Node, view_id: ViewId) -> Option<Node> {
        match node {
            Node::Leaf(id) if *id == view_id => None,
            Node::Leaf(id) => Some(Node::leaf(*id)),
            Node::Container {
                layout,
                children,
                ratios: _,
            } => {
                let new_children: Vec<_> = children
                    .iter()
                    .filter_map(|c| self.remove_view(c, view_id))
                    .collect();

                match new_children.len() {
                    0 => None,
                    1 => Some(new_children.into_iter().next().unwrap()),
                    _ => {
                        let n = new_children.len();
                        let ratio = 1.0 / n as f32;
                        Some(Node::Container {
                            layout: *layout,
                            children: new_children,
                            ratios: vec![ratio; n],
                        })
                    }
                }
            }
        }
    }

    /// Navigate to sibling view
    pub fn focus_direction(&mut self, direction: Direction) -> bool {
        if let Some(sibling) = self.root.find_sibling(self.focus, direction) {
            self.focus = sibling;
            true
        } else {
            false
        }
    }

    /// Cycle focus to next view
    pub fn focus_next(&mut self) {
        let views = self.views();
        if let Some(idx) = views.iter().position(|&id| id == self.focus) {
            let next_idx = (idx + 1) % views.len();
            self.focus = views[next_idx];
        }
    }

    /// Cycle focus to previous view
    pub fn focus_prev(&mut self) {
        let views = self.views();
        if let Some(idx) = views.iter().position(|&id| id == self.focus) {
            let prev_idx = if idx == 0 { views.len() - 1 } else { idx - 1 };
            self.focus = views[prev_idx];
        }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new(ViewId::default())
    }
}
