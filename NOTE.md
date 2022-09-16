# Hashmap
- take a hasher that returns some u64 index value
- multiple keys can map to the same index (hash collision)
- array of linked lists (linked hashmap)

# Writing a library
1. start with an example of how the user would use it.


# General
- Prefer putting trait bounds on functions instead of on the type.

- Hashers are cummulative.
Calling `finish(&self)` on a hasher does not reset its internal state.
This means we need to create a new hasher if we want a fresh hash value.
Usually you would have a buildhasher with a particular seed from which the hash values grow.
This is done to prevent different hashmaps from having the same hashes which can be abused in attacks.
