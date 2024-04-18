use std::{
    cell::{Cell, RefCell},
    hash::{BuildHasher, Hash, Hasher},
    rc::Rc,
    vec,
};

use rustc_hash::{FxHashMap, FxHasher};

pub const LEAF_LEVEL: usize = 1;
pub const LEAF_SIZE: usize = 1 << LEAF_LEVEL;

type Leaf = [[u8; LEAF_SIZE]; LEAF_SIZE];

#[derive(Clone, PartialEq, Eq)]
pub enum NodeKind {
    Leaf(Leaf),
    Branch {
        nw: Rc<RefCell<Node>>,
        ne: Rc<RefCell<Node>>,
        sw: Rc<RefCell<Node>>,
        se: Rc<RefCell<Node>>,
    },
}

impl NodeKind {
    pub fn new_leaf() -> Self {
        Self::Leaf([[0; LEAF_SIZE]; LEAF_SIZE])
    }

    pub fn new_branch(
        nw: &Rc<RefCell<Node>>,
        ne: &Rc<RefCell<Node>>,
        sw: &Rc<RefCell<Node>>,
        se: &Rc<RefCell<Node>>,
    ) -> Self {
        Self::Branch {
            nw: Rc::clone(nw),
            ne: Rc::clone(ne),
            sw: Rc::clone(sw),
            se: Rc::clone(se),
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
    pub generation: Cell<u64>,
    pub population: Cell<i32>,
    pub memo_hash: Cell<Option<u64>>,
}

impl Node {
    pub fn new(level: usize) -> Self {
        Self {
            node: NodeKind::new_leaf(),
            level,
            generation: Cell::new(0),
            population: Cell::new(0),
            memo_hash: Cell::new(None),
        }
    }
    pub fn new_branch(
        nw: &Rc<RefCell<Node>>,
        ne: &Rc<RefCell<Node>>,
        sw: &Rc<RefCell<Node>>,
        se: &Rc<RefCell<Node>>,
    ) -> Self {
        let level = nw.borrow().level + 1;
        let population = nw.borrow().population.get()
            + ne.borrow().population.get()
            + sw.borrow().population.get()
            + se.borrow().population.get();
        Self {
            node: NodeKind::new_branch(nw, ne, sw, se),
            level,
            generation: Cell::new(0),
            population: Cell::new(population),
            memo_hash: Cell::new(None),
        }
    }

    pub fn subdivide(&mut self) {
        if self.level == LEAF_LEVEL {
            panic!();
        }

        if let NodeKind::Leaf(_) = self.node {
            let nw = Node::new(self.level - 1);
            let ne = Node::new(self.level - 1);
            let sw = Node::new(self.level - 1);
            let se = Node::new(self.level - 1);

            self.node = NodeKind::new_branch(
                &Rc::new(RefCell::new(nw)),
                &Rc::new(RefCell::new(ne)),
                &Rc::new(RefCell::new(sw)),
                &Rc::new(RefCell::new(se)),
            );
        }
    }

    pub fn get_child(&self, x: i32, y: i32) -> &Rc<RefCell<Self>> {
        match &self.node {
            NodeKind::Branch { nw, ne, sw, se } => match (x < 0, y < 0) {
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
            NodeKind::Branch { nw, ne, sw, se } => match (x < 0, y < 0) {
                (true, true) => nw,
                (false, true) => ne,
                (true, false) => sw,
                (false, false) => se,
            },
            NodeKind::Leaf(_) => panic!(),
        }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if self.level == LEAF_LEVEL {
            let v = match &self.node {
                NodeKind::Leaf(v) => v,
                _ => panic!(),
            };
            if !(0 <= x + 1 && x + 1 < LEAF_SIZE as i32 && 0 <= y + 1 && y + 1 < LEAF_SIZE as i32) {
                return 0;
            }
            return v[(y + 1) as usize][(x + 1) as usize];
        }

        let quarter = 1 << (self.level - 2);

        match &self.node {
            NodeKind::Leaf(_) => 0,
            NodeKind::Branch { nw, ne, sw, se } => match (x < 0, y < 0) {
                (true, true) => nw.borrow().get(x + quarter, y + quarter),
                (false, true) => ne.borrow().get(x - quarter, y + quarter),
                (true, false) => sw.borrow().get(x + quarter, y - quarter),
                (false, false) => se.borrow().get(x - quarter, y - quarter),
            },
        }
    }

    pub fn insert(&mut self, x: i32, y: i32, value: u8) -> i32 {
        self.memo_hash.take(); // invalidate cached hash

        let half = 1 << (self.level - 1);

        if let NodeKind::Leaf(v) = &mut self.node {
            if self.level == LEAF_LEVEL {
                let (i, j) = ((y + half) as usize, (x + half) as usize);
                let d = value as i32 - v[i][j] as i32;
                self.population.set(self.population.get() + d);

                v[i][j] = value;
                return d;
            }
            self.subdivide();
        }

        match &mut self.node {
            NodeKind::Branch { nw, ne, sw, se } => {
                let quarter = 1 << (self.level - 2);
                match (x < 0, y < 0) {
                    (true, true) => {
                        let cloned = nw.borrow().clone();
                        *nw = Rc::new(RefCell::new(cloned));
                        let d = nw.borrow_mut().insert(x + quarter, y + quarter, value);
                        self.population.set(self.population.get() + d);
                        d
                    }
                    (false, true) => {
                        let cloned = ne.borrow().clone();
                        *ne = Rc::new(RefCell::new(cloned));
                        let d = ne.borrow_mut().insert(x - quarter, y + quarter, value);
                        self.population.set(self.population.get() + d);
                        d
                    }
                    (true, false) => {
                        let cloned = sw.borrow().clone();
                        *sw = Rc::new(RefCell::new(cloned));
                        let d = sw.borrow_mut().insert(x + quarter, y - quarter, value);
                        self.population.set(self.population.get() + d);
                        d
                    }
                    (false, false) => {
                        let cloned = se.borrow().clone();
                        *se = Rc::new(RefCell::new(cloned));
                        let d = se.borrow_mut().insert(x - quarter, y - quarter, value);
                        self.population.set(self.population.get() + d);
                        d
                    }
                }
            }
            _ => panic!(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self.node, NodeKind::Leaf(_))
    }

    pub fn get_pseudo_child(&self, x: i32, y: i32) -> Self {
        if self.level <= LEAF_LEVEL {
            panic!();
        }

        let mut new_node = Self::new(self.level - 1);
        if new_node.level > LEAF_LEVEL {
            new_node.subdivide();
        }

        for yy in [-1, 1] {
            for xx in [-1, 1] {
                let child = self
                    .get_child(if x == 0 { xx } else { x }, if y == 0 { yy } else { y })
                    .borrow();

                if child.level == LEAF_LEVEL {
                    let v = match child.node {
                        NodeKind::Leaf(v) => v,
                        _ => panic!(),
                    };
                    // HACK: !!!!!
                    let (i, j) = ((yy - 1) / 2, (xx - 1) / 2);
                    let (ii, jj) = (
                        (if y == 0 { -yy } else { yy } + 1) / 2,
                        (if x == 0 { -xx } else { xx } + 1) / 2,
                    );
                    new_node.insert(j, i, v[ii as usize][jj as usize]);
                } else if child.is_leaf() {
                    *new_node.get_child_mut(xx, yy) =
                        Rc::new(RefCell::new(Node::new(self.level - 2)));
                } else {
                    let child = child
                        .get_child(if x == 0 { -xx } else { xx }, if y == 0 { -yy } else { yy });
                    *new_node.get_child_mut(xx, yy) = Rc::clone(child);
                    new_node
                        .population
                        .set(new_node.population.get() + child.borrow().population.get());
                };
            }
        }

        new_node
    }

    pub fn get_mut_with_center(&mut self, x: i32, y: i32) -> &mut Rc<RefCell<Self>> {
        if x == 0 && y == 0 {
            panic!();
        }
        let mut l = 1 << (self.level - 1);

        let mut x = x;
        let mut y = y;

        let ret;

        unsafe {
            self.subdivide();
            let mut node: *mut Rc<RefCell<Self>> = self.get_child_mut(x, y);
            x = x.rem_euclid(l);
            y = y.rem_euclid(l);
            l >>= 1;
            x -= l;
            y -= l;

            while x != 0 && y != 0 {
                (*node).borrow_mut().subdivide();
                node = (*node).borrow_mut().get_child_mut(x, y);
                x = x.rem_euclid(l);
                y = y.rem_euclid(l);
                l >>= 1;
                x -= l;
                y -= l;
            }
            ret = &mut *node;
        }
        ret
    }

    pub fn grown(&self) -> Node {
        let mut new_node = Self::new(self.level + 1);
        let quarter = 1 << (self.level - 2);
        if let NodeKind::Branch { nw, ne, sw, se } = &self.node {
            *new_node.get_mut_with_center(-quarter, -quarter) = Rc::clone(nw);
            *new_node.get_mut_with_center(quarter, -quarter) = Rc::clone(ne);
            *new_node.get_mut_with_center(-quarter, quarter) = Rc::clone(sw);
            *new_node.get_mut_with_center(quarter, quarter) = Rc::clone(se);
        }
        new_node.level = self.level + 1;
        new_node
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.memo_hash.get().is_none() {
            let mut new_state = FxHasher::default();
            match &self.node {
                NodeKind::Leaf(v) => {
                    v.hash(&mut new_state);
                    self.level.hash(&mut new_state);
                }
                NodeKind::Branch { nw, ne, sw, se } => {
                    nw.borrow().hash(&mut new_state);
                    ne.borrow().hash(&mut new_state);
                    sw.borrow().hash(&mut new_state);
                    se.borrow().hash(&mut new_state);
                }
            }
            self.memo_hash.replace(Some(new_state.finish()));
        }
        // HACK: hashing the hash
        self.memo_hash.get().unwrap().hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if self.level != other.level {
            return false;
        }

        let mut state = FxHasher::default();
        let mut other_state = FxHasher::default();
        self.hash(&mut state);
        other.hash(&mut other_state);
        state.finish() == other_state.finish() // HACK: ICKY
    }
}
impl Eq for Node {}

#[derive(Clone)]
pub struct Universe {
    pub cache: RefCell<FxHashMap<(Node, i32), Node>>,
    pub root: Rc<RefCell<Node>>,
    pub generation: u64,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(FxHashMap::default()),
            root: Rc::new(RefCell::new(Node::new(16))),
            generation: 0,
        }
    }

    pub fn neighbor_count(&self, node: &Node, x: i32, y: i32) -> u8 {
        let mut count = 0;
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue;
                }
                let xx = x + j;
                let yy = y + i;
                count += node.get(xx, yy);
            }
        }
        count
    }

    pub fn steppa(&mut self, generations: i32) {
        let generations = generations.min(self.root.borrow().level as i32 - 2);
        let next = Rc::new(RefCell::new(
            self.step(&self.root.borrow().grown(), generations),
        ));
        self.root = next;
    }

    pub fn step(&self, node: &Node, generations: i32) -> Node {
        let mut state = self.cache.borrow().hasher().build_hasher();
        node.hash(&mut state);
        generations.hash(&mut state);
        let hash = state.finish();

        if let Some((_, n)) = self
            .cache
            .borrow()
            .raw_entry()
            .from_hash(hash, |(n, g)| n == node && *g == generations)
        {
            return n.clone();
        }

        let new_node = if node.level == LEAF_LEVEL + 1 {
            let mut new_node = Node::new(LEAF_LEVEL);
            for i in -1..1 {
                for j in -1..1 {
                    let v = match self.neighbor_count(&node, j, i) {
                        2 => node.get(j, i),
                        3 => 1,
                        _ => 0,
                    };

                    new_node.insert(j, i, v);
                }
            }
            new_node
        } else {
            let mut quads = vec![];
            for y in -1..=1 {
                for x in -1..=1 {
                    let pseudo_child = node.get_pseudo_child(x, y);
                    quads.push(Rc::new(RefCell::new(self.step(&pseudo_child, generations))));
                }
            }

            // WARN: ugly code
            let mut nw = Node::new_branch(&quads[0], &quads[1], &quads[3], &quads[4]);
            let mut ne = Node::new_branch(&quads[1], &quads[2], &quads[4], &quads[5]);
            let mut sw = Node::new_branch(&quads[3], &quads[4], &quads[6], &quads[7]);
            let mut se = Node::new_branch(&quads[4], &quads[5], &quads[7], &quads[8]);
            if generations + 2 >= node.level as i32 {
                nw = self.step(&nw, generations);
                ne = self.step(&ne, generations);
                sw = self.step(&sw, generations);
                se = self.step(&se, generations);
            } else {
                nw = nw.get_pseudo_child(0, 0);
                ne = ne.get_pseudo_child(0, 0);
                sw = sw.get_pseudo_child(0, 0);
                se = se.get_pseudo_child(0, 0);
            }

            Node::new_branch(
                &Rc::new(RefCell::new(nw)),
                &Rc::new(RefCell::new(ne)),
                &Rc::new(RefCell::new(sw)),
                &Rc::new(RefCell::new(se)),
            )
        };

        self.cache
            .borrow_mut()
            .raw_entry_mut()
            .from_hash(hash, |(n, g)| n == node && *g == generations)
            .or_insert((node.clone(), generations), new_node.clone());

        new_node
    }
}
