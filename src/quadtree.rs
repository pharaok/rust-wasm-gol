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
    pub fn as_branch(&self) -> &[NodeRef; 4] {
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

    pub fn get_child_index(x: i64, y: i64) -> usize {
        match (y < 0, x < 0) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        }
    }
    pub fn get_child(&self, x: i64, y: i64) -> NodeRef {
        match self.data {
            NodeKind::Branch(children) => children[Self::get_child_index(x, y)],
            NodeKind::Leaf(_) => panic!(),
        }
    }
    pub fn get_child_offset(i: usize, level: u8) -> (i64, i64) {
        let half = 1i64 << (level - 1);
        match i {
            0 => (0, 0),
            1 => (half, 0),
            2 => (0, half),
            3 => (half, half),
            _ => unreachable!(),
        }
    }
    pub fn normalize_coords(x: i64, y: i64, level: u8) -> (i64, i64) {
        let half = 1i64 << (level - 1);
        (x.rem_euclid(2 * half) - half, y.rem_euclid(2 * half) - half)
    }
    pub fn offset_to_child(i: usize, level: u8) -> (i64, i64) {
        let q = 1i64 << (level - 2);
        match i {
            0 => (q, q),
            1 => (-q, q),
            2 => (q, -q),
            3 => (-q, -q),
            _ => unreachable!(),
        }
    }
    pub fn partition_points_mut(
        points: &mut [(i64, i64)],
        level: u8,
        left: i64,
        top: i64,
    ) -> [&mut [(i64, i64)]; 4] {
        let half = 1i64 << (level - 1);

        let split_y = points.iter_mut().partition_in_place(|p| p.1 < top + half);
        let (n_points, s_points) = points.split_at_mut(split_y);

        let split_nx = n_points
            .iter_mut()
            .partition_in_place(|p| p.0 < left + half);
        let split_sx = s_points
            .iter_mut()
            .partition_in_place(|p| p.0 < left + half);
        let (nw_points, ne_points) = n_points.split_at_mut(split_nx);
        let (sw_points, se_points) = s_points.split_at_mut(split_sx);
        [nw_points, ne_points, sw_points, se_points]
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
