//! Oxidize-C Transpiled Module
//! Source: sample_input.c
//! Generated: 100% Safe, idiomatic Rust with RAII ownership semantics

/// A dynamically resizing vector of 32-bit signed integers.
/// 100% Safe Rust implementation translating the legacy C IntVector.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntVector {
    data: Vec<i32>,
}

impl IntVector {
    /// Creates a new empty `IntVector` with the specified initial capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Creates a new empty `IntVector`.
    #[must_use]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Appends an integer value to the end of the vector.
    pub fn push(&mut self, value: i32) {
        self.data.push(value);
    }

    /// Returns the element at the specified index, or `None` if the index is out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<i32> {
        self.data.get(index).copied()
    }

    /// Returns the number of elements in the vector.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the vector contains no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the total number of elements the vector can hold without reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }
}

// Automatic RAII cleanup is handled by Rust's Drop implementation for Vec<i32>.
// No manual `free` is required.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_lifecycle() {
        let mut vec = IntVector::with_capacity(4);
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());

        vec.push(10);
        vec.push(20);
        vec.push(30);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(10));
        assert_eq!(vec.get(1), Some(20));
        assert_eq!(vec.get(2), Some(30));
        assert_eq!(vec.get(3), None); // Out of bounds
    }

    #[test]
    fn test_vector_reallocation() {
        let mut vec = IntVector::with_capacity(2);
        for i in 0..100 {
            vec.push(i);
        }
        assert_eq!(vec.len(), 100);
        for i in 0..100 {
            assert_eq!(vec.get(i as usize), Some(i));
        }
    }
}
