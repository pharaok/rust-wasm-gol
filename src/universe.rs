use crate::{
    arena::Arena,
    quadtree::{Branch, Leaf, Node, NodeKind, NodeRef, LEAF_LEVEL, LEAF_SIZE},
};
use rustc_hash::FxHashMap;

type Key = (NodeKind, i32); // (node, generations)

pub struct Universe {
    pub arena: Arena<Node, NodeKind>,
    pub cache: FxHashMap<Key, (NodeRef, u64)>,
    pub empty_ref: Vec<NodeRef>,
    pub root: NodeRef,
    pub generation: u64,
    pub step: i32,
}

pub const ARENA_SIZE: usize = 1 << 10;
impl Default for Universe {
    fn default() -> Self {
        Self::with_size(16)
    }
}
impl Universe {
    pub fn with_size_and_arena_capacity(size: u8, capacity: usize) -> Self {
        let mut arena = Arena::new(capacity);

        let mut empty_ref = vec![0; 256];
        let node = Node::new_empty_leaf();
        empty_ref[LEAF_LEVEL as usize] = arena.insert(node);
        for level in (LEAF_LEVEL as i32 + 1)..256i32 {
            let node = Node::new_branch([empty_ref[(level - 1) as usize]; 4], level as u8, 0);
            empty_ref[level as usize] = arena.insert(node);
        }
        let root = empty_ref[size as usize];
        let mut cache = FxHashMap::default();
        cache.reserve(capacity);
        Self {
            arena,
            cache,
            empty_ref,
            root,
            generation: 0,
            step: 0,
        }
    }
    pub fn with_size(size: u8) -> Self {
        Self::with_size_and_arena_capacity(size, ARENA_SIZE)
    }

    pub fn new_node(&self, level: u8) -> Node {
        if level == LEAF_LEVEL {
            Node::new_empty_leaf()
        } else if level > LEAF_LEVEL {
            Node {
                data: NodeKind::new_branch([self.empty_ref[(level - 1) as usize]; 4]),
                level,
                population: 0,
            }
        } else {
            panic!()
        }
    }

    pub fn get_population(&self) -> u64 {
        self.arena.get(self.root).population
    }
    pub fn get_level(&self) -> u8 {
        self.arena.get(self.root).level
    }

    pub fn grown(&mut self, node_ref: NodeRef) -> NodeRef {
        // returns NodeRef to node of level node.level + 1
        // with node in its center

        let node = *self.arena.get(node_ref);
        if node.population == 0 {
            return self.empty_ref[(node.level + 1) as usize];
        }

        let mut children = Branch::default();
        let mut pop = 0;

        let empty_grandchild = self.empty_ref[((node.level) - 1) as usize];
        for i in 0..4 {
            let child_ref = node.data.as_branch()[i];
            let child = self.arena.get(child_ref);

            let mut grandchildren: Branch = [empty_grandchild; 4];
            grandchildren[3 - i] = child_ref;
            let new_child = Node::new_branch(grandchildren, node.level, child.population);
            children[i] = self.arena.insert(new_child);
            pop += new_child.population;
        }
        let new_node = Node::new_branch(children, node.level + 1, pop);
        self.arena.insert(new_node)
    }
    pub fn shrunk(&mut self, node_ref: NodeRef) -> (NodeRef, u64) {
        let node = *self.arena.get(node_ref);
        if node.is_leaf() {
            panic!();
        }
        let l = node.level;
        if node.population == 0 {
            return (self.empty_ref[(l - 1) as usize], 0);
        }

        let new_node = if l == LEAF_LEVEL + 1 {
            let mut data = Leaf::default();
            let h = (LEAF_SIZE / 2) as i32;
            let mut pop = 0;
            for (y, row) in data.iter_mut().enumerate() {
                for (x, cell) in row.iter_mut().enumerate() {
                    *cell = self._get(x as i32 - h, y as i32 - h, node_ref);
                    pop += *cell as u64;
                }
            }
            Node::new_leaf(data, pop)
        } else {
            let mut children = Branch::default();
            let mut pop = 0;
            for (i, c) in children.iter_mut().enumerate() {
                let child_ref = node.data.as_branch()[i];
                let child = self.arena.get(child_ref);

                let grandchild_ref = child.data.as_branch()[3 - i];
                let grandchild = self.arena.get(grandchild_ref);

                *c = grandchild_ref;
                pop += grandchild.population;
            }
            Node::new_branch(children, node.level - 1, pop)
        };

        (self.arena.insert(new_node), new_node.population)
    }

