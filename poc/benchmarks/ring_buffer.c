#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

typedef struct {
    int *buffer;
    size_t capacity;
    size_t head;
    size_t tail;
    size_t count;
} RingBuffer;

RingBuffer* rb_create(size_t capacity) {
    if (capacity == 0) return NULL;
    RingBuffer *rb = (RingBuffer*)malloc(sizeof(RingBuffer));
    if (!rb) return NULL;

    rb->buffer = (int*)malloc(capacity * sizeof(int));
    if (!rb->buffer) {
        free(rb);
        return NULL;
    }

    rb->capacity = capacity;
    rb->head = 0;
    rb->tail = 0;
    rb->count = 0;
    return rb;
}

bool rb_push(RingBuffer *rb, int item) {
    if (!rb || rb->count == rb->capacity) return false;
    rb->buffer[rb->tail] = item;
    rb->tail = (rb->tail + 1) % rb->capacity;
    rb->count++;
    return true;
}

bool rb_pop(RingBuffer *rb, int *out_item) {
    if (!rb || rb->count == 0 || !out_item) return false;
    *out_item = rb->buffer[rb->head];
    rb->head = (rb->head + 1) % rb->capacity;
    rb->count--;
    return true;
}

void rb_free(RingBuffer *rb) {
    if (rb) {
        if (rb->buffer) free(rb->buffer);
        free(rb);
    }
}
