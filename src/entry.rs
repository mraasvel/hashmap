use std::hash::Hash;

use crate::HashMap;

pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V
    where
        K: Hash + Eq,
    {
        self.or_insert_with(|| default)
    }

    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce() -> V,
        K: Hash + Eq,
    {
        match self {
            Entry::Occupied(e) => &mut e.map.buckets[e.bucket][e.index].1,
            Entry::Vacant(e) => e.insert(f()),
        }
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut e) => {
                f(e.get_mut());
                Entry::Occupied(e)
            }
            Entry::Vacant(e) => Entry::Vacant(e),
        }
    }
}

pub struct OccupiedEntry<'a, K, V> {
    map: &'a mut HashMap<K, V>,
    bucket: usize,
    index: usize,
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    pub fn new(map: &'a mut HashMap<K, V>, bucket: usize, index: usize) -> Self {
        OccupiedEntry { map, bucket, index }
    }

    pub fn get(&self) -> &V {
        &self.map.buckets[self.bucket][self.index].1
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.map.buckets[self.bucket][self.index].1
    }
}

pub struct VacantEntry<'a, K, V> {
    map: &'a mut HashMap<K, V>,
    bucket: usize,
    key: K,
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    pub fn new(map: &'a mut HashMap<K, V>, bucket: usize, key: K) -> Self {
        VacantEntry { map, bucket, key }
    }

    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash + Eq,
    {
        if self.map.should_resize() {
            self.map.resize();
        }
        self.map.items += 1;
        let bucket = &mut self.map.buckets[self.bucket];
        bucket.push((self.key, value));
        &mut bucket.last_mut().unwrap().1
    }
}