    pub fn step(&mut self) {
        let step = self.step.min((self.arena.get(self.root).level - 2) as i32);
        let root_ref = self.grown(self.root);
        let next = self.step_node(root_ref, self.step).0;
        self.generation += (1 << step) as u64;
        self.root = next;
    }

    fn step_node(&mut self, node_ref: NodeRef, mut step: i32) -> (NodeRef, u64) {
        let node = self.arena.get(node_ref);
        let level = node.level;
        step = step.min(node.level as i32 - 2);

        if node.population < 3 {
            return (self.empty_ref[(level - 1) as usize], 0);
        }
        let key: Key = (node.data, step);
        if let Some(&n) = self.cache.get(&key) {
            return n;
        }

        let (new_node, population) = if level == LEAF_LEVEL + 1 {
            const SIZE: usize = 1 << (LEAF_LEVEL + 1);
            // pad by 1 cell
            let mut data = [[0; SIZE + 2]; SIZE + 2];
            for ci in 0..2 {
                for cj in 0..2 {
                    let child_data = self
                        .arena
                        .get(node.data.as_branch()[2 * ci + cj])
                        .data
                        .as_leaf();
                    for i in 0..LEAF_SIZE {
                        for j in 0..LEAF_SIZE {
                            data[1 + ci * LEAF_SIZE + i][1 + cj * LEAF_SIZE + j] = child_data[i][j];
                        }
                    }
                }
            }

            // advance min(step, 2^(LEAF_LEVEL - 1)) steps
            let mut next_data = data;
            for _ in 0..(1 << step.min((LEAF_LEVEL - 1) as i32)) {
                for i in 1..(1 + SIZE) {
                    for j in 1..(1 + SIZE) {
                        // TODO: bitboard?
                        let neighbors = data[i - 1][j - 1]
                            + data[i - 1][j]
                            + data[i - 1][j + 1]
                            + data[i][j - 1]
                            + data[i][j + 1]
                            + data[i + 1][j - 1]
                            + data[i + 1][j]
                            + data[i + 1][j + 1];

                        next_data[i][j] = if neighbors == 2 {
                            data[i][j]
                        } else if neighbors == 3 {
                            1
                        } else {
                            0
                        };
                    }
                }

                std::mem::swap(&mut data, &mut next_data);
            }

            let mut leaf_data: Leaf = Leaf::default();
            let mut pop = 0;
            for i in 0..LEAF_SIZE {
                for j in 0..LEAF_SIZE {
                    let shift = 1 + LEAF_SIZE / 2;
                    leaf_data[i][j] = data[shift + i][shift + j];
                    pop += leaf_data[i][j] as u64;
                }
            }
            let new_node = Node::new_leaf(leaf_data, pop);
            (self.arena.insert(new_node), new_node.population)
        } else {
            // everything is in row major order
            let mut sub_16 = [0; 16];
            let mut sub_16_pop = [0; 16];
            for i1 in 0..2 {
                for j1 in 0..2 {
                    let child = self.arena.get(node.data.as_branch()[2 * i1 + j1]);
                    for i2 in 0..2 {
                        for j2 in 0..2 {
                            let k = 8 * i1 + 4 * i2 + 2 * j1 + j2;
                            let gc_ref = child.data.as_branch()[2 * i2 + j2];
                            sub_16[k] = gc_ref;
                            let grandchild = self.arena.get(gc_ref);
                            sub_16_pop[k] = grandchild.population;
                        }
                    }
                }
            }

            let mut sub_9 = [0; 9];
            let mut sub_9_pop = [0; 9];
            for i in 0..3 {
                for j in 0..3 {
                    let k = 4 * i + j;
                    sub_9[3 * i + j] = self.arena.insert(Node::new_branch(
                        [sub_16[k], sub_16[k + 1], sub_16[k + 4], sub_16[k + 5]],
                        level - 1,
                        sub_16_pop[k] + sub_16_pop[k + 1] + sub_16_pop[k + 4] + sub_16_pop[k + 5],
                    ));
                    (sub_9[3 * i + j], sub_9_pop[3 * i + j]) =
                        self.step_node(sub_9[3 * i + j], step);
                }
            }

            let mut sub_4 = [0; 4];
            for i in 0..2 {
                for j in 0..2 {
                    let k = 3 * i + j;
                    sub_4[2 * i + j] = self.arena.insert(Node::new_branch(
                        [sub_9[k], sub_9[k + 1], sub_9[k + 3], sub_9[k + 4]],
                        level - 1,
                        sub_9_pop[k] + sub_9_pop[k + 1] + sub_9_pop[k + 3] + sub_9_pop[k + 4],
                    ));
                }
            }

            let mut pop = 0;
            if step + 2 >= level as i32 {
                for sub in &mut sub_4 {
                    let (stepped, p) = self.step_node(*sub, step);
                    *sub = stepped;
                    pop += p;
                }
            } else {
                for sub in &mut sub_4 {
                    let (shrunk, p) = self.shrunk(*sub);
                    *sub = shrunk;
                    pop += p;
                }
            }

            let new_node = Node::new_branch(sub_4, level - 1, pop);
            (self.arena.insert(new_node), new_node.population)
        };

        self.cache.insert(key, (new_node, population));
        (new_node, population)
    }

