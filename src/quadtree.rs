pub const LEAF_LEVEL: u8 = 2;
pub const LEAF_SIZE: usize = 1 << LEAF_LEVEL;

pub type Leaf = [[u8; LEAF_SIZE]; LEAF_SIZE];
pub type NodeRef = u32;
pub type Branch = [NodeRef; 4]; // [nw, ne, sw, se]

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum NodeKind {
    Leaf(Leaf),
    Branch(Branch),
}

impl NodeKind {
    pub fn new_empty_leaf() -> Self {
        Self::Leaf([[0; LEAF_SIZE]; LEAF_SIZE])
    }
    pub fn new_leaf(data: Leaf) -> Self {
        Self::Leaf(data)
    }

    pub fn new_branch(children: [NodeRef; 4]) -> Self {
        Self::Branch(children)
    }

    pub fn as_leaf(&self) -> &Leaf {
        match self {
            Self::Leaf(v) => v,
            _ => panic!(),
        }
    }
    pub fn as_leaf_mut(&mut self) -> &mut Leaf {
        match self {
            Self::Leaf(v) => v,
            _ => panic!(),
        }
    }
    pub fn as_branch(&self) -> &[NodeRef; 4] {
        match self {
            Self::Branch(children) => children,
            _ => panic!(),
        }
    }
    pub fn as_branch_mut(&mut self) -> &mut [NodeRef; 4] {
        match self {
            Self::Branch(children) => children,
            _ => panic!(),
        }
    }
}

impl From<Leaf> for NodeKind {
    fn from(leaf: Leaf) -> Self {
        Self::Leaf(leaf)
    }
}

#[derive(Clone, Copy)]
pub struct Node {
    pub data: NodeKind,
    pub level: u8,
    pub population: u64,
}

impl Node {
    pub fn new_empty_leaf() -> Self {
        Self {
            data: NodeKind::new_empty_leaf(),
            level: LEAF_LEVEL,
            population: 0,
        }
    }
    pub fn new_leaf(data: Leaf, population: u64) -> Self {
        Self {
            data: NodeKind::new_leaf(data),
            level: LEAF_LEVEL,
            population,
        }
    }
    pub fn new_branch(children: [NodeRef; 4], level: u8, population: u64) -> Self {
        Self {
            data: NodeKind::new_branch(children),
            level,
            population,
        }
    }

    pub fn get_child(&self, x: i32, y: i32) -> NodeRef {
        match self.data {
            NodeKind::Branch([nw, ne, sw, se]) => match (x < 0, y < 0) {
                (true, true) => nw,
                (false, true) => ne,
                (true, false) => sw,
                (false, false) => se,
            },
            NodeKind::Leaf(_) => panic!(),
        }
    }
    pub fn get_child_mut(&mut self, x: i32, y: i32) -> &mut NodeRef {
        match &mut self.data {
            NodeKind::Branch([nw, ne, sw, se]) => match (x < 0, y < 0) {
                (true, true) => nw,
                (false, true) => ne,
                (true, false) => sw,
                (false, false) => se,
            },
            NodeKind::Leaf(_) => panic!(),
        }
    }
    pub fn to_child_coords(&self, x: i32, y: i32) -> (i32, i32) {
        let quarter = 1 << (self.level - 2);
        let half = quarter << 1;
        (x.rem_euclid(half) - quarter, y.rem_euclid(half) - quarter)
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.data, NodeKind::Leaf(_))
    }
    pub fn is_branch(&self) -> bool {
        matches!(self.data, NodeKind::Branch(_))
    }
}

impl From<Node> for NodeKind {
    fn from(node: Node) -> Self {
        node.data
    }
}
