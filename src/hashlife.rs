use std::{
    cell::{Cell, RefCell},
    hash::{BuildHasher, Hash, Hasher},
    rc::Rc,
};

use rustc_hash::FxHashMap;

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
    pub population: Cell<i32>,
    pub memo_hash: Cell<Option<u64>>,
}

impl Node {
    pub fn new(level: usize) -> Self {
        Self {
            node: NodeKind::new_leaf(),
            level,
            population: Cell::new(0),
            memo_hash: Cell::new(None),
        }
    }
    pub fn new_branch(children: [&Rc<RefCell<Node>>; 4]) -> Self {
        let level = children[0].borrow().level + 1;
        let population = children.map(|c| c.borrow().population.get()).iter().sum();
        Self {
            node: NodeKind::new_branch(children),
            level,
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

            self.node = NodeKind::new_branch([
                &Rc::new(RefCell::new(nw)),
                &Rc::new(RefCell::new(ne)),
                &Rc::new(RefCell::new(sw)),
                &Rc::new(RefCell::new(se)),
            ]);
        }
    }

    pub fn child(&self, i: usize) -> &Rc<RefCell<Self>> {
        match &self.node {
            NodeKind::Branch(children) => &children[i],
            _ => panic!(),
        }
    }
    pub fn child_mut(&mut self, i: usize) -> &mut Rc<RefCell<Self>> {
        match &mut self.node {
            NodeKind::Branch(children) => &mut children[i],
            _ => panic!(),
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
            NodeKind::Branch([nw, ne, sw, se]) => match (x < 0, y < 0) {
                (true, true) => nw.borrow().get(x + quarter, y + quarter),
                (false, true) => ne.borrow().get(x - quarter, y + quarter),
                (true, false) => sw.borrow().get(x + quarter, y - quarter),
                (false, false) => se.borrow().get(x - quarter, y - quarter),
            },
        }
    }

    pub fn insert(&mut self, x: i32, y: i32, value: u8) -> i32 {
        self.memo_hash.take(); // invalidate hash

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
            NodeKind::Branch([nw, ne, sw, se]) => {
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
        if x1 >= half || y1 >= half || x2 < -half || y2 < -half || self.population.get() == 0 {
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
                self.population.set(self.population.get() + d);
                return d;
            }

            self.subdivide();
        }
        let (nw, ne, sw, se) = match &mut self.node {
            NodeKind::Branch([nw, ne, sw, se]) => (nw, ne, sw, se),
            _ => panic!(),
        };

        let q = 1 << (self.level - 2);
        let mut d = 0;

        let cloned = nw.borrow().clone();
        *nw = Rc::new(RefCell::new(cloned));
        let cloned = ne.borrow().clone();
        *ne = Rc::new(RefCell::new(cloned));
        let cloned = sw.borrow().clone();
        *sw = Rc::new(RefCell::new(cloned));
        let cloned = se.borrow().clone();
        *se = Rc::new(RefCell::new(cloned));

        d += nw.borrow_mut()._set_rect(x + q, y + q, grid);
        d += ne.borrow_mut()._set_rect(x - q, y + q, grid);
        d += sw.borrow_mut()._set_rect(x + q, y - q, grid);
        d += se.borrow_mut()._set_rect(x - q, y - q, grid);
        self.population.set(self.population.get() + d);
        d
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
                    // HACK:
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

    pub fn grown(&self) -> Node {
        let mut new_node = Self::new(self.level + 1);
        new_node.subdivide();
        new_node.population.set(self.population.get());

        for i in 0..4 {
            let child = self.child(i);
            let new_child = new_node.child(i);
            new_child.borrow_mut().subdivide();
            *new_child.borrow_mut().child_mut(3 - i) = Rc::clone(child);
            new_child
                .borrow_mut()
                .population
                .set(child.borrow().population.get());
        }

        new_node.level = self.level + 1;
        new_node
    }

    pub fn get_hash(&self, hasher: &impl BuildHasher) -> u64 {
        let mut state = hasher.build_hasher();

        if self.memo_hash.get().is_none() {
            if self.population.get() == 0 {
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
}

#[derive(Clone)]
pub struct Universe {
    pub cache: RefCell<FxHashMap<(u64, i32), Node>>,
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

    pub fn step(&mut self, generations: i32) {
        let generations = generations.min(self.root.borrow().level as i32 - 2);
        let next = self.step_node(&self.root.borrow().grown(), generations);
        self.generation += (1 << generations) as u64;
        self.root = Rc::new(RefCell::new(next));
    }

    fn step_node(&self, node: &Node, generations: i32) -> Node {
        let node_hash = node.get_hash(self.cache.borrow().hasher());

        if let Some(n) = self.cache.borrow().get(&(node_hash, generations)) {
            return n.clone();
        }

        let new_node = if node.level == LEAF_LEVEL + 1 {
            let mut new_node = Node::new(LEAF_LEVEL);
            for i in -1..1 {
                for j in -1..1 {
                    let v = match self.neighbor_count(node, j, i) {
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
                    quads.push(Rc::new(RefCell::new(
                        self.step_node(&pseudo_child, generations),
                    )));
                }
            }

            let mut children = [
                Node::new_branch([&quads[0], &quads[1], &quads[3], &quads[4]]),
                Node::new_branch([&quads[1], &quads[2], &quads[4], &quads[5]]),
                Node::new_branch([&quads[3], &quads[4], &quads[6], &quads[7]]),
                Node::new_branch([&quads[4], &quads[5], &quads[7], &quads[8]]),
            ];
            if generations + 2 >= node.level as i32 {
                children = children.map(|c| self.step_node(&c, generations));
            } else {
                children = children.map(|c| c.get_pseudo_child(0, 0));
            }
            Node::new_branch(children.map(|c| Rc::new(RefCell::new(c))).each_ref())
        };

        self.cache
            .borrow_mut()
            .insert((node_hash, generations), new_node.clone());

        new_node
    }
}
