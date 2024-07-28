use std::{cell::RefCell, rc::Rc};

use rustc_hash::FxHashMap;

use crate::quadtree::{Node, LEAF_LEVEL};

type Key = (u64, i32); // (node hash, generations)

#[derive(Clone)]
pub struct Universe {
    pub cache: RefCell<FxHashMap<Key, Rc<RefCell<Node>>>>,
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
        self.generation += (1 << generations) as u64;
        let next = self.step_node(&self.root.borrow().grown(), generations);
        self.root = next;
    }

    fn step_node(&self, node: &Node, generations: i32) -> Rc<RefCell<Node>> {
        let generations = generations.min(node.level as i32 - 2);
        let node_hash = node.get_hash(self.cache.borrow().hasher());

        if node.population.get() < 3 {
            return Rc::new(RefCell::new(Node::new(node.level - 1)));
        }
        if let Some(n) = self.cache.borrow().get(&(node_hash, generations)) {
            return Rc::clone(n);
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
                    quads.push(self.step_node(&pseudo_child, generations));
                }
            }

            let mut children = [
                Node::new_branch([&quads[0], &quads[1], &quads[3], &quads[4]]),
                Node::new_branch([&quads[1], &quads[2], &quads[4], &quads[5]]),
                Node::new_branch([&quads[3], &quads[4], &quads[6], &quads[7]]),
                Node::new_branch([&quads[4], &quads[5], &quads[7], &quads[8]]),
            ]
            .map(|c| Rc::new(RefCell::new(c)));

            if generations + 2 >= node.level as i32 {
                children = children.map(|c| self.step_node(&c.borrow(), generations));
            } else {
                children =
                    children.map(|c| Rc::new(RefCell::new(c.borrow().get_pseudo_child(0, 0))));
            }
            Node::new_branch(children.each_ref())
        };
        let new_node = Rc::new(RefCell::new(new_node));

        self.cache
            .borrow_mut()
            .insert((node_hash, generations), Rc::clone(&new_node));

        new_node
    }

    pub fn insert(&mut self, x: i32, y: i32, value: u8) {
        if self.generation > 0 {
            let cloned = self.root.borrow().deep_clone();
            self.root = Rc::new(RefCell::new(cloned));
            self.generation = 0
        }
        self.root.borrow_mut().insert(x, y, value);
    }
}
