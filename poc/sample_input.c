#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

// Dynamic integer array structure
typedef struct {
    int *data;
    size_t capacity;
    size_t length;
} IntVector;

// Initialize a new vector with initial capacity
IntVector* vector_create(size_t initial_capacity) {
    IntVector *vec = (IntVector*)malloc(sizeof(IntVector));
    if (!vec) return NULL;

    vec->data = (int*)malloc(initial_capacity * sizeof(int));
    if (!vec->data) {
        free(vec);
        return NULL;
    }

    vec->capacity = initial_capacity;
    vec->length = 0;
    return vec;
}

// Push an element to the vector (reallocates if full)
bool vector_push(IntVector *vec, int value) {
    if (!vec) return false;

    if (vec->length >= vec->capacity) {
        size_t new_cap = vec->capacity == 0 ? 4 : vec->capacity * 2;
        int *new_data = (int*)realloc(vec->data, new_cap * sizeof(int));
        if (!new_data) return false;

        vec->data = new_data;
        vec->capacity = new_cap;
    }

    vec->data[vec->length++] = value;
    return true;
}

// Get element at index safely
bool vector_get(const IntVector *vec, size_t index, int *out_value) {
    if (!vec || !out_value || index >= vec->length) {
        return false;
    }
    *out_value = vec->data[index];
    return true;
}

// Free all allocated memory
void vector_free(IntVector *vec) {
    if (vec) {
        if (vec->data) {
            free(vec->data);
        }
        free(vec);
    }
}
