use rustc_hash::FxHashMap;
use std::cmp::Eq;
use std::hash::Hash;

type Size = u32;

pub struct Arena<T, K>
where
    K: Eq + Hash + From<T>,
    T: Clone,
{
    vec: Vec<T>,
    cache: FxHashMap<K, Size>,
    size: Size,
}

impl<T, K> Arena<T, K>
where
    K: Eq + Hash + From<T>,
    T: Clone,
{
    pub fn new(size: usize) -> Self {
        let mut cache = FxHashMap::default();
        cache.reserve(size);
        Self {
            vec: Vec::with_capacity(size),
            cache,
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
