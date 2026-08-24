/**
 * @file oxidize_allocator.h
 * @brief Oxidize-C Deterministic Embedded Allocator C-FFI Header
 * @note Compatible with STM32CubeIDE, Keil MDK, ESP-IDF (FreeRTOS), ARM GCC, and Clang.
 */

#ifndef OXIDIZE_ALLOCATOR_H
#define OXIDIZE_ALLOCATOR_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Initializes or resets the global embedded memory manager.
 */
void oxidize_arena_init(void);

/**
 * @brief Allocates an 8-byte aligned memory chunk from the static arena.
 * @param size Number of bytes to allocate.
 * @return Pointer to allocated memory, or NULL if arena is exhausted.
 */
uint8_t* oxidize_arena_alloc(size_t size);

/**
 * @brief Performs instant O(1) bulk deallocation of the entire arena region.
 */
void oxidize_arena_reset(void);

/**
 * @brief Returns the current number of allocated bytes in the arena.
 */
size_t oxidize_arena_used_bytes(void);

/**
 * @brief Allocates a fixed-size sensor telemetry packet from the slab pool in O(1) time.
 * @param sensor_id Unique sensor identifier.
 * @param timestamp Epoch timestamp in milliseconds/microseconds.
 * @return Slot index (0..15), or -1 if pool is full.
 */
int32_t oxidize_slab_alloc_packet(uint32_t sensor_id, uint64_t timestamp);

/**
 * @brief Releases a packet back to the slab pool in O(1) time.
 * @param index Slot index to free.
 * @return true if successfully freed, false if index was out of bounds or empty.
 */
bool oxidize_slab_free_packet(size_t index);

#ifdef __cplusplus
}
#endif

#endif /* OXIDIZE_ALLOCATOR_H */
