use crate::{
    arena::Arena,
    quadtree::{Leaf, Node, NodeKind, NodeRef, LEAF_LEVEL, LEAF_SIZE},
};
use std::collections::HashMap;

type Key = (NodeKind, i32); // (node, generations)

pub struct Universe {
    pub arena: Arena<Node, NodeKind>,
    pub cache: HashMap<Key, NodeRef>,
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
        let node = Node::new_leaf();
        empty_ref[LEAF_LEVEL as usize] = arena.insert(node);
        for level in (LEAF_LEVEL as i32 + 1)..256i32 {
            let node = Node::new_branch([empty_ref[(level - 1) as usize]; 4], level as u8, 0);
            empty_ref[level as usize] = arena.insert(node);
        }
        let root = empty_ref[size as usize];
        Self {
            arena,
            cache: HashMap::with_capacity(capacity),
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
            Node::new_leaf()
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
        let node = self.arena.get(node_ref);
        let l = node.level;
        let data = node.data;
        let population = node.population;
        if population == 0 {
            return self.empty_ref[(l + 1) as usize];
        }

        let mut new_node = self.new_node(l + 1);
        new_node.population = population;

        for i in 0..4 {
            let child_ref = data.as_branch()[i];
            let child = self.arena.get(child_ref);

            let mut new_child =
                Node::new_branch([self.empty_ref[(l - 1) as usize]; 4], l, child.population);
            new_child.data.as_branch_mut()[3 - i] = child_ref;
            let new_child_ref = self.arena.insert(new_child);

            new_node.data.as_branch_mut()[i] = new_child_ref;
        }
        self.arena.insert(new_node)
    }
    pub fn shrunk(&mut self, node_ref: NodeRef) -> NodeRef {
        let node = *self.arena.get(node_ref);
        if node.is_leaf() {
            panic!();
        }
        let l = node.level;
        if node.population == 0 {
            return self.empty_ref[(l - 1) as usize];
        }

        let mut new_node = self.new_node(l - 1);

        if l == LEAF_LEVEL + 1 {
            let data = new_node.data.as_leaf_mut();
            let h = (LEAF_SIZE / 2) as i32;
            for i in 0..LEAF_SIZE {
                for j in 0..LEAF_SIZE {
                    data[i][j] = self._get(j as i32 - h, i as i32 - h, node_ref);
                    new_node.population += data[i][j] as u64;
                }
            }
        } else {
            for i in 0..4 {
                let child_ref = node.data.as_branch()[i];
                let child = self.arena.get(child_ref);

                let grandchild_ref = child.data.as_branch()[3 - i];
                let grandchild = self.arena.get(grandchild_ref);

                new_node.data.as_branch_mut()[i] = grandchild_ref;
                new_node.population += grandchild.population;
            }
        }

        self.arena.insert(new_node)
    }

    pub fn step(&mut self) {
        let step = self.step.min((self.arena.get(self.root).level - 2) as i32);
        let root_ref = self.grown(self.root);
        let next = self.step_node(root_ref, self.step);
        self.generation += (1 << step) as u64;
        self.root = next;
    }

