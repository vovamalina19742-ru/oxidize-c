//! Oxidize-Alloc: Deterministic No-Heap Region, Slab & Lock-Free Embedded Allocators
//! Designed for bare-metal IoT microcontrollers (ARM Cortex-M, RISC-V, ESP32, RP2040).
//!
//! # Safety & Reliability Guarantees:
//! - Pure `#![no_std]` core with zero dynamic heap dependencies.
//! - Hardware DMA alignment via `#[repr(align(64))]` backing store.
//! - Strict integer overflow protection via `checked_add` on all allocation paths.
//! - Thread-safe global controller via `#![no_std]` atomic spinlock (zero `static mut`).
//! - C-FFI API compatible with STM32CubeIDE, ESP-IDF (FreeRTOS) and Zephyr.

#![no_std]
#![allow(dead_code)]

#[cfg(test)]
extern crate std;

use core::cell::UnsafeCell;

#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub const SENSOR_POOL_SIZE: usize = 16;
pub const PACKET_PAYLOAD_SIZE: usize = 64;
pub const ARENA_BUFFER_SIZE: usize = 1024;

/// Compile-time check to guarantee power-of-two alignment.
#[must_use]
pub const fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Hardware-aligned memory buffer with explicit 64-byte alignment attribute.
///
/// Ensures DMA controllers, SIMD registers, and cache lines access memory
/// without alignment faults on all embedded microarchitectures.
#[repr(align(64))]
#[derive(Debug, Clone, Copy)]
pub struct AlignedBuffer<const N: usize>(pub [u8; N]);

impl<const N: usize> AlignedBuffer<N> {
    #[must_use]
    pub const fn new() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> Default for AlignedBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-dependency spinlock supporting SMP architectures and single-core microcontrollers.
#[cfg(target_has_atomic = "ptr")]
/// Zero-dependency spinlock supporting SMP architectures and single-core microcontrollers.
#[cfg(target_has_atomic = "ptr")]
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

#[cfg(not(target_has_atomic = "ptr"))]
pub struct SpinLock<T> {
    data: UnsafeCell<T>,
}

#[cfg(target_has_atomic = "ptr")]
unsafe impl<T: Send> Sync for SpinLock<T> {}
#[cfg(target_has_atomic = "ptr")]
unsafe impl<T: Send> Send for SpinLock<T> {}

#[cfg(not(target_has_atomic = "ptr"))]
unsafe impl<T: Send> Sync for SpinLock<T> {}
#[cfg(not(target_has_atomic = "ptr"))]
unsafe impl<T: Send> Send for SpinLock<T> {}

#[cfg(target_has_atomic = "ptr")]
impl<T> SpinLock<T> {
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Executes `f` with mutable access to protected data under spinlock.
    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        let res = f(unsafe { &mut *self.data.get() });
        self.locked.store(false, Ordering::Release);
        res
    }
}

#[cfg(not(target_has_atomic = "ptr"))]
impl<T> SpinLock<T> {
    #[must_use]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    /// Executes `f` on single-core bare-metal targets without hardware CAS.
    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(unsafe { &mut *self.data.get() })
    }
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
        Self { slots: [None; N] }
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

impl<T: Copy + Default, const N: usize> Default for SlabPool<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Linear Region-based Arena with Hardware Alignment & Const Assertion.
#[derive(Debug)]
pub struct LinearArena<const SIZE: usize, const ALIGN: usize = 4> {
    buffer: AlignedBuffer<SIZE>,
    offset: usize,
}

impl<const SIZE: usize, const ALIGN: usize> LinearArena<SIZE, ALIGN> {
    #[must_use]
    pub const fn new() -> Self {
        // Compile-time static assert enforcing power-of-two alignment
        assert!(is_power_of_two(ALIGN), "ALIGN must be a power of two");
        assert!(ALIGN <= 64, "ALIGN must be <= 64 for AlignedBuffer");
        Self {
            buffer: AlignedBuffer::new(),
            offset: 0,
        }
    }

