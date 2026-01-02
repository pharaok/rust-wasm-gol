use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

type Size = u32;

pub struct Arena<T, K>
where
    K: Eq + Hash + From<T>,
    T: Clone,
{
    vec: Vec<T>,
    cache: HashMap<K, Size>,
    size: Size,
}

impl<T, K> Arena<T, K>
where
    K: Eq + Hash + From<T>,
    T: Clone,
{
    pub fn new(size: usize) -> Self {
        Self {
            vec: Vec::with_capacity(size),
            cache: HashMap::new(), // TODO: should probably be with_capacity
            size: 0,
        }
    }
    pub fn insert(&mut self, value: T) -> Size {
        let key: K = value.clone().into();
        if let Some(&index) = self.cache.get(&key) {
            return index;
        }
        self.vec.push(value);
        self.cache.insert(key, self.size);
        self.size += 1;
        self.size - 1
    }
    pub fn get(&self, index: Size) -> &T {
        self.vec.get(index as usize).unwrap()
    }
}