    pub fn normalize_coords(x: i32, y: i32, level: u8) -> (i32, i32) {
        let half = 1 << (level - 1);
        (x.rem_euclid(2 * half) - half, y.rem_euclid(2 * half) - half)
    }

    pub fn clear(&mut self) {
        self.root = self.empty_ref[self.get_level() as usize];
    }
    fn _get(&self, x: i32, y: i32, node_ref: NodeRef) -> u8 {
        let node = self.arena.get(node_ref);
        let half = 1 << (node.level - 1);
        if !(-half <= x && x < half && -half <= y && y < half) {
            // TODO: out of bounds
            return 0;
        }

        if node.is_leaf() {
            node.data.as_leaf()[(y + half) as usize][(x + half) as usize]
        } else {
            let (cx, cy) = Self::normalize_coords(x, y, node.level - 1);
            self._get(cx, cy, node.get_child(x, y))
        }
    }
    pub fn get(&self, x: i32, y: i32) -> u8 {
        self._get(x, y, self.root)
    }
    fn _set(&mut self, x: i32, y: i32, value: u8, node_ref: NodeRef) -> (NodeRef, i64) {
        let mut node = *self.arena.get(node_ref);
        let mut dpop = 0;
        if node.is_leaf() {
            let s = (LEAF_SIZE / 2) as i32;
            let data = node.data.as_leaf_mut();
            dpop -= data[(y + s) as usize][(x + s) as usize] as i64;
            data[(y + s) as usize][(x + s) as usize] = value;
            dpop += data[(y + s) as usize][(x + s) as usize] as i64;
        } else {
            let (cx, cy) = Self::normalize_coords(x, y, node.level - 1);
            let [nw, ne, sw, se] = *node.data.as_branch();
            let (child, d) = match (x < 0, y < 0) {
                (true, true) => self._set(cx, cy, value, nw),
                (false, true) => self._set(cx, cy, value, ne),
                (true, false) => self._set(cx, cy, value, sw),
                (false, false) => self._set(cx, cy, value, se),
            };
            *node.get_child_mut(x, y) = child;
            dpop = d;
        }
        node.population = (node.population as i64 + dpop) as u64;
        (self.arena.insert(node), dpop)
    }
    pub fn set(&mut self, x: i32, y: i32, value: u8) {
        self.root = self._set(x, y, value, self.root).0;
        self.generation = 0
    }

