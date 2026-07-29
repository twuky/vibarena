use core::iter::Zip;
use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut};
use nohash_hasher::IsEnabled;
use std::hash::Hash;
use std::num::NonZeroU32;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Key {
    idx: u32,
    ver: NonZeroU32,
}

impl Key {
    pub fn new(idx: u32, ver: NonZeroU32) -> Key {
        Key { idx, ver }
    }

    /// Returns the index of the key in the arena's internal vector.
    pub fn idx(&self) -> u32 {
        self.idx
    }

    /// Returns the version of the key in the arena's internal vector.
    pub fn ver(&self) -> NonZeroU32 {
        self.ver
    }

    pub fn parts(&self) -> (u32, NonZeroU32) {
        (self.idx, self.ver)
    }
}

impl IsEnabled for Key {}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(((self.ver.get() as u64) << 32) | self.idx as u64);
    }
}

#[derive(Clone)]
pub struct Arena<T> {
    outers: Vec<u32>,
    versions: Vec<NonZeroU32>,
    data: Vec<T>,
    inner: Vec<u32>,
}

impl<T> Arena<T> {
    /// Create a new empty `Arena`.
    pub fn new() -> Arena<T> {
        Self::with_capacity(32)
    }

    /// Create a new empty `Arena` with the given capacity.
    pub fn with_capacity(capacity: usize) -> Arena<T> {
        Arena {
            outers: Vec::with_capacity(capacity),
            versions: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the arena.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the capacity of the arena.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Clears the arena, removing all elements.
    /// This increments the version counter for all slots.
    pub fn clear(&mut self) {
        self.data.clear();
        for version in &mut self.versions {
            *version = version.saturating_add(1);
        }

        for (counter, slot) in &mut self.inner.iter_mut().enumerate() {
            *slot = counter as u32;
        }
    }

    /// Inserts a value into the arena.
    /// Returns the key that was used to store the value.
    pub fn insert(&mut self, value: T) -> Key {
        let index = self.data.len() as u32;
        self.data.push(value);
        
        if index as usize == self.outers.len() {
            let z = NonZeroU32::MIN;
            self.outers.push(index);
            self.versions.push(z);
            self.inner.push(index);
            Key { idx: index, ver: z }
        } else {
            unsafe {
                let key_index = *self.inner.get_unchecked(index as usize);
                let version = self.versions.get_unchecked_mut(key_index as usize);
                self.outers[key_index as usize] = index;
                Key {
                    idx: key_index,
                    ver: *version,
                }
            }
        }
    }

    /// Inserts a value into the arena only if the internal vec is not at capacity.
    /// Returns the key that was used to store the value.
    /// If the arena is full, the value is returned instead.
    pub fn try_insert(&mut self, value: T) -> Result<Key, T> {
        if self.data.len() == self.capacity() {
            return Err(value);
        }
        Ok(self.insert(value))
    }

    /// Removes a value from the arena.
    /// Returns the removed value, or `None` if the key is invalid.
    pub fn remove(&mut self, key: Key) -> Option<T> {
        let version = self.versions.get_mut(key.idx as usize)?;
        if *version != key.ver {
            return None;
        }
        *version = version.saturating_add(1);

        let remove_index = unsafe { *self.outers.get_unchecked(key.idx as usize) };
        let removed = self.data.swap_remove(remove_index as usize);
        unsafe {
            let slot = self.inner.get_unchecked_mut(self.data.len());
            let update_index = *slot;
            *slot = key.idx;
            *self.inner.get_unchecked_mut(remove_index as usize) = update_index;
            *self.outers.get_unchecked_mut(update_index as usize) = remove_index;
            Some(removed)
        }
    }

    /// Inserts a value into the arena using a closure that provides the key.
    /// This is useful for a data type that requires a self-referential key.
    pub fn insert_with_key(&mut self, closure: impl FnOnce(Key) -> T) -> Key {
        let index = self.data.len() as u32;
        let key = if index as usize == self.outers.len() {
            self.outers.push(index);
            self.versions.push(NonZeroU32::MIN);
            self.inner.push(index);
            Key {
                idx: index,
                ver: NonZeroU32::MIN,
            }
        } else {
            unsafe {
                let key_index = *self.inner.get_unchecked(index as usize);
                *self.outers.get_unchecked_mut(key_index as usize) = index;
                Key {
                    idx: key_index,
                    ver: *self.versions.get_unchecked(key_index as usize),
                }
            }
        };
        self.data.push(closure(key));
        // Record inserts made during an active walk so the cursor skips them.
        key
    }

    /// Inserts a value into the arena using a closure that provides the key.
    /// 
    /// This is useful for a data type that requires a self-referential key.
    /// If the arena is at capacity, the closure is returned instead.
    pub fn try_insert_with_key(&mut self, closure: impl FnOnce(Key) -> T) -> Result<Key, impl FnOnce(Key) -> T> {
        if self.data.len() == self.capacity() {
            return Err(closure);
        }
        Ok(self.insert_with_key(closure))
    }

    /// Returns a reference to the value associated with the key.
    /// Returns `None` if the key is invalid.
    pub fn get(&self, key: Key) -> Option<&T> {
        let &version = self.versions.get(key.idx as usize)?;
        if version != key.ver {
            return None;
        }
        unsafe {
            let outer = *self.outers.get_unchecked(key.idx as usize);
            Some(self.data.get_unchecked(outer as usize))
        }
    }

    /// Returns a mutable reference to the value associated with the key.
    /// Returns `None` if the key is invalid.
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        let &version = self.versions.get(key.idx as usize)?;
        if version != key.ver {
            return None;
        }
        unsafe {
            let outer = *self.outers.get_unchecked(key.idx as usize);
            Some(self.data.get_unchecked_mut(outer as usize))
        }
    }

    /// Returns `true` if the value is present.
    pub fn contains(&self, value: &T) -> bool
    where T: PartialEq<T> {
        self.data.contains(value)
    }

    /// Returns `true` if the key is valid and the value is present.
    pub fn contains_key(&self, key: Key) -> bool {
        let version = self.versions.get(key.idx as usize);
        version.is_some() && unsafe { *version.unwrap_unchecked() == key.ver }
    }

    /// Returns a reference to the value associated with the key.
    /// Returns `None` if the key is invalid.
    /// # Safety
    /// The key must be valid.
    pub unsafe fn get_unchecked(&self, key: Key) -> Option<&T> { unsafe {
        if *self.versions.get_unchecked(key.idx as usize) != key.ver {
            return None;
        }
        let outer = *self.outers.get_unchecked(key.idx as usize);
        Some(self.data.get_unchecked(outer as usize))
    }}

    /// Returns a mutable reference to the value associated with the key.
    /// Returns `None` if the key is invalid.
    /// # Safety
    /// The key must be valid.
    pub unsafe fn get_unchecked_mut(&mut self, key: Key) -> Option<&mut T> { unsafe {
        if *self.versions.get_unchecked(key.idx as usize) != key.ver {
            return None;
        }
        let outer = *self.outers.get_unchecked(key.idx as usize);
        Some(self.data.get_unchecked_mut(outer as usize))
    }}

    /// Returns an iterator over the values in the arena.
    pub fn iter(&self) -> Iter<'_, T> {
        self.data.iter()
    }

