# Todo
- impl IntoIterator (self, &self (iter()), &mut self (into_iter()))
- clean up implementation block

# Hashmap
- take a hasher that returns some u64 index value
- multiple keys can map to the same index (hash collision)
- array of linked lists (linked hashmap)

# Writing a library
1. start with an example of how the user would use it.
2. implement the rest of the library.

# General
- Prefer putting trait bounds on functions instead of on the type.

- Hashers are cummulative.
Calling `finish(&self)` on a hasher does not reset its internal state.
This means we need to create a new hasher if we want a fresh hash value.
Usually you would have a buildhasher with a particular seed from which the hash values grow.
This is done to prevent different hashmaps from having the same hashes which can be abused in attacks.

- Least strict trait bounds
```Rust
// if K == String, the user has to own the key and provide a &String type even though &str would be good enough.
fn get(key: &K) -> Option<&V>;

// Now k.borrow() results in a &Q, so if K is a String it can give a &str type.
fn get<Q: ?Sized>(key: &Q) -> Option<&V>
where
	K: Borrow<Q>,
	Q: Hash + Eq,
{}
```