    fn step_node(&mut self, node_ref: NodeRef, mut step: i32) -> NodeRef {
        let node = self.arena.get(node_ref);
        let level = node.level;
        step = step.min(node.level as i32 - 2);

        if node.population < 3 {
            return self.empty_ref[(level - 1) as usize];
        }
        let key: Key = (node.data, step);
        if let Some(&n) = self.cache.get(&key) {
            return n;
        }

        let new_node = if level == LEAF_LEVEL + 1 {
            const PADDED_SIZE: usize = 1 << (LEAF_LEVEL + 1);
            let mut data = [[0; PADDED_SIZE + 2]; PADDED_SIZE + 2];
            for bi in 0..2 {
                for bj in 0..2 {
                    let child = self.arena.get(node.data.as_branch()[2 * bi + bj]);
                    for i in 0..LEAF_SIZE {
                        for j in 0..LEAF_SIZE {
                            data[1 + bi * LEAF_SIZE + i][1 + bj * LEAF_SIZE + j] =
                                child.data.as_leaf()[i][j];
                        }
                    }
                }
            }

            // advance min(step, 2^(LEAF_LEVEL - 1)) steps
            let mut next_data = data;
            for _ in 0..(1 << step.min((LEAF_LEVEL - 1) as i32)) {
                for i in 1..(1 + PADDED_SIZE) {
                    for j in 1..(1 + PADDED_SIZE) {
                        let mut neighbors = 0;
                        for di in -1..=1i32 {
                            for dj in -1..=1i32 {
                                neighbors +=
                                    data[(i as i32 + di) as usize][(j as i32 + dj) as usize];
                            }
                        }
                        neighbors -= data[i][j];
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

            let mut leaf_data: Leaf = [[0u8; LEAF_SIZE]; LEAF_SIZE];
            let mut pop: u64 = 0;
            for i in 0..LEAF_SIZE {
                for j in 0..LEAF_SIZE {
                    leaf_data[i][j] = data[1 + LEAF_SIZE / 2 + i][1 + LEAF_SIZE / 2 + j];
                    pop += data[1 + LEAF_SIZE / 2 + i][1 + LEAF_SIZE / 2 + j] as u64;
                }
            }
            let leaf: NodeKind = leaf_data.into();
            let mut new_node = Node::new_leaf();
            new_node.data = leaf;
            new_node.population = pop;
            self.arena.insert(new_node)
        } else {
            let mut g = [0; 16];
            let mut pop1 = [0; 16];
            for i1 in 0..2 {
                for j1 in 0..2 {
                    let child = self.arena.get(node.data.as_branch()[2 * i1 + j1]);
                    for i2 in 0..2 {
                        for j2 in 0..2 {
                            let k = 8 * i1 + 2 * j1 + 4 * i2 + j2;
                            let gc_ref = child.data.as_branch()[2 * i2 + j2];
                            g[k] = gc_ref;
                            let grandchild = self.arena.get(gc_ref);
                            pop1[k] = grandchild.population;
                        }
                    }
                }
            }

            let mut nons = [0; 9];
            let mut pop2 = [0; 9];
            for i in 0..3 {
                for j in 0..3 {
                    let k = 4 * i + j;
                    nons[3 * i + j] = self.arena.insert(Node::new_branch(
                        [g[k], g[k + 1], g[k + 4], g[k + 5]],
                        level - 1,
                        pop1[k] + pop1[k + 1] + pop1[k + 4] + pop1[k + 5],
                    ));
                    nons[3 * i + j] = self.step_node(nons[3 * i + j], step);
                    let non = self.arena.get(nons[3 * i + j]);
                    pop2[3 * i + j] = non.population;
                }
            }

            let mut quads = [0; 4];
            for i in 0..2 {
                for j in 0..2 {
                    let k = 3 * i + j;
                    quads[2 * i + j] = self.arena.insert(Node::new_branch(
                        [nons[k], nons[k + 1], nons[k + 3], nons[k + 4]],
                        level - 1,
                        pop2[k] + pop2[k + 1] + pop2[k + 3] + pop2[k + 4],
                    ));
                }
            }
            if step + 2 >= level as i32 {
                for i in 0..4 {
                    quads[i] = self.step_node(quads[i], step);
                }
            } else {
                for i in 0..4 {
                    quads[i] = self.shrunk(quads[i]);
                }
            }

            let mut pop = 0;
            for i in 0..4 {
                let quad = self.arena.get(quads[i]);
                pop += quad.population;
            }

            let new_node = Node::new_branch(quads, level - 1, pop);
            self.arena.insert(new_node)
        };

        self.cache.insert(key, new_node);
        new_node
    }

    pub fn normalize_coords(x: i32, y: i32, level: u8) -> (i32, i32) {
        let half = 1 << (level - 1);
        (x.rem_euclid(2 * half) - half, y.rem_euclid(2 * half) - half)
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
}
