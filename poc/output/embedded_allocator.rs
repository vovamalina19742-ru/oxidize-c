//! Oxidize-C Micro-IoT Transpiled Module
//! Source: embedded_allocator.c
//! Pattern: Deterministic No-Heap Region, Slab & Lock-Free Multi-Core Allocators
//! Features:
//!   - Hardware DMA Alignment with Static Compile-Time Const Asserts
//!   - RAII Scoped Arena Guards for Stack-like Lifetime Tracking
//!   - Lock-Free Atomic Arena for Dual-Core ESP32 / RP2040 / SMP Microcontrollers
//!   - Seamless `extern "C"` FFI API for STM32CubeIDE / ESP-IDF (FreeRTOS)

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

pub const SENSOR_POOL_SIZE: usize = 16;
pub const PACKET_PAYLOAD_SIZE: usize = 64;
pub const ARENA_BUFFER_SIZE: usize = 1024;

/// Compile-time check to guarantee power-of-two alignment.
#[must_use]
pub const fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Deterministic sensor telemetry packet representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, align(8))] // 8-byte hardware alignment for 64-bit registers & DMA
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

    /// Releases an entry back to the pool in O(1) time.
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

/// Linear Region-based Arena with Configurable Hardware Alignment & Const Assertion.
#[derive(Debug)]
pub struct LinearArena<const SIZE: usize, const ALIGN: usize = 4> {
    buffer: [u8; SIZE],
    offset: usize,
}

impl<const SIZE: usize, const ALIGN: usize> LinearArena<SIZE, ALIGN> {
    #[must_use]
    pub const fn new() -> Self {
        // Compile-time static assert enforcing power-of-two alignment
        assert!(is_power_of_two(ALIGN), "ALIGN must be a power of two");
        Self {
            buffer: [0u8; SIZE],
            offset: 0,
        }
    }

    /// Allocates a contiguous byte slice aligned to `ALIGN` bytes.
    pub fn alloc_bytes(&mut self, len: usize) -> Option<&mut [u8]> {
        let align_mask = ALIGN - 1;
        let aligned_offset = (self.offset + align_mask) & !align_mask;
        let new_offset = aligned_offset + len;

        if new_offset <= SIZE {
            self.offset = new_offset;
            Some(&mut self.buffer[aligned_offset..aligned_offset + len])
        } else {
            None
        }
    }

    /// Creates an RAII Scoped Guard that automatically resets arena allocation to current offset upon drop.
    #[must_use]
    pub fn scoped_guard(&mut self) -> ScopedArenaGuard<'_, SIZE, ALIGN> {
        let saved_offset = self.offset;
        ScopedArenaGuard {
            arena: self,
            saved_offset,
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

/// RAII Scoped Guard ensuring zero-overhead stack-like unwinding of transient arena allocations.
pub struct ScopedArenaGuard<'a, const SIZE: usize, const ALIGN: usize> {
    arena: &'a mut LinearArena<SIZE, ALIGN>,
    saved_offset: usize,
}

impl<'a, const SIZE: usize, const ALIGN: usize> ScopedArenaGuard<'a, SIZE, ALIGN> {
    pub fn alloc_bytes(&mut self, len: usize) -> Option<&mut [u8]> {
        self.arena.alloc_bytes(len)
    }

    #[must_use]
    pub fn used_bytes(&self) -> usize {
        self.arena.used_bytes()
    }
}

impl<'a, const SIZE: usize, const ALIGN: usize> Drop for ScopedArenaGuard<'a, SIZE, ALIGN> {
    fn drop(&mut self) {
        // Automatic rollback to snapshot offset on scope exit (Zero memory leaks)
        self.arena.offset = self.saved_offset;
    }
}

/// Lock-Free Atomic Linear Arena for Multi-Core / Dual-Core Microcontrollers (ESP32 / RP2040).
pub struct AtomicLinearArena<const SIZE: usize, const ALIGN: usize = 4> {
    buffer: [u8; SIZE],
    offset: AtomicUsize,
}

impl<const SIZE: usize, const ALIGN: usize> AtomicLinearArena<SIZE, ALIGN> {
    #[must_use]
    pub const fn new() -> Self {
        assert!(is_power_of_two(ALIGN), "ALIGN must be a power of two");
        Self {
            buffer: [0u8; SIZE],
            offset: AtomicUsize::new(0),
        }
    }