    /// Returns an iterator over the values in the arena.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.data.iter_mut()
    }

    /// Returns an iterator over the arena as key-value pairs.
    pub fn iter_keyed(&self) -> IterKeyed<'_, T> {
        let len = self.data.len();
        IterKeyed {
            versions: &self.versions,
            iter: self.inner[..len].iter().zip(self.data.iter()),
        }
    }

    /// Returns a mutable iterator over the arena as key-value pairs.
    pub fn iter_keyed_mut(&mut self) -> IterKeyedMut<'_, T> {
        let len = self.data.len();
        IterKeyedMut {
            versions: &self.versions,
            iter: self.inner[..len].iter().zip(self.data.iter_mut()),
        }
    }

    /// Returns an iterator over the keys in the arena.
    pub fn keys(&self) -> Keys<'_> {
        let len = self.data.len();
        Keys {
            versions: &self.versions,
            iter: self.inner[..len].iter(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = &'a T ;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = &'a mut T ;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// Iterator over `Key`s in dense storage order.
pub struct Keys<'a> {
    versions: &'a [NonZeroU32],
    iter: Iter<'a, u32>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        let &idx = self.iter.next()?;
        // `inner` within the active region always points to a live slot.
        let ver = unsafe { *self.versions.get_unchecked(idx as usize) };
        Some(Key { idx, ver })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for Keys<'a> {}


/// Iterator over `(Key, &T)` pairs in dense storage order.
pub struct IterKeyed<'a, T> {
    versions: &'a [NonZeroU32],
    iter: Zip<Iter<'a, u32>, Iter<'a, T>>,
}

impl<'a, T> Iterator for IterKeyed<'a, T> {
    type Item = (Key, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let (&idx, value) = self.iter.next()?;
        // `inner` within the active region always points to a live slot.
        let ver = unsafe { *self.versions.get_unchecked(idx as usize) };
        Some((Key { idx, ver }, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for IterKeyed<'a, T> {}

/// Iterator over `(Key, &mut T)` pairs in dense storage order.
pub struct IterKeyedMut<'a, T> {
    versions: &'a [NonZeroU32],
    iter: Zip<Iter<'a, u32>, IterMut<'a, T>>,
}

impl<'a, T> Iterator for IterKeyedMut<'a, T> {
    type Item = (Key, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let (&idx, value) = self.iter.next()?;
        // `inner` within the active region always points to a live slot.
        let ver = unsafe { *self.versions.get_unchecked(idx as usize) };
        Some((Key { idx, ver }, value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for IterKeyedMut<'a, T> {}

impl<T> Index<Key> for Arena<T> {
    type Output = T;

    fn index(&self, key: Key) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<T> IndexMut<Key> for Arena<T> {
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}