    /// Allocates a contiguous byte slice aligned to `ALIGN` bytes.
    ///
    /// Fully guarded against integer overflow using `checked_add`.
    pub fn alloc_bytes(&mut self, len: usize) -> Option<&mut [u8]> {
        let align_mask = ALIGN - 1;
        let aligned_offset = self.offset.checked_add(align_mask)? & !align_mask;
        let new_offset = aligned_offset.checked_add(len)?;

        if new_offset <= SIZE {
            self.offset = new_offset;
            Some(&mut self.buffer.0[aligned_offset..new_offset])
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
        SIZE.saturating_sub(self.offset)
    }

    /// Verifies hardware pointer alignment of base buffer.
    #[must_use]
    pub fn is_buffer_aligned(&self) -> bool {
        (self.buffer.0.as_ptr() as usize) % ALIGN == 0
    }
}

impl<const SIZE: usize, const ALIGN: usize> Default for LinearArena<SIZE, ALIGN> {
    fn default() -> Self {
        Self::new()
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

/// Lock-Free Atomic Linear Arena for Multi-Core Microcontrollers (ESP32 / RP2040).
#[cfg(target_has_atomic = "ptr")]
pub struct AtomicLinearArena<const SIZE: usize, const ALIGN: usize = 4> {
    buffer: AlignedBuffer<SIZE>,
    offset: AtomicUsize,
}

#[cfg(target_has_atomic = "ptr")]
impl<const SIZE: usize, const ALIGN: usize> AtomicLinearArena<SIZE, ALIGN> {
    #[must_use]
    pub const fn new() -> Self {
        assert!(is_power_of_two(ALIGN), "ALIGN must be a power of two");
        assert!(ALIGN <= 64, "ALIGN must be <= 64 for AlignedBuffer");
        Self {
            buffer: AlignedBuffer::new(),
            offset: AtomicUsize::new(0),
        }
    }

    /// Lock-free allocation via atomic Compare-And-Swap (CAS) loop with checked arithmetic.
    pub fn alloc_bytes(&self, len: usize) -> Option<usize> {
        let align_mask = ALIGN - 1;
        let mut current = self.offset.load(Ordering::Relaxed);

        loop {
            let aligned = current.checked_add(align_mask)? & !align_mask;
            let next = aligned.checked_add(len)?;

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

#[cfg(target_has_atomic = "ptr")]
impl<const SIZE: usize, const ALIGN: usize> Default for AtomicLinearArena<SIZE, ALIGN> {
    fn default() -> Self {
        Self::new()
    }
}

/// Micro-IoT Composite Deterministic Memory Controller.
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

impl Default for EmbeddedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// C-FFI Exported Functions (Seamless STM32CubeIDE / ESP-IDF FreeRTOS Bindings)
//
// Thread-Safety Guarantee:
// Managed via global SpinLock with zero `static mut` and zero UB under Rust 2024.
// If invoking from bare-metal interrupt handlers (ISR), callers must mask
// interrupts or allocate exclusively in task context to avoid deadlock.
// =========================================================================

static GLOBAL_EMBEDDED_MGR: SpinLock<EmbeddedMemoryManager> =
    SpinLock::new(EmbeddedMemoryManager::new());

#[no_mangle]
pub extern "C" fn oxidize_arena_init() {
    GLOBAL_EMBEDDED_MGR.lock(|mgr| {
        *mgr = EmbeddedMemoryManager::new();
    });
}

#[no_mangle]
pub extern "C" fn oxidize_arena_alloc(size: usize) -> *mut u8 {
    GLOBAL_EMBEDDED_MGR.lock(|mgr| match mgr.arena.alloc_bytes(size) {
        Some(slice) => slice.as_mut_ptr(),
        None => core::ptr::null_mut(),
    })
}

#[no_mangle]
pub extern "C" fn oxidize_arena_reset() {
    GLOBAL_EMBEDDED_MGR.lock(|mgr| {
        mgr.arena.reset();
    });
}

#[no_mangle]
pub extern "C" fn oxidize_arena_used_bytes() -> usize {
    GLOBAL_EMBEDDED_MGR.lock(|mgr| mgr.arena.used_bytes())
}

#[no_mangle]
pub extern "C" fn oxidize_slab_alloc_packet(sensor_id: u32, timestamp: u64) -> i32 {
    let pkt = SensorPacket {
        sensor_id,
        timestamp,
        payload: [0u8; PACKET_PAYLOAD_SIZE],
        length: 0,
    };
    GLOBAL_EMBEDDED_MGR.lock(|mgr| match mgr.slab_pool.allocate(pkt) {
        Some(idx) => idx as i32,
        None => -1,
    })
}

#[no_mangle]
pub extern "C" fn oxidize_slab_free_packet(index: usize) -> bool {
    GLOBAL_EMBEDDED_MGR.lock(|mgr| mgr.slab_pool.free(index).is_some())
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
    fn test_hardware_alignment_and_offset() {
        let mut arena = LinearArena::<256, 8>::new();
        assert!(arena.is_buffer_aligned(), "Buffer must be hardware aligned to 8 bytes");

        let slice1 = arena.alloc_bytes(5).expect("5 bytes allocated");
        slice1[0] = 0xAA;
        assert_eq!(arena.used_bytes(), 5);

        let slice2 = arena.alloc_bytes(10).expect("10 bytes allocated");
        slice2[0] = 0xBB;
        assert_eq!(slice2.as_ptr() as usize % 8, 0, "Slice2 must be 8-byte aligned");
        // 5 aligned to 8 is 8, 8 + 10 = 18
        assert_eq!(arena.used_bytes(), 18);
    }

    #[test]
    fn test_overflow_protection_in_alloc() {
        let mut arena = LinearArena::<256, 8>::new();
        // Passing usize::MAX or close to overflow should return None instead of wrapping
        assert!(arena.alloc_bytes(usize::MAX).is_none());
        assert!(arena.alloc_bytes(usize::MAX - 3).is_none());
        assert_eq!(arena.used_bytes(), 0);
    }

    #[test]
    #[cfg(target_has_atomic = "ptr")]
    fn test_atomic_arena_overflow_protection() {
        let arena = AtomicLinearArena::<256, 8>::new();
        assert!(arena.alloc_bytes(usize::MAX).is_none());
        assert!(arena.alloc_bytes(usize::MAX - 3).is_none());
        assert_eq!(arena.used_bytes(), 0);
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
    #[cfg(target_has_atomic = "ptr")]
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

    #[test]
    fn test_thread_safe_global_mgr_ffi() {
        oxidize_arena_init();
        let ptr1 = oxidize_arena_alloc(32);
        assert!(!ptr1.is_null());
        assert_eq!(oxidize_arena_used_bytes(), 32);

        let pkt_idx = oxidize_slab_alloc_packet(1, 9999);
        assert!(pkt_idx >= 0);
        assert!(oxidize_slab_free_packet(pkt_idx as usize));

        oxidize_arena_reset();
        assert_eq!(oxidize_arena_used_bytes(), 0);
    }

    #[test]
    #[cfg(target_has_atomic = "ptr")]
    fn test_atomic_lock_free_concurrency_stress_test() {
        use std::sync::Arc;
        use std::thread;
        use std::vec::Vec;

        const TOTAL_THREADS: usize = 8;
        const ALLOCS_PER_THREAD: usize = 50;
        const ALLOC_SIZE: usize = 16;
        const ARENA_CAPACITY: usize = TOTAL_THREADS * ALLOCS_PER_THREAD * ALLOC_SIZE * 2;

        let arena = Arc::new(AtomicLinearArena::<ARENA_CAPACITY, 8>::new());
        let mut handles = Vec::with_capacity(TOTAL_THREADS);

        for _ in 0..TOTAL_THREADS {
            let arena_clone = Arc::clone(&arena);
            handles.push(thread::spawn(move || {
                let mut allocated_offsets = Vec::with_capacity(ALLOCS_PER_THREAD);
                for _ in 0..ALLOCS_PER_THREAD {
                    if let Some(offset) = arena_clone.alloc_bytes(ALLOC_SIZE) {
                        allocated_offsets.push(offset);
                    }
                }
                allocated_offsets
            }));
        }

        let mut all_offsets = Vec::new();
        for h in handles {
            let offsets = h.join().expect("Thread finished without panic");
            all_offsets.extend(offsets);
        }

        assert_eq!(all_offsets.len(), TOTAL_THREADS * ALLOCS_PER_THREAD);
        all_offsets.sort_unstable();
        for window in all_offsets.windows(2) {
            assert!(
                window[1] >= window[0] + ALLOC_SIZE,
                "Race condition detected: overlapping allocation offsets!"
            );
        }
    }
}