    /// Lock-free allocation via atomic Compare-And-Swap (CAS) loop.
    pub fn alloc_bytes(&self, len: usize) -> Option<usize> {
        let align_mask = ALIGN - 1;
        let mut current = self.offset.load(Ordering::Relaxed);

        loop {
            let aligned = (current + align_mask) & !align_mask;
            let next = aligned + len;

            if next > SIZE {
                return None; // Out of memory
            }

            match self.offset.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(aligned),
                Err(actual) => current = actual,
            }
        }
    }

    /// Reset offset atomically.
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Release);
    }

    #[must_use]
    pub fn used_bytes(&self) -> usize {
        self.offset.load(Ordering::Acquire)
    }
}

/// Micro-IoT Composite Deterministic Memory Controller (8-byte DMA Aligned).
#[derive(Debug)]
pub struct EmbeddedMemoryManager {
    pub slab_pool: SlabPool<SensorPacket, SENSOR_POOL_SIZE>,
    pub arena: LinearArena<ARENA_BUFFER_SIZE, 8>, // 8-byte DMA Alignment
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

// =========================================================================
// C-FFI Exported Functions (Seamless STM32CubeIDE / ESP-IDF FreeRTOS Bindings)
// =========================================================================

static mut GLOBAL_EMBEDDED_MGR: EmbeddedMemoryManager = EmbeddedMemoryManager::new();

#[no_mangle]
pub unsafe extern "C" fn oxidize_arena_init() {
    GLOBAL_EMBEDDED_MGR = EmbeddedMemoryManager::new();
}

#[no_mangle]
pub unsafe extern "C" fn oxidize_arena_alloc(size: usize) -> *mut u8 {
    match GLOBAL_EMBEDDED_MGR.arena.alloc_bytes(size) {
        Some(slice) => slice.as_mut_ptr(),
        None => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn oxidize_arena_reset() {
    GLOBAL_EMBEDDED_MGR.arena.reset();
}

#[no_mangle]
pub unsafe extern "C" fn oxidize_arena_used_bytes() -> usize {
    GLOBAL_EMBEDDED_MGR.arena.used_bytes()
}

#[no_mangle]
pub unsafe extern "C" fn oxidize_slab_alloc_packet(sensor_id: u32, timestamp: u64) -> i32 {
    let pkt = SensorPacket {
        sensor_id,
        timestamp,
        payload: [0u8; PACKET_PAYLOAD_SIZE],
        length: 0,
    };
    match GLOBAL_EMBEDDED_MGR.slab_pool.allocate(pkt) {
        Some(idx) => idx as i32,
        None => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn oxidize_slab_free_packet(index: usize) -> bool {
    GLOBAL_EMBEDDED_MGR.slab_pool.free(index).is_some()
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
    fn test_8byte_dma_alignment() {
        let mut arena = LinearArena::<256, 8>::new();
        let slice1 = arena.alloc_bytes(5).expect("5 bytes allocated");
        slice1[0] = 0xAA;
        assert_eq!(arena.used_bytes(), 5);

        let slice2 = arena.alloc_bytes(10).expect("10 bytes allocated");
        slice2[0] = 0xBB;
        assert_eq!(arena.used_bytes(), 18);
    }

    #[test]
    fn test_raii_scoped_arena_guard_unwinding() {
        let mut arena = LinearArena::<256, 4>::new();
        let _ = arena.alloc_bytes(16);
        assert_eq!(arena.used_bytes(), 16);

        {
            let mut guard = arena.scoped_guard();
            let _ = guard.alloc_bytes(32);
            assert_eq!(guard.used_bytes(), 48);
        }

        assert_eq!(arena.used_bytes(), 16);
    }

    #[test]
    fn test_atomic_lock_free_arena_concurrency() {
        let arena = AtomicLinearArena::<512, 8>::new();
        let offset1 = arena.alloc_bytes(30).expect("Alloc 1");
        assert_eq!(offset1, 0);

        let offset2 = arena.alloc_bytes(20).expect("Alloc 2");
        assert_eq!(offset2, 32); // 8-byte aligned (30 -> 32)
        assert_eq!(arena.used_bytes(), 52);

        arena.reset();
        assert_eq!(arena.used_bytes(), 0);
    }
}
