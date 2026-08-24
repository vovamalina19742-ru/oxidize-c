//! Oxidize-C Transpiled Module
//! Source: ring_buffer.c
//! Pattern: Automatic RAII Vector with Wrapping Indices

/// A 100% Safe Circular Ring Buffer.
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    count: usize,
}

impl<T: Clone> RingBuffer<T> {
    #[must_use]
    pub fn new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            return None;
        }
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, None);

        Some(Self {
            buffer,
            capacity,
            head: 0,
            tail: 0,
            count: 0,
        })
    }

    pub fn push(&mut self, item: T) -> bool {
        if self.count == self.capacity {
            return false;
        }
        self.buffer[self.tail] = Some(item);
        self.tail = (self.tail + 1) % self.capacity;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.count -= 1;
        item
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        self.count == self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_fifo() {
        let mut rb = RingBuffer::new(3).expect("Valid capacity");
        assert!(rb.is_empty());

        assert!(rb.push(1));
        assert!(rb.push(2));
        assert!(rb.push(3));
        assert!(!rb.push(4)); // Full!
        assert!(rb.is_full());

        assert_eq!(rb.pop(), Some(1));
        assert_eq!(rb.pop(), Some(2));
        assert!(rb.push(40)); // Wraparound push

        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), Some(40));
        assert_eq!(rb.pop(), None); // Empty!
    }
}
