use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

// FIXME: make it bigger like 2^10
const INITIAL_NBUCKETS: usize = 1;

fn compute_hash<T: Hash + ?Sized>(value: &T, len: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    (hasher.finish() % len as u64) as usize
}

type Bucket<K, V> = Vec<(K, V)>;

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
        self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4
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
        let index = compute_hash(key, self.buckets.len());
        let bucket = &self.buckets[index];
        bucket
            .iter()
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
        let index = compute_hash(&key, self.buckets.len());
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
        new_buckets.resize_with(target_size, || Vec::new());

        // drain removes the range from buckets
        // flatten flattens the nested vector structure so it yields (K, V) pairs instead of Bucket
        // this is the same as doing flat_map(|bucket| bucket.into_iter())
        for (key, value) in self.buckets.drain(..).flatten() {
            // recompute hash for new size
            let index = compute_hash(&key, new_buckets.len());
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
        let index = compute_hash(key, self.buckets.len());
        let bucket = &mut self.buckets[index];
        match bucket.iter().position(|(ekey, _)| ekey.borrow() == key) {
            Some(index) => Some(bucket.swap_remove(index).1),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let key = "key";
        let mut map = HashMap::new();
        map.insert(key, 42);
        assert_eq!(map.get("key"), Some(&42));
        map.insert(key, 69);
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
        assert_eq!(map.remove("key"), Some(42));
        assert_eq!(map.remove("key"), None);
    }
}
