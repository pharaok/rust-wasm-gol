use std::{
    cell::{Cell, RefCell},
    hash::{BuildHasher, Hash, Hasher},
    rc::Rc,
};

pub const LEAF_LEVEL: usize = 1;
pub const LEAF_SIZE: usize = 1 << LEAF_LEVEL;

type Leaf = [[u8; LEAF_SIZE]; LEAF_SIZE];

#[derive(Clone)]
pub enum NodeKind {
    Leaf(Leaf),
    Branch([Rc<RefCell<Node>>; 4]), // [nw, ne, sw, se]
}

impl NodeKind {
    pub fn new_leaf() -> Self {
        Self::Leaf([[0; LEAF_SIZE]; LEAF_SIZE])
    }

    pub fn new_branch(children: [&Rc<RefCell<Node>>; 4]) -> Self {
        Self::Branch(children.map(Rc::clone))
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
    pub fn as_branch(&self) -> &[Rc<RefCell<Node>>; 4] {
        match self {
            Self::Branch(children) => children,
            _ => panic!(),
        }
    }
    pub fn as_branch_mut(&mut self) -> &mut [Rc<RefCell<Node>>; 4] {
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

#[derive(Clone)]
pub struct Node {
    pub node: NodeKind,
    pub level: usize,
    pub population: i32,
    pub memo_hash: Cell<Option<u64>>,
}

impl Node {
    pub fn new(level: usize) -> Self {
        Self {
            node: NodeKind::new_leaf(),
            level,
            population: 0,
            memo_hash: Cell::new(None),
        }
    }
    pub fn new_branch(children: [&Rc<RefCell<Node>>; 4]) -> Self {
        let level = children[0].borrow().level + 1;
        let population = children.map(|c| c.borrow().population).iter().sum();
        Self {
            node: NodeKind::new_branch(children),
            level,
            population,
            memo_hash: Cell::new(None),
        }
    }

    pub fn subdivide(&mut self) {
        if self.level == LEAF_LEVEL {
            panic!();
        }

        if self.is_leaf() {
            // this feels stupid
            self.node = NodeKind::new_branch([
                &Rc::new(RefCell::new(Node::new(self.level - 1))),
                &Rc::new(RefCell::new(Node::new(self.level - 1))),
                &Rc::new(RefCell::new(Node::new(self.level - 1))),
                &Rc::new(RefCell::new(Node::new(self.level - 1))),
            ]);
        }
    }

    pub fn get_child(&self, x: i32, y: i32) -> &Rc<RefCell<Self>> {
        match &self.node {
            NodeKind::Branch([nw, ne, sw, se]) => match (x < 0, y < 0) {
                (true, true) => nw,
                (false, true) => ne,
                (true, false) => sw,
                (false, false) => se,
            },
            NodeKind::Leaf(_) => panic!(),
        }
    }
    pub fn get_child_mut(&mut self, x: i32, y: i32) -> &mut Rc<RefCell<Self>> {
        match &mut self.node {
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

    pub fn get(&self, x: i32, y: i32) -> u8 {
        let half = 1 << (self.level - 1);
        if !(-half <= x && x < half && -half <= y && y < half) {
            // TODO: out of bounds
            return 0;
        }

        if self.level == LEAF_LEVEL {
            let v = self.node.as_leaf();
            return v[(y + half) as usize][(x + half) as usize];
        }

        let (cx, cy) = self.to_child_coords(x, y);
        match &self.node {
            NodeKind::Leaf(_) => 0,
            NodeKind::Branch(_) => self.get_child(x, y).borrow().get(cx, cy),
        }
    }
    pub fn insert(&mut self, x: i32, y: i32, value: u8) -> i32 {
        self.memo_hash.take(); // invalidate hash

        if let NodeKind::Leaf(v) = &mut self.node {
            if self.level == LEAF_LEVEL {
                let half = 1 << (LEAF_LEVEL - 1);
                let (i, j) = ((y + half) as usize, (x + half) as usize);
                let d = value as i32 - v[i][j] as i32;
                self.population += d;

                v[i][j] = value;
                return d;
            }
            self.subdivide();
        }

        let (cx, cy) = self.to_child_coords(x, y);
        let d = self.get_child(x, y).borrow_mut().insert(cx, cy, value);
        self.population += d;
        d
    }

    pub fn get_rect(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<Vec<u8>> {
        let half = 1 << (self.level - 1);
        let (x1, y1, x2, y2) = (x1.max(-half), y1.max(-half), x2.min(half), y2.min(half));

        let (w, h) = ((x2 - x1) as usize, (y2 - y1) as usize);
        let mut grid = vec![vec![0; w]; h];

        self._get_rect(x1, y1, x2, y2, &mut grid);

        grid
    }
    fn _get_rect(&self, x1: i32, y1: i32, x2: i32, y2: i32, grid: &mut Vec<Vec<u8>>) {
        let half = 1 << (self.level - 1);
        if x1 >= half || y1 >= half || x2 < -half || y2 < -half || self.population == 0 {
            return;
        }
        let (w, h) = ((x2 - x1) as usize, (y2 - y1) as usize);

        match &self.node {
            NodeKind::Leaf(v) => {
                if self.level != LEAF_LEVEL {
                    return;
                }
                for i in 0..LEAF_SIZE {
                    for j in 0..LEAF_SIZE {
                        let (x, y) = ((j as i32 - x1 - 1) as usize, (i as i32 - y1 - 1) as usize);
                        if x < w && y < h {
                            grid[y][x] = v[i][j];
                        }
                    }
                }
            }
            NodeKind::Branch([nw, ne, sw, se]) => {
                let q = 1 << (self.level - 2);
                nw.borrow()._get_rect(x1 + q, y1 + q, x2 + q, y2 + q, grid);
                ne.borrow()._get_rect(x1 - q, y1 + q, x2 - q, y2 + q, grid);
                sw.borrow()._get_rect(x1 + q, y1 - q, x2 + q, y2 - q, grid);
                se.borrow()._get_rect(x1 - q, y1 - q, x2 - q, y2 - q, grid);
            }
        }
    }
    pub fn set_rect(&mut self, x: i32, y: i32, grid: &Vec<Vec<u8>>) {
        self._set_rect(x, y, grid);
    }
    fn _set_rect(&mut self, x: i32, y: i32, grid: &Vec<Vec<u8>>) -> i32 {
        let half = 1 << (self.level - 1);
        let (w, h) = (grid[0].len(), grid.len());

        if x >= half || y >= half || x + (w as i32) < -half || y + (h as i32) < -half {
            return 0;
        }

        self.memo_hash.take();

        if let NodeKind::Leaf(v) = &mut self.node {
            if self.level == LEAF_LEVEL {
                let mut d = 0;
                for i in 0..LEAF_SIZE {
                    for j in 0..LEAF_SIZE {
                        let (x, y) = ((j as i32 - x - 1) as usize, (i as i32 - y - 1) as usize);
                        if x < w && y < h {
                            d += grid[y][x] as i32 - v[i][j] as i32;
                            v[i][j] = grid[y][x];
                        }
                    }
                }
                self.population += d;
                return d;
            }

            self.subdivide();
        }
        let [nw, ne, sw, se] = self.node.as_branch_mut();
        let q = 1 << (self.level - 2);
        let mut d = 0;

        d += nw.borrow_mut()._set_rect(x + q, y + q, grid);
        d += ne.borrow_mut()._set_rect(x - q, y + q, grid);
        d += sw.borrow_mut()._set_rect(x + q, y - q, grid);
        d += se.borrow_mut()._set_rect(x - q, y - q, grid);
        self.population += d;
        d
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.node, NodeKind::Leaf(_))
    }
    pub fn is_branch(&self) -> bool {
        matches!(self.node, NodeKind::Branch(_))
    }

    pub fn get_pseudo_child(&self, dx: i32, dy: i32) -> Self {
        if self.level < LEAF_LEVEL + 1 {
            panic!();
        }
        if self.is_leaf() {
            return Self::new(self.level - 1);
        }

        let mut new_node = Self::new(self.level - 1);
        if new_node.level > LEAF_LEVEL {
            new_node.subdivide();
        }

        for y in -1..1i32 {
            for x in -1..1i32 {
                let (mut yy, mut xx) = (y + dy, x + dx);

                let child = self.get_child(xx, yy).borrow();
                (yy, xx) = (yy.rem_euclid(2) - 1, xx.rem_euclid(2) - 1);
                if child.level == LEAF_LEVEL {
                    new_node.insert(x, y, child.get(xx, yy));
                } else if child.is_branch() {
                    let grandchild = child.get_child(xx, yy);
                    *new_node.get_child_mut(x, y) = Rc::clone(grandchild);

                    new_node.population += grandchild.borrow().population
                }
            }
        }

        new_node
    }

    pub fn grown(&self) -> Self {
        if self.population == 0 {
            return Self::new(self.level + 1);
        }

        let mut new_node = Self::new(self.level + 1);
        new_node.subdivide();
        new_node.population = self.population;

        for i in 0..4 {
            let child = &self.node.as_branch()[i];
            let new_child = &new_node.node.as_branch()[i];
            new_child.borrow_mut().subdivide();
            new_child.borrow_mut().node.as_branch_mut()[3 - i] = Rc::clone(child);
            new_child.borrow_mut().population = child.borrow().population;
        }

        new_node.level = self.level + 1;
        new_node
    }

    pub fn get_hash(&self, hasher: &impl BuildHasher) -> u64 {
        let mut state = hasher.build_hasher();

        if self.memo_hash.get().is_none() {
            if self.population == 0 {
                self.level.hash(&mut state);
            } else {
                match &self.node {
                    NodeKind::Leaf(v) => {
                        v.hash(&mut state);
                        self.level.hash(&mut state);
                    }
                    NodeKind::Branch([nw, ne, sw, se]) => {
                        nw.borrow().get_hash(hasher).hash(&mut state);
                        ne.borrow().get_hash(hasher).hash(&mut state);
                        sw.borrow().get_hash(hasher).hash(&mut state);
                        se.borrow().get_hash(hasher).hash(&mut state);
                    }
                }
            }
            self.memo_hash.replace(Some(state.finish()));
        }

        self.memo_hash.get().unwrap()
    }

    pub fn deep_clone(&self) -> Self {
        let mut new_node = Self::new(self.level);
        new_node.population = self.population;

        new_node.node = match &self.node {
            NodeKind::Leaf(v) => NodeKind::Leaf(*v),
            NodeKind::Branch(children) => NodeKind::Branch(
                children
                    .clone()
                    .map(|c| Rc::new(RefCell::new(c.borrow().deep_clone()))),
            ),
        };

        new_node
    }
}
