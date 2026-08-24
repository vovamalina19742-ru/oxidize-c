#include <stdio.h>
#include <stdlib.h>

// Doubly-linked node (Self-referential cyclic graph structure)
typedef struct Node {
    int value;
    struct Node *next;
    struct Node *prev;
} Node;

// Doubly-linked list container
typedef struct {
    Node *head;
    Node *tail;
    size_t size;
} DoublyLinkedList;

DoublyLinkedList* list_create(void) {
    DoublyLinkedList *list = (DoublyLinkedList*)malloc(sizeof(DoublyLinkedList));
    if (!list) return NULL;
    list->head = NULL;
    list->tail = NULL;
    list->size = 0;
    return list;
}

void list_push_back(DoublyLinkedList *list, int value) {
    if (!list) return;
    Node *node = (Node*)malloc(sizeof(Node));
    if (!node) return;
    node->value = value;
    node->next = NULL;
    node->prev = list->tail;

    if (list->tail) {
        list->tail->next = node;
    } else {
        list->head = node;
    }
    list->tail = node;
    list->size++;
}

void list_free(DoublyLinkedList *list) {
    if (!list) return;
    Node *curr = list->head;
    while (curr) {
        Node *next = curr->next;
        free(curr);
        curr = next;
    }
    free(list);
}
