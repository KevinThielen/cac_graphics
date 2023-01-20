//! Collection for Handles with automatic invalidation
//!
//! Values will return a handle upon insertion. Once the value has been removed or replaced
//! all existing handles to it will be invalidated, returning None when trying to retrieve the
//! value.
//!
//! The first generic argument, `K`, establishes a type safe connection between handles and the collection.
//!
//! The second generic argument, `V`, is the actual Value stored inside the collection.
//!
//!
//! # Examples
//!
//! ```
//!    use cac_core::gen_vec;
//!    // The `Key` argument could be any type. It is recommended to create every
//!    // storage with a unique Key, so that handles can only be used with that specific storage.
//!    // This prevents accidents, like removing a value from the wrong collection.
//!    struct Key;
//!    let mut storage = gen_vec::GenVec::<Key, _>::new();
//!
//!    //insertion returns handles to the value
//!    let handle_0 = storage.insert("foo");
//!    let handle_1 = storage.insert("bar");
//!
//!    assert_eq!(storage.get(handle_0), Some(&"foo"));
//!    assert_eq!(storage.get(handle_1), Some(&"bar"));
//!
//!    //mutating a value doesn't change it's handle.
//!    if let Some(value) = storage.get_mut(handle_1) {
//!         *value = "changed the value";
//!    }
//!    assert_eq!(storage.get(handle_1), Some(&"changed the value"));
//!
//!    //removing a value invalidates existing handles to that value
//!    let old_value = storage.remove(handle_0);
//!    assert_eq!(old_value, Some("foo"));
//!    assert_eq!(storage.get(handle_0), None);
//! ```

use std::marker::PhantomData;

/// Handle to a value inserted into the `GenVec`
pub struct Handle<K> {
    index: usize,
    generation: u32,
    phantom: PhantomData<K>,
}

//Manually implement deriveable traits because of the PhantomData.
//There is no need to force K to implement any of them via bounds, so they can't be derived. What a
//bummer.
impl<K> Copy for Handle<K> {}
impl<K> Clone for Handle<K> {
    fn clone(&self) -> Self {
        Self {
            phantom: PhantomData,
            index: self.index,
            generation: self.generation,
        }
    }
}
impl<K> Eq for Handle<K> {}
impl<K> PartialEq for Handle<K> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

/// Storage for values that invalidates handles to them once the values are removed/replaces
///
/// The argument K allows type safety to make Handles only work with matching
/// `GenVec` with the same K argument.
///
/// The argument V is the actual value being stored in the collection.
pub struct GenVec<K, V> {
    values: Vec<Value<V>>,
    free: Vec<usize>,
    phantom: PhantomData<K>,
}

struct Value<V> {
    value: Option<V>,
    generation: u32,
}

impl<K, V> GenVec<K, V> {
    /// Constructor
    ///
    /// # Examples
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// //creates a `GenVec` storing Strings  
    /// let storage = gen_vec::GenVec::<u32, String>::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: Vec::new(),
            free: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Constructs the `GenVec` and preallocates at least to a certain capacity.
    /// The capacity is at least twice the argument, because it also pre-allocates the space for
    /// the empty values.
    ///
    /// # Examples
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// //creates a `GenVec` storing Strings with a default capacity of 10
    /// let storage = gen_vec::GenVec::<String, String>::with_capacity(10);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            free: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    /// Constructs the `GenVec` and inserts the passed values.
    ///
    /// In addition to the `GenVec`, this function will also return the created handles
    /// to the passed values in the same order as they were passed.
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// struct Key;
    ///
    /// let (handles, storage) = gen_vec::GenVec::<Key, _>::with_values(["foo", "bar"]);
    ///
    /// assert_eq!(storage.get(handles[0]), Some(&"foo"));
    /// assert_eq!(storage.get(handles[1]), Some(&"bar"));
    /// ```
    #[must_use]
    pub fn with_values<const N: usize>(values: [V; N]) -> ([Handle<K>; N], Self) {
        let mut storage = Self::with_capacity(N);

        let mut handles: [Handle<_>; N] = [Handle {
            index: 0,
            generation: 0,
            phantom: PhantomData,
        }; N];

        values.into_iter().enumerate().for_each(|(i, v)| {
            handles[i] = storage.insert(v);
        });

        (handles, storage)
    }

    /// Gets the immutable reference to the value associated with the `Handle`.
    ///
    /// If the `Handle` has no valid associated value, the returned value is `None`.
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// struct Key;
    /// let mut storage = gen_vec::GenVec::<Key, _>::new();
    ///
    /// let handle_0 = storage.insert("foo");
    /// let handle_1 = storage.insert("bar");
    ///
    /// assert_eq!(storage.get(handle_0), Some(&"foo"));
    /// assert_eq!(storage.get(handle_1),Some(&"bar"));
    /// ```
    #[must_use]
    pub fn get(&self, handle: Handle<K>) -> Option<&V> {
        self.values
            .get(handle.index)
            .filter(|r| r.generation == handle.generation)
            .and_then(|r| r.value.as_ref())
    }

    /// Gets the mutable reference to the value associated with the `Handle`.
    ///
    /// If the `Handle` has no valid associated value associated, the returned value is `None`.
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// struct Key;
    /// let mut storage = gen_vec::GenVec::<Key, _>::new();
    /// let handle = storage.insert(5);
    ///
    /// if let Some(value) = storage.get_mut(handle) {
    ///     *value *= 3;
    /// }
    ///
    /// assert_eq!(storage.get(handle), Some(&15));
    /// ```
    #[must_use]
    pub fn get_mut(&mut self, handle: Handle<K>) -> Option<&mut V> {
        self.values
            .get_mut(handle.index)
            .filter(|r| r.generation == handle.generation)
            .and_then(|r| r.value.as_mut())
    }

