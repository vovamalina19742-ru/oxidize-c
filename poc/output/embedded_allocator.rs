//! Oxidize-C Micro-IoT Transpiled Module
//! Source: embedded_allocator.c
//! Pattern: Deterministic No-Heap Region & Slab Allocators for <=128 KB RAM Systems

#![allow(dead_code)]

pub const SENSOR_POOL_SIZE: usize = 16;
pub const PACKET_PAYLOAD_SIZE: usize = 64;
pub const ARENA_BUFFER_SIZE: usize = 1024;

/// Deterministic sensor telemetry packet representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorPacket {
    pub sensor_id: u32,
    pub timestamp: u64,
    pub payload: [u8; PACKET_PAYLOAD_SIZE],
    pub length: u16,
}

impl Default for SensorPacket {
    fn default() -> Self {
        Self {
            sensor_id: 0,
            timestamp: 0,
            payload: [0u8; PACKET_PAYLOAD_SIZE],
            length: 0,
        }
    }
}

/// Fixed-size Slab Pool delivering deterministic O(1) allocation without fragmentation.
#[derive(Debug)]
pub struct SlabPool<T, const N: usize> {
    slots: [Option<T>; N],
}

impl<T: Copy + Default, const N: usize> SlabPool<T, N> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: [None; N],
        }
    }

    /// Allocates an entry with O(1) deterministic complexity.
    pub fn allocate(&mut self, item: T) -> Option<usize> {
        for (idx, slot) in self.slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(item);
                return Some(idx);
            }
        }
        None
    }

    /// Releases an entry back to the pool.
    pub fn free(&mut self, index: usize) -> Option<T> {
        if index < N {
            self.slots[index].take()
        } else {
            None
        }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < N {
            self.slots[index].as_ref()
        } else {
            None
        }
    }
}

/// Linear Region-based Arena for transient micro-task scratch memory.
#[derive(Debug)]
pub struct LinearArena<const SIZE: usize> {
    buffer: [u8; SIZE],
    offset: usize,
}

impl<const SIZE: usize> LinearArena<SIZE> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; SIZE],
            offset: 0,
        }
    }

    /// Allocates a contiguous byte slice aligned to 4 bytes.
    pub fn alloc_bytes(&mut self, len: usize) -> Option<&mut [u8]> {
        let aligned_len = (len + 3) & !3;
        if self.offset + aligned_len <= SIZE {
            let start = self.offset;
            self.offset += aligned_len;
            Some(&mut self.buffer[start..start + len])
        } else {
            None
        }
    }

    /// Instantaneous O(1) bulk deallocation of the entire region.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    #[must_use]
    pub fn used_bytes(&self) -> usize {
        self.offset
    }

    #[must_use]
    pub fn available_bytes(&self) -> usize {
        SIZE - self.offset
    }
}

/// Micro-IoT Composite Deterministic Memory Controller.
#[derive(Debug)]
pub struct EmbeddedMemoryManager {
    pub slab_pool: SlabPool<SensorPacket, SENSOR_POOL_SIZE>,
    pub arena: LinearArena<ARENA_BUFFER_SIZE>,
}

impl EmbeddedMemoryManager {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slab_pool: SlabPool::new(),
            arena: LinearArena::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slab_pool_deterministic_alloc() {
        let mut pool = SlabPool::<SensorPacket, 4>::new();
        let pkt = SensorPacket {
            sensor_id: 101,
            timestamp: 123456789,
            payload: [42u8; PACKET_PAYLOAD_SIZE],
            length: 1,
        };

        let idx0 = pool.allocate(pkt).expect("Alloc slot 0");
        assert_eq!(idx0, 0);

        let retrieved = pool.get(0).expect("Item present");
        assert_eq!(retrieved.sensor_id, 101);

        let freed = pool.free(0).expect("Freed item");
        assert_eq!(freed.sensor_id, 101);
        assert!(pool.get(0).is_none());
    }

    #[test]
    fn test_linear_arena_reset() {
        let mut arena = LinearArena::<256>::new();
        let slice = arena.alloc_bytes(10).expect("10 bytes allocated");
        slice[0] = 0xAA;
        assert_eq!(arena.used_bytes(), 12); // Aligned to 4 bytes

        arena.reset();
        assert_eq!(arena.used_bytes(), 0);
    }
}
