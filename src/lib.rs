#![deny(missing_docs)]

//! # About
//! A trivial and fast[^1] slab container that doesn't grow and provides additional guarantees[^2].
//!
//! [^1]: Hasn't been profiled really.
//!
//! [^2]: I haven't figured out how to tell that to Rust, so unsafe is necessary.

/// Limited size preallocated slab storage that won't reallocate ever
///
/// # Example
/// ```
/// use ochenslab::OchenSlab;
///
/// let mut slab = OchenSlab::<usize>::with_capacity(2);
///
/// let a = slab.insert(31337);
/// let b = slab.insert(31338);
/// assert!(a.is_some());
/// assert!(b.is_some());
///
/// // at this point container is at its max capacity
/// let c = slab.insert(31339);
/// assert!(c.is_none());
///
/// assert_eq!(*slab.get(a.unwrap()).unwrap(), 31337);
/// assert_eq!(*slab.get(b.unwrap()).unwrap(), 31338);
/// ```
pub struct OchenSlab<T> {
    // Primary storage for items
    storage: Vec<Option<T>>,

    // Storage for free indices
    free: Vec<usize>,
}

impl<T> OchenSlab<T> {
    /// Create slab instance with given capacity
    /// Capacity will be constant for the entire lifetime of this object and cannot increase
    pub fn with_capacity(capacity: usize) -> OchenSlab<T> {
        let mut storage = Vec::<Option<T>>::with_capacity(capacity);
        storage.resize_with(capacity, || None);
        let mut free = Vec::<usize>::with_capacity(capacity);
        let mut i = 0 as usize;
        free.resize_with(capacity, || {
            let value = capacity - 1 - i;
            i += 1;
            value
        });

        OchenSlab {
            storage, free
        }
    }

    /// Return number of elements in this container
    pub fn len(&self) -> usize {
        self.storage.len() - self.free.len()
    }

    /// Get reference to an item by its index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.storage.get(index)?.as_ref()
    }

    /// Get mutable reference to an item by its index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.storage.get_mut(index)?.as_mut()
    }

    /// Insert a new item and return its index.
    /// Returns None if there's no space left
    /// Contrary to Rust opinion this cannot affect any other items in this container, so it is
    /// safe to hold a mutable reference to some other item while inserting another. This is
    /// because insert will not ever reallocate, so it can't invalidate existing pointers.
    pub fn insert(&mut self, t: T) -> Option<usize> {
        let index = self.free.pop()?;
        *self.storage.get_mut(index)? = Some(t);
        Some(index)
    }

    /// Remove an item by its index.
    /// Returns the item by value if there was one
    pub fn remove(&mut self, index: usize) -> Option<T> {
        let value = self.storage.get_mut(index)?.take()?;
        self.free.push(index);
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_insert_and_remove_an_element() {
        let mut slab = OchenSlab::<usize>::with_capacity(8);
        let index = slab.insert(31337);
        assert!(index.is_some());
        assert_eq!(slab.len(), 1);
        let index = index.unwrap();
        let item = slab.get(index);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(*item, 31337);
        slab.remove(index);
        assert!(slab.get(index).is_none());
        assert_eq!(slab.len(), 0);
    }

    #[test]
    fn can_reach_capacity() {
        let mut slab = OchenSlab::<usize>::with_capacity(4);
        assert!(slab.insert(1).is_some());
        assert!(slab.insert(2).is_some());
        assert!(slab.insert(3).is_some());
        assert!(slab.insert(4).is_some());
        assert_eq!(slab.len(), 4);
        assert!(slab.insert(5).is_none());
    }

    #[test]
    fn can_reach_capacity_and_back() {
        let mut slab = OchenSlab::<usize>::with_capacity(4);
        assert!(slab.insert(1).is_some());
        let index = slab.insert(2);
        assert!(index.is_some());
        assert!(slab.insert(3).is_some());
        assert!(slab.insert(4).is_some());
        assert!(slab.insert(5).is_none());
        assert_eq!(slab.len(), 4);
        let item = slab.remove(index.unwrap());
        assert_eq!(slab.len(), 3);
        assert!(item.is_some());
        let item = item.unwrap();
        assert_eq!(item, 2);
        assert!(slab.insert(6).is_some());
        assert_eq!(slab.len(), 4);
        assert!(slab.insert(7).is_none());
        assert_eq!(slab.len(), 4);
    }

    #[test]
    fn can_mutate_element() {
        let mut slab = OchenSlab::<usize>::with_capacity(4);
        let index = slab.insert(1).expect("insert() failed");
        let item = slab.get_mut(index).expect("get_mut() failed");
        *item = 2;
        let item = slab.get(index).expect("get() failed");
        assert_eq!(*item, 2);
    }
}