    /// Inserts a new value into the collection and returns a `Handle` to it.
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// struct Key;
    /// let mut storage = gen_vec::GenVec::<Key, _>::new();
    ///
    /// let handle_0 = storage.insert("foo");
    /// let handle_1 = storage.insert("bar");
    ///
    /// assert_eq!(storage.get(handle_0), Some(&"foo"));
    /// assert_eq!(storage.get(handle_1), Some(&"bar"));
    /// ```
    pub fn insert(&mut self, value: V) -> Handle<K> {
        //take an index out of the "free" vec, or push a new value
        if let Some(index) = self.free.pop() {
            let v = self
                .values
                .get_mut(index)
                .expect("The free list should be unable to store indices that are out of bounds!");

            v.generation += 1;
            v.value = Some(value);

            Handle {
                index,
                generation: v.generation,
                phantom: PhantomData,
            }
        } else {
            let index = self.values.len();
            let generation = 0;
            self.values.push(Value {
                value: Some(value),
                generation,
            });

            Handle {
                index,
                generation,
                phantom: PhantomData,
            }
        }
    }

    /// Remove the value associated with the `Handle` from the collection.
    ///
    /// All handles to the removed value will be invalidated.
    /// Returns the removed Value, or `None` if the valid is invalid.
    /// ```
    /// use cac_core::gen_vec;
    ///
    /// struct Key;
    /// let mut storage = gen_vec::GenVec::<Key, _>::new();
    ///
    /// let handle_0 = storage.insert("foo");
    /// let handle_1 = storage.insert("bar");
    ///
    /// let old_value = storage.remove(handle_0);
    ///
    /// assert_eq!(old_value, Some("foo"));
    /// assert_eq!(storage.get(handle_0), None);
    /// assert_eq!(storage.get(handle_1), Some(&"bar"));
    /// ```
    pub fn remove(&mut self, handle: Handle<K>) -> Option<V> {
        self.values
            .get_mut(handle.index)
            .filter(|v| v.generation == handle.generation)
            .and_then(|v| {
                self.free.push(handle.index);
                v.value.take()
            })
    }
}

impl<K, V> Default for GenVec<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub mod test {
    pub use super::*;

    //create a test storage with N elements
    fn test_storage<const N: usize>() -> ([Handle<u32>; N], GenVec<u32, String>) {
        let values: Vec<_> = (0..N).map(|i| format!("Value{i}")).collect();

        GenVec::with_values(values.try_into().unwrap())
    }

    #[test]
    fn construct_with_values_test() {
        let values = [
            "Value0".to_owned(),
            "Value1".to_owned(),
            "Value2".to_owned(),
        ];

        let (handles, storage) = GenVec::<i32, _>::with_values(values);

        for (i, h) in handles.iter().enumerate() {
            assert_eq!(storage.get(*h), Some(&format!("Value{i}")));
        }
    }

    #[test]
    fn insertion_test() {
        let mut storage: GenVec<String, String> = GenVec::new();
        let mut handles = Vec::new();

        (0..10).for_each(|i| {
            handles.push(storage.insert(format!("Value{i}")));
        });

        for (i, h) in handles.iter().enumerate() {
            assert_eq!(storage.get(*h), Some(&format!("Value{i}")));
        }
    }

    #[test]
    fn free_cycle_test() {
        let (handles, mut storage) = test_storage::<10>();

        storage.remove(handles[0]);
        storage.remove(handles[5]);
        storage.remove(handles[7]);

        let h0 = storage.insert("4".to_owned());
        let h1 = storage.insert("2".to_owned());
        let h2 = storage.insert("42".to_owned());

        //check for proper insertions
        assert_eq!(storage.get(h0), Some(&"4".to_owned()));
        assert_eq!(storage.get(h1), Some(&"2".to_owned()));
        assert_eq!(storage.get(h2), Some(&"42".to_owned()));
    }
    #[test]
    fn removal_test() {
        let (handles, mut storage) = test_storage::<3>();

        let old_val = storage.remove(handles[2]);
        assert_eq!(old_val, Some(format!("Value{}", 2)));

        handles.iter().enumerate().for_each(|(i, h)| {
            if i == 2 {
                assert_eq!(storage.get(*h), None);
            } else {
                assert_eq!(storage.get(*h), Some(&format!("Value{i}")));
            }
        });
    }

    #[test]
    fn get_mutable_test() {
        let mut storage: GenVec<String, String> = GenVec::new();

        let handle = storage.insert("some_string".to_owned());
        let handle_copy = handle;

        if let Some(s) = storage.get_mut(handle) {
            *s = "ANOTHER STRING".to_owned();
        }

        //value has changed
        assert_eq!(storage.get(handle), Some(&"ANOTHER STRING".to_owned()));

        //both should point to the same value
        assert_eq!(storage.get(handle), storage.get(handle_copy));
    }

    #[test]
    fn insert_over_capacity_test() {
        let mut storage: GenVec<String, String> = GenVec::with_capacity(3);

        for i in 0..100 {
            storage.insert(format!("{i}"));
        }
    }
}