    pub fn get_rect(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<Vec<u8>> {
        let half = 1 << (self.get_level() - 1);
        let (x1, y1, x2, y2) = (x1.max(-half), y1.max(-half), x2.min(half), y2.min(half));

        let (w, h) = ((x2 - x1) as usize, (y2 - y1) as usize);
        let mut grid = vec![vec![0; w]; h];

        self._get_rect(x1, y1, x2, y2, &mut grid, self.root);

        grid
    }
    fn _get_rect(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        grid: &mut Vec<Vec<u8>>,
        node_ref: NodeRef,
    ) {
        let node = self.arena.get(node_ref);
        let half = 1 << (node.level - 1);
        if x1 >= half || y1 >= half || x2 < -half || y2 < -half || node.population == 0 {
            return;
        }
        let (w, h) = ((x2 - x1) as usize, (y2 - y1) as usize);

        match node.data {
            NodeKind::Leaf(v) => {
                if node.level != LEAF_LEVEL {
                    return;
                }
                for (i, row) in v.iter().enumerate() {
                    for (j, cell) in row.iter().enumerate() {
                        let (x, y) = ((j as i32 - x1 - 1) as usize, (i as i32 - y1 - 1) as usize);
                        if x < w && y < h {
                            grid[y][x] = *cell;
                        }
                    }
                }
            }
            NodeKind::Branch([nw, ne, sw, se]) => {
                let q = 1 << (node.level - 2);
                self._get_rect(x1 + q, y1 + q, x2 + q, y2 + q, grid, nw);
                self._get_rect(x1 - q, y1 + q, x2 - q, y2 + q, grid, ne);
                self._get_rect(x1 + q, y1 - q, x2 + q, y2 - q, grid, sw);
                self._get_rect(x1 - q, y1 - q, x2 - q, y2 - q, grid, se);
            }
        }
    }
    pub fn set_rect(&mut self, x: i32, y: i32, grid: &Vec<Vec<u8>>) {
        self.root = self._set_rect(x, y, grid, self.root).0;
    }
    fn _set_rect(
        &mut self,
        x: i32,
        y: i32,
        grid: &Vec<Vec<u8>>,
        node_ref: NodeRef,
    ) -> (NodeRef, u64) {
        let node = *self.arena.get(node_ref);
        let half = 1 << (node.level - 1);
        let (w, h) = (grid[0].len(), grid.len());

        if x >= half || y >= half || x + (w as i32) < -half || y + (h as i32) < -half {
            return (node_ref, 0);
        }

        if let NodeKind::Leaf(v) = node.data {
            let mut new_data = v;
            let mut pop = 0;
            for (i, row) in new_data.iter_mut().enumerate() {
                for (j, cell) in row.iter_mut().enumerate() {
                    let (x, y) = ((j as i32 - x - 1) as usize, (i as i32 - y - 1) as usize);
                    if x < w && y < h {
                        *cell = grid[y][x];
                    }
                    pop += *cell as u64;
                }
            }
            let new_node = Node::new_leaf(new_data, pop);
            return (self.arena.insert(new_node), new_node.population);
        }
        let [nw, ne, sw, se] = node.data.as_branch();
        let q = 1 << (node.level - 2);

        let (new_nw, nw_pop) = self._set_rect(x + q, y + q, grid, *nw);
        let (new_ne, ne_pop) = self._set_rect(x - q, y + q, grid, *ne);
        let (new_sw, sw_pop) = self._set_rect(x + q, y - q, grid, *sw);
        let (new_se, se_pop) = self._set_rect(x - q, y - q, grid, *se);
        let new_node = Node::new_branch(
            [new_nw, new_ne, new_sw, new_se],
            node.level,
            nw_pop + ne_pop + sw_pop + se_pop,
        );
        (self.arena.insert(new_node), new_node.population)
    }
}
