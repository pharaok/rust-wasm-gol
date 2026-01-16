use std::collections::VecDeque;

use crate::{
    arena::Arena,
    quadtree::{Branch, LEAF_LEVEL, LEAF_SIZE, Leaf, Node, NodeKind, NodeRef},
};
use rustc_hash::FxHashMap;

pub fn step_grid(grid: &[Vec<u8>], res: &mut [Vec<u8>]) {
    for i in 1..(grid.len() - 1) {
        for j in 1..(grid[0].len() - 1) {
            let neighbors = grid[i - 1][j - 1]
                + grid[i - 1][j]
                + grid[i - 1][j + 1]
                + grid[i][j - 1]
                + grid[i][j + 1]
                + grid[i + 1][j - 1]
                + grid[i + 1][j]
                + grid[i + 1][j + 1];

            res[i][j] = if neighbors == 2 {
                grid[i][j]
            } else if neighbors == 3 {
                1
            } else {
                0
            };
        }
    }
}

type Key = (NodeKind, i32); // (node, generations)
enum Bound {
    Top,
    Left,
    Bottom,
    Right,
}

pub enum InsertMode {
    Copy,
    Or,
}
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
        Self::with_size(10)
    }
}
impl Universe {
    pub fn with_size_and_arena_capacity(size: u8, capacity: usize) -> Self {
        let mut arena = Arena::new(capacity);

        let mut empty_ref = vec![0; 256];
        let node = Node::new_empty_leaf();
        empty_ref[LEAF_LEVEL as usize] = arena.insert(node);
        for level in (LEAF_LEVEL as i32 + 1)..256 {
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

    pub fn _set_points(
        &mut self,
        points: &mut [(i64, i64)],
        mode: &InsertMode,
        curr: NodeRef,
        left: i64,
        top: i64,
    ) -> (NodeRef, u64) {
        let node = self.arena.get(curr);
        let level = node.level;
        if points.is_empty() {
            return match mode {
                InsertMode::Copy => (self.empty_ref[level as usize], 0),
                InsertMode::Or => (curr, node.population),
            };
        }
        match node.data {
            NodeKind::Leaf(mut data) => {
                let mut pop = node.population;
                match mode {
                    InsertMode::Copy => {
                        data = Leaf::default();
                        pop = 0;
                        for (x, y) in points {
                            pop += 1;
                            data[(*y - top) as usize][(*x - left) as usize] = 1;
                        }
                    }
                    InsertMode::Or => {
                        for (x, y) in points {
                            if data[(*y - top) as usize][(*x - left) as usize] == 0 {
                                pop += 1;
                            }
                            data[(*y - top) as usize][(*x - left) as usize] = 1;
                        }
                    }
                }

                (self.arena.insert(Node::new_leaf(data, pop)), pop)
            }
            NodeKind::Branch(mut children) => {
                let parts = Node::partition_points_mut(points, level, left, top);
                let mut pop = 0;
                for (i, child) in children.iter_mut().enumerate() {
                    let (ox, oy) = Node::get_child_offset(i, level);
                    let (c, c_pop) = self._set_points(parts[i], mode, *child, left + ox, top + oy);
                    *child = c;
                    pop += c_pop;
                }

                (
                    self.arena.insert(Node::new_branch(children, level, pop)),
                    pop,
                )
            }
        }
    }
    pub fn set_points(&mut self, points: &[(i64, i64)], mode: &InsertMode) {
        let q = 1i64 << (self.get_level() - 2);
        let mut points = points.to_owned();
        self.root = self
            ._set_points(&mut points, mode, self.root, -2 * q, -2 * q)
            .0;
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
    fn _get_node(&self, x: i64, y: i64, level: u8, curr: NodeRef) -> NodeRef {
        let node = self.arena.get(curr);
        if node.level <= level {
            return curr;
        }

        let (xx, yy) = Node::normalize_coords(x, y, node.level - 1);
        self._get_node(xx, yy, level, node.get_child(x, y))
    }
    pub fn get_node(&self, x: i64, y: i64, level: u8) -> NodeRef {
        self._get_node(x, y, level, self.root)
    }
    fn _set_node(
        &mut self,
        x: i64,
        y: i64,
        node_level: u8,
        node_ref: NodeRef,
        curr: NodeRef,
    ) -> (NodeRef, u64) {
        let node = *self.arena.get(curr);
        if node.level <= node_level {
            return (node_ref, self.arena.get(node_ref).population);
        }

        if let Node {
            data: NodeKind::Branch(mut children),
            level,
            mut population,
        } = node
        {
            let i = Node::get_child_index(x, y);
            let (xx, yy) = Node::normalize_coords(x, y, level - 1);
            population -= self.arena.get(children[i]).population;
            let (new_child, p) = self._set_node(xx, yy, node_level, node_ref, children[i]);
            children[i] = new_child;
            population += p;
            (
                self.arena
                    .insert(Node::new_branch(children, level, population)),
                population,
            )
        } else {
            panic!()
        }
    }
    pub fn set_node(&mut self, x: i64, y: i64, level: u8, node_ref: NodeRef) {
        self.root = self._set_node(x, y, level, node_ref, self.root).0;
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
            let h = (LEAF_SIZE / 2) as i64;
            let mut pop = 0;
            for (y, row) in data.iter_mut().enumerate() {
                for (x, cell) in row.iter_mut().enumerate() {
                    *cell = self._get(x as i64 - h, y as i64 - h, node_ref);
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
        self.generation += 1u64 << step;
        self.root = next;
    }

    fn step_node(&mut self, curr: NodeRef, mut step: i32) -> (NodeRef, u64) {
        let node = self.arena.get(curr);
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
            let mut data = vec![vec![0; SIZE + 2]; SIZE + 2];
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
            let mut next_data = data.clone();
            for _ in 0..(1 << step.min((LEAF_LEVEL - 1) as i32)) {
                step_grid(&data, &mut next_data);
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

    pub fn clear(&mut self) {
        self.root = self.empty_ref[self.get_level() as usize];
    }
    fn _get(&self, x: i64, y: i64, curr: NodeRef) -> u8 {
        let node = self.arena.get(curr);
        let half = 1i64 << (node.level - 1);
        if !(-half <= x && x < half && -half <= y && y < half) {
            return 0;
        }

        if node.is_leaf() {
            node.data.as_leaf()[(y + half) as usize][(x + half) as usize]
        } else {
            let (cx, cy) = Node::normalize_coords(x, y, node.level - 1);
            self._get(cx, cy, node.get_child(x, y))
        }
    }
    pub fn get(&self, x: i64, y: i64) -> u8 {
        self._get(x, y, self.root)
    }
    fn _set(&mut self, x: i64, y: i64, value: u8, curr: NodeRef) -> (NodeRef, i64) {
        let node = *self.arena.get(curr);
        let mut dpop = 0;

        let Node {
            mut data,
            level,
            mut population,
        } = node;
        match &mut data {
            NodeKind::Branch(children) => {
                let (cx, cy) = Node::normalize_coords(x, y, node.level - 1);
                let i = Node::get_child_index(x, y);
                let (new_child, d) = self._set(cx, cy, value, children[i]);
                children[i] = new_child;
                dpop = d;
            }
            NodeKind::Leaf(data) => {
                let s = (LEAF_SIZE / 2) as i64;
                dpop -= data[(y + s) as usize][(x + s) as usize] as i64;
                data[(y + s) as usize][(x + s) as usize] = value;
                dpop += data[(y + s) as usize][(x + s) as usize] as i64;
            }
        };
        population = (population as i64 + dpop) as u64;

        (
            self.arena.insert(Node {
                data,
                level,
                population,
            }),
            dpop,
        )
    }
    pub fn set(&mut self, x: i64, y: i64, value: u8) {
        self.root = self._set(x, y, value, self.root).0;
        self.generation = 0
    }

    fn _get_rect(
        &self,
        x1: i64,
        y1: i64,
        x2: i64,
        y2: i64,
        grid: &mut Vec<Vec<u8>>,
        curr: NodeRef,
    ) {
        let node = self.arena.get(curr);
        let half = 1i64 << (node.level - 1);
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
                        let (x, y) = ((j as i64 - x1 - 1) as usize, (i as i64 - y1 - 1) as usize);
                        if x < w && y < h {
                            grid[y][x] = *cell;
                        }
                    }
                }
            }
            NodeKind::Branch(children) => {
                for (i, child) in children.iter().enumerate() {
                    let (ox, oy) = Node::offset_to_child(i, node.level);
                    self._get_rect(x1 + ox, y1 + oy, x2 + ox, y2 + oy, grid, *child);
                }
            }
        }
    }
    pub fn get_rect(&self, x1: i64, y1: i64, x2: i64, y2: i64) -> Vec<Vec<u8>> {
        let half = 1i64 << (self.get_level() - 1);
        let (x1, y1, x2, y2) = (
            x1.clamp(-half, half - 1),
            y1.clamp(-half, half - 1),
            x2.clamp(-half, half - 1),
            y2.clamp(-half, half - 1),
        );

        let (w, h) = ((x2 - x1) as usize, (y2 - y1) as usize);
        let mut grid = vec![vec![0; w]; h];

        self._get_rect(x1, y1, x2, y2, &mut grid, self.root);

        grid
    }
    fn _set_rect(&mut self, x: i64, y: i64, grid: &Vec<Vec<u8>>, curr: NodeRef) -> (NodeRef, u64) {
        let node = *self.arena.get(curr);
        let half = 1i64 << (node.level - 1);
        let (w, h) = (grid[0].len(), grid.len());

        if x >= half || y >= half || x + (w as i64) < -half || y + (h as i64) < -half {
            return (curr, 0);
        }

        let new_node = match node.data {
            NodeKind::Leaf(mut data) => {
                let mut pop = 0;
                for (i, row) in data.iter_mut().enumerate() {
                    for (j, cell) in row.iter_mut().enumerate() {
                        let (x, y) = ((j as i64 - x - 1) as usize, (i as i64 - y - 1) as usize);
                        if x < w && y < h {
                            *cell = grid[y][x];
                        }
                        pop += *cell as u64;
                    }
                }
                Node::new_leaf(data, pop)
            }
            NodeKind::Branch(mut children) => {
                let mut pop = 0;
                for (i, child) in children.iter_mut().enumerate() {
                    let (ox, oy) = Node::offset_to_child(i, node.level);
                    let (new_child, new_pop) = self._set_rect(x + ox, y + oy, grid, *child);
                    *child = new_child;
                    pop += new_pop;
                }

                Node::new_branch(children, node.level, pop)
            }
        };
        (self.arena.insert(new_node), new_node.population)
    }
    pub fn set_rect(&mut self, x: i64, y: i64, grid: &Vec<Vec<u8>>) {
        let half = 1i64 << (self.get_level() - 1);
        let (x, y) = (x.clamp(-half, half - 1), y.clamp(-half, half - 1));
        self.root = self._set_rect(x, y, grid, self.root).0;
    }
    fn _clear_rect(&mut self, x1: i64, y1: i64, x2: i64, y2: i64, curr: NodeRef) -> (NodeRef, u64) {
        let node = *self.arena.get(curr);
        let half = 1i64 << (node.level - 1);

        if node.population == 0 {
            return (curr, 0);
        }
        if x1 >= half || y1 >= half || x2 < -half || y2 < -half {
            return (curr, node.population);
        }
        if x1 <= -half && y1 <= -half && half < x2 && half < y2 {
            return (self.empty_ref[node.level as usize], 0);
        }

        let new_node = match node.data {
            NodeKind::Leaf(mut data) => {
                let mut pop = 0;
                for (i, row) in data.iter_mut().enumerate() {
                    for (j, cell) in row.iter_mut().enumerate() {
                        let (x, y) = (j as i64 - half, i as i64 - half);
                        if x1 <= x && x <= x2 && y1 <= y && y <= y2 {
                            *cell = 0;
                        }
                        pop += *cell as u64;
                    }
                }
                Node::new_leaf(data, pop)
            }
            NodeKind::Branch(mut children) => {
                let mut pop = 0;
                for (i, child) in children.iter_mut().enumerate() {
                    let (ox, oy) = Node::offset_to_child(i, node.level);
                    let (new_child, new_pop) =
                        self._clear_rect(x1 + ox, y1 + oy, x2 + ox, y2 + oy, *child);
                    *child = new_child;
                    pop += new_pop;
                }

                Node::new_branch(children, node.level, pop)
            }
        };
        (self.arena.insert(new_node), new_node.population)
    }
    pub fn clear_rect(&mut self, x1: i64, y1: i64, x2: i64, y2: i64) {
        let half = 1i64 << (self.get_level() - 1);
        let (x1, y1, x2, y2) = (
            x1.clamp(-half, half - 1),
            y1.clamp(-half, half - 1),
            x2.clamp(-half, half - 1),
            y2.clamp(-half, half - 1),
        );
        self.root = self._clear_rect(x1, y1, x2, y2, self.root).0;
    }

    fn _get_bound(&self, bound: &Bound, curr: NodeRef, left: i64, top: i64) -> i64 {
        let node = self.arena.get(curr);
        let worst = match bound {
            Bound::Top | Bound::Left => i64::MAX,
            Bound::Bottom | Bound::Right => i64::MIN,
        };
        let mut best = worst;
        if node.population == 0 {
            return best;
        }

        let mut take = |v: i64| {
            match bound {
                Bound::Top => best = best.min(v),
                Bound::Left => best = best.min(v),
                Bound::Bottom => best = best.max(v),
                Bound::Right => best = best.max(v),
            };
        };
        match node.data {
            NodeKind::Leaf(data) => {
                for (y, row) in data.iter().enumerate() {
                    for (x, cell) in row.iter().enumerate() {
                        if *cell != 0 {
                            match bound {
                                Bound::Top => take(top + y as i64),
                                Bound::Left => take(left + x as i64),
                                Bound::Bottom => take(top + y as i64),
                                Bound::Right => take(left + x as i64),
                            };
                        }
                    }
                }
            }
            NodeKind::Branch(children) => {
                let (a, b) = match bound {
                    Bound::Top => ([0, 1], [2, 3]),
                    Bound::Left => ([0, 2], [1, 3]),
                    Bound::Bottom => ([2, 3], [0, 1]),
                    Bound::Right => ([1, 3], [0, 2]),
                };
                let mut found = false;
                for i in a {
                    let c = self.arena.get(children[i]);
                    let (dx, dy) = Node::get_child_offset(i, node.level);
                    if c.population != 0 {
                        take(self._get_bound(bound, children[i], left + dx, top + dy));
                        found = true;
                    }
                }
                if !found {
                    for i in b {
                        let (dx, dy) = Node::get_child_offset(i, node.level);
                        take(self._get_bound(bound, children[i], left + dx, top + dy))
                    }
                }
            }
        }
        best
    }
    pub fn get_bounding_rect(&self) -> (i64, i64, i64, i64) {
        let h = 1i64 << (self.get_level() - 1);
        (
            self._get_bound(&Bound::Left, self.root, -h, -h),
            self._get_bound(&Bound::Top, self.root, -h, -h),
            self._get_bound(&Bound::Right, self.root, -h, -h),
            self._get_bound(&Bound::Bottom, self.root, -h, -h),
        )
    }
}

pub struct UniverseIterator<'a> {
    universe: &'a Universe,
    stack: VecDeque<(NodeRef, i64, i64)>,
    leaf_queue: VecDeque<(i64, i64)>,
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
}

impl<'a> Iterator for UniverseIterator<'a> {
    type Item = (i64, i64);
    fn next(&mut self) -> Option<Self::Item> {
        if !self.leaf_queue.is_empty() {
            return Some(self.leaf_queue.pop_front().unwrap());
        }

        while !self.stack.is_empty() {
            let (node_ref, left, top) = self.stack.pop_back().unwrap();
            let node = self.universe.arena.get(node_ref);
            if node.population == 0 {
                continue;
            }

            match node.data {
                NodeKind::Leaf(data) => {
                    for i in 0..LEAF_SIZE * LEAF_SIZE {
                        let (y, x) = ((i / LEAF_SIZE) as i64, (i % LEAF_SIZE) as i64);
                        if self.x1 <= left + x
                            && left + x <= self.x2
                            && self.y1 <= top + y
                            && top + y <= self.y2
                            && data[y as usize][x as usize] != 0
                        {
                            self.leaf_queue.push_back((left + x, top + y));
                        }
                    }
                    if !self.leaf_queue.is_empty() {
                        return Some(self.leaf_queue.pop_front().unwrap());
                    }
                }
                NodeKind::Branch(children) => {
                    let half = 1i64 << (node.level - 1);
                    for (i, child) in children.iter().enumerate().rev() {
                        let (ox, oy) = Node::get_child_offset(i, node.level);
                        let (cx1, cy1) = (left + ox, top + oy);
                        let (cx2, cy2) = (cx1 + half, cy1 + half);
                        if !(cx2 < self.x1 || cy2 < self.y1 || cx1 > self.x2 || cy1 > self.y2) {
                            self.stack.push_back((*child, left + ox, top + oy));
                        }
                    }
                }
            }
        }
        None
    }
}
impl Universe {
    pub fn iter_alive(&self) -> UniverseIterator<'_> {
        let half = 1i64 << (self.get_level() - 1);
        self.iter_alive_in_rect(-half, -half, half - 1, half - 1)
    }
    pub fn iter_alive_in_rect(&self, x1: i64, y1: i64, x2: i64, y2: i64) -> UniverseIterator<'_> {
        let half = 1i64 << (self.get_level() - 1);
        UniverseIterator {
            universe: self,
            stack: VecDeque::from([(self.root, -half, -half)]),
            leaf_queue: VecDeque::new(),
            x1,
            y1,
            x2,
            y2,
        }
    }
}
