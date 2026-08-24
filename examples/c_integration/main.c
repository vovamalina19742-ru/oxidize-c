/**
 * @file main.c
 * @brief End-to-End Embedded C Firmware Integration Example using Oxidize-C Allocator
 * @details Demonstrates linking with liboxidize_c.a on STM32 / FreeRTOS / ESP32.
 */

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "../../poc/output/oxidize_allocator.h"

int main(void) {
    printf("=====================================================\n");
    printf("   OXIDIZE-C: EMBEDDED C-FFI INTEGRATION TEST (C99)  \n");
    printf("=====================================================\n");

    // 1. Initialize Rust static memory subsystem
    oxidize_arena_init();
    printf("[+] Memory subsystem initialized. Used bytes: %zu\n", oxidize_arena_used_bytes());

    // 2. Allocate 8-byte aligned scratch buffer from Rust Linear Arena
    uint8_t *scratch = oxidize_arena_alloc(128);
    assert(scratch != NULL);
    memset(scratch, 0x5A, 128);
    printf("[+] Allocated 128 bytes from Rust Linear Arena. Used bytes: %zu\n", oxidize_arena_used_bytes());

    // 3. Allocate a deterministic telemetry packet from the Fixed-size Slab Pool
    int32_t slot = oxidize_slab_alloc_packet(0x42, 1724538900000ULL);
    assert(slot >= 0);
    printf("[+] Allocated sensor telemetry slot #%d from Slab Pool (O(1))\n", slot);

    // 4. Free the slot
    bool freed = oxidize_slab_free_packet((size_t)slot);
    assert(freed == true);
    printf("[+] Freed slot #%d back to Slab Pool\n", slot);

    // 5. Instant bulk reset of the entire arena (1 CPU cycle)
    oxidize_arena_reset();
    printf("[+] Arena reset performed. Used bytes: %zu\n", oxidize_arena_used_bytes());
    assert(oxidize_arena_used_bytes() == 0);

    printf("[✔] ALL C-FFI INVARIANTS VERIFIED SUCCESSFULLY (100%% PASS)!\n");
    return 0;
}
