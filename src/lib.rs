mod arena;


pub use arena::Arena;
pub use arena::Key;

pub type KeySet = nohash_hasher::IntSet<Key>;
pub type ArenaMap<T> = nohash_hasher::IntMap<Key, T>;

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

/// Create a new empty `ArenaMap`.
///
/// This is a "secondary" arena that can be used to assign secondary data to keys.
/// This is a wrapper around a non-hashing map.
pub fn map<T>() -> ArenaMap<T> {
    ArenaMap::<T>::default()
}

/// Create a new empty `KeySet`.
///
/// This is a set of unique keys, that does not need to hash.
/// This is a wrapper around a non-hashing set.
pub fn set() -> KeySet {
    KeySet::default()
}