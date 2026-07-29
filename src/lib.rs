mod arena;

use std::collections::HashMap;
use std::collections::HashSet;

pub use arena::Arena;
pub use arena::Key;

/// Create a new empty `Arena`.
pub fn new<T>() -> Arena<T> {
    Arena::new()
}

/// Create a new empty `Arena` with the given capacity.
pub fn with_capacity<T>(capacity: usize) -> Arena<T> {
    Arena::with_capacity(capacity)
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::with_capacity(32)
    }
}


/// Create a new empty `HashMap`.
/// 
/// This is a map of unique keys, that does hash internally.
/// If you don't need hashing for security, use `map()` instead as it is faster.
pub fn hash_map<T>() -> HashMap<Key, T> {
    HashMap::new()
}

/// Create a new empty `HashSet`.
///
/// This is a set of unique keys, that does hash internally.
/// If you don't need hashing for security, use `set()` instead as it is faster.
pub fn hash_set<T>() -> HashSet<Key> {
    HashSet::new()
}

#[cfg(feature = "nohash")]
pub type KeySet = nohash_hasher::IntSet<Key>;
#[cfg(feature = "nohash")]
pub type ArenaMap<T> = nohash_hasher::IntMap<Key, T>;
#[cfg(feature = "nohash")]
impl nohash_hasher::IsEnabled for Key {}

/// Create a new empty `ArenaMap`.
///
/// This is a "secondary" arena that can be used to assign secondary data to keys.
/// This is a wrapper around a non-hashing map.
#[cfg(feature = "nohash")]
pub fn map<T>() -> ArenaMap<T> {
    ArenaMap::<T>::default()
}

/// Create a new empty `KeySet`.
///
/// This is a set of unique keys, that does not need to hash.
/// This is a wrapper around a non-hashing set.
#[cfg(feature = "nohash")]
pub fn set() -> KeySet {
    KeySet::default()
}