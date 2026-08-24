// Embedded Micro-IoT Packet Handler with Region-based Memory in C
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

#define SENSOR_POOL_SIZE 16
#define PACKET_PAYLOAD_SIZE 64

typedef struct {
    uint32_t sensor_id;
    uint64_t timestamp;
    uint8_t payload[PACKET_PAYLOAD_SIZE];
    uint16_t length;
    bool in_use;
} SensorPacket;

typedef struct {
    SensorPacket pool[SENSOR_POOL_SIZE];
    uint8_t arena_buffer[1024];
    size_t arena_offset;
} EmbeddedMemoryManager;

EmbeddedMemoryManager g_mem_mgr;

void mem_init(EmbeddedMemoryManager *mgr) {
    memset(mgr, 0, sizeof(EmbeddedMemoryManager));
}

SensorPacket* slab_alloc_packet(EmbeddedMemoryManager *mgr) {
    for (int i = 0; i < SENSOR_POOL_SIZE; i++) {
        if (!mgr->pool[i].in_use) {
            mgr->pool[i].in_use = true;
            return &mgr->pool[i];
        }
    }
    return NULL; // Out of fixed blocks (Deterministic O(1))
}

void slab_free_packet(SensorPacket *pkt) {
    if (pkt) {
        pkt->in_use = false;
    }
}

void* arena_alloc(EmbeddedMemoryManager *mgr, size_t size) {
    // 4-byte align
    size_t aligned_size = (size + 3) & ~3;
    if (mgr->arena_offset + aligned_size <= sizeof(mgr->arena_buffer)) {
        void *ptr = &mgr->arena_buffer[mgr->arena_offset];
        mgr->arena_offset += aligned_size;
        return ptr;
    }
    return NULL; // Arena overflow
}

void arena_reset(EmbeddedMemoryManager *mgr) {
    mgr->arena_offset = 0; // O(1) Instant Bulk Deallocation
}
