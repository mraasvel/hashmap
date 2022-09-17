use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::Index;

// FIXME: make it bigger like 2^10
const INITIAL_NBUCKETS: usize = 1;

#[inline]
fn compute_hash<T: Hash + ?Sized>(value: &T, len: usize) -> Option<usize> {
    match len {
        0 => None,
        len => {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            Some((hasher.finish() % len as u64) as usize)
        }
    }
}

#[inline]
fn compute_hash_unchecked<T: Hash + ?Sized>(value: &T, len: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    (hasher.finish() % len as u64) as usize
}

type Bucket<K, V> = Vec<(K, V)>;

#[derive(Default)]
pub struct HashMap<K, V> {
    buckets: Vec<Bucket<K, V>>,
    items: usize,
}   

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }

    fn should_resize(&self) -> bool {
        // empty or 3 quarters full (meaning load_factor == 0.75)
        self.is_empty() || self.items > 3 * self.buckets.len() / 4
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        IntoIterator::into_iter(self)
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    // The trait bounds for Q in this case allows for &str to be accepted when the key is a String.
    // This means that the user doesn't have to own the referenced value.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = compute_hash(key, self.buckets.len())?;
        let bucket = &self.buckets[index];
        bucket
            .iter()
            .find(|(ekey, _)| ekey.borrow() == key)
            .map(|(_, value)| value)
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = compute_hash(key, self.buckets.len())?;
        let bucket = &mut self.buckets[index];
        bucket
            .iter_mut()
            .find(|(ekey, _)| ekey.borrow() == key)
            .map(|(_, value)| value)
    }

    /// Inserts value under key into the hashmap.
    /// Returns the old value of that key if it was present.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // assert that we have enough space
        if self.should_resize() {
            self.resize();
        }

        // find the bucket this key belongs to
        let index = compute_hash_unchecked(&key, self.buckets.len());
        let bucket = &mut self.buckets[index];

        // if present (linear search), replace the value and return it
        if let Some((_, evalue)) = bucket.iter_mut().find(|(ekey, _)| ekey == &key) {
            return Some(mem::replace(evalue, value));
        }
        bucket.push((key, value));
        self.items += 1;
        None
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };

        // new vector with new size
        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.resize_with(target_size, Vec::new);

        // drain removes the range from buckets
        // flatten flattens the nested vector structure so it yields (K, V) pairs instead of Bucket
        // this is the same as doing flat_map(|bucket| bucket.into_iter())
        for (key, value) in self.buckets.drain(..).flatten() {
            // recompute hash for new size
            let index = compute_hash_unchecked(&key, new_buckets.len());
            // because everything is moved the copies are relatively efficient
            new_buckets[index].push((key, value));
        }

        // moving the new buckets is simply a copy of the pointers
        self.buckets = new_buckets;
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let index = compute_hash(key, self.buckets.len())?;
        let bucket = &mut self.buckets[index];
        // In a multimap you might want to use retain.
        // Single instance of the key we can stop at the index and use swap_remove for constant removal.
        match bucket.iter().position(|(ekey, _)| ekey.borrow() == key) {
            Some(index) => {
                self.items -= 1;
                Some(bucket.swap_remove(index).1)
            }
            None => None,
        }
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get(key).is_some()
    }
}

impl <K, V, Q> Index<&Q> for HashMap<K, V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: ?Sized + Hash + Eq,
{
    type Output = V;
    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).unwrap()
    }
}

/**
 * Implement IntoIterator
 */

/**
 * IntoIter
 */

pub struct IntoIter<K, V> {
    hashmap: HashMap<K, V>,
    bucket: usize,
}

impl <K, V> IntoIter<K, V> {
    fn new(hashmap: HashMap<K, V>) -> Self {
        IntoIter {
            hashmap,
            bucket: 0,
        }
    }
}

impl <K, V> IntoIterator for HashMap<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl <K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket < self.hashmap.buckets.len() && self.hashmap.buckets[self.bucket].is_empty() {
            self.bucket += 1;
        }
        if self.bucket == self.hashmap.buckets.len() {
            return None;
        }
        self.hashmap.buckets[self.bucket].pop()
    }
}

/**
 * Iter
 */

pub struct Iter<'hashmap, K, V> {
    hashmap: &'hashmap HashMap<K, V>,
    bucket: usize,
    index: usize,
}

impl <'hashmap, K, V> Iter<'hashmap, K, V> {
    fn new(hashmap: &'hashmap HashMap<K, V>) -> Self {
        Iter {
            hashmap,
            bucket: 0,
            index: 0,
        }
    }
}

impl <'hashmap, K, V> IntoIterator for &'hashmap HashMap<K, V> {
    type IntoIter = Iter<'hashmap, K, V>;
    type Item = &'hashmap (K, V);
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl <'hashmap, K, V> Iterator for Iter<'hashmap, K, V> {
    type Item = &'hashmap (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket < self.hashmap.buckets.len() {
            let bucket = &self.hashmap.buckets[self.bucket];
            if self.index >= bucket.len() {
                self.index = 0;
            } else {
                self.index += 1;
                return Some(&bucket[self.index - 1]);
            }
            self.bucket += 1;
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let key = "key";
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        map.insert(key, 42);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("key"), Some(&42));
        map.insert(key, 69);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("key"), Some(&69));
        assert_eq!(map.get("value"), None);
    }

    #[test]
    fn test_string() {
        let mut map = HashMap::new();
        map.insert("string".to_string(), 42);
        assert_eq!(map.get("string"), Some(&42));
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), 42);
        assert_eq!(map.get("key"), Some(&42));
        assert_eq!(map.len(), 1);
        assert_eq!(map.remove("key"), Some(42));
        assert_eq!(map.len(), 0);
        assert_eq!(map.get("key"), None);
        assert_eq!(map.remove("key"), None);
    }

    #[test]
    fn test_contains_key() {
        let mut map = HashMap::new();
        assert_eq!(map.contains_key("key"), false);
        map.insert("key".to_string(), Some(42));
        assert_eq!(map.contains_key("key"), true);
    }

    #[test]
    fn test_index() {
        let mut map = HashMap::new();
        map.insert("key", 42);
        assert_eq!(map["key"], 42);
    }

    #[test]
    #[should_panic]
    fn test_invalid_index() {
        let map: HashMap<String, i32> = HashMap::new();
        let _ = map["key"];
    }

    #[test]
    fn test_into_iter() {
        let mut map = HashMap::new();

        let v1 = vec![
            ('a', 42),
            ('b', 11),
            ('c', 422),
            ('d', 3),
        ];
        for (key, value) in v1.iter() {
            map.insert(key.clone(), value.clone());
        }
        let mut v2: Vec<_> = map.into_iter().collect();
        v2.sort();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_iter() {
        let mut map = HashMap::new();

        let v1 = vec![
            ('a', 42),
            ('b', 11),
            ('c', 422),
            ('d', 3),
        ];
        for (key, value) in v1.iter() {
            map.insert(key.clone(), value.clone());
        }
        let mut v2: Vec<_> = map.iter().cloned().collect();
        v2.sort();
        assert_eq!(v1, v2);
    }
}
