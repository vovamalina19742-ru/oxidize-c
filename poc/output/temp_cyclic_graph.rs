//! Oxidize-C Transpiled Module
//! Source: cyclic_graph.c
//! Pattern: Generational Arena (SlotMap with NodeKey)

/// A versioned index key referencing a node in the generational arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeKey {
    pub index: usize,
    pub generation: u32,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub value: i32,
    pub next: Option<NodeKey>,
    pub prev: Option<NodeKey>,
    generation: u32,
    is_alive: bool,
}

/// A 100% Safe, cache-friendly Doubly-Linked List backed by a Generational Arena.
#[derive(Debug, Clone, Default)]
pub struct DoublyLinkedList {
    nodes: Vec<Node>,
    free_indices: Vec<usize>,
    head: Option<NodeKey>,
    tail: Option<NodeKey>,
    size: usize,
}

impl DoublyLinkedList {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_back(&mut self, value: i32) -> NodeKey {
        let key = if let Some(idx) = self.free_indices.pop() {
            let node = &mut self.nodes[idx];
            node.value = value;
            node.next = None;
            node.prev = self.tail;
            node.is_alive = true;
            NodeKey {
                index: idx,
                generation: node.generation,
            }
        } else {
            let idx = self.nodes.len();
            let key = NodeKey {
                index: idx,
                generation: 1,
            };
            self.nodes.push(Node {
                value,
                next: None,
                prev: self.tail,
                generation: 1,
                is_alive: true,
            });
            key
        };

        if let Some(tail_key) = self.tail {
            if let Some(tail_node) = self.get_mut_node(tail_key) {
                tail_node.next = Some(key);
            }
        } else {
            self.head = Some(key);
        }

        self.tail = Some(key);
        self.size += 1;
        key
    }

    #[must_use]
    pub fn get(&self, key: NodeKey) -> Option<&i32> {
        self.nodes.get(key.index).and_then(|n| {
            if n.is_alive && n.generation == key.generation {
                Some(&n.value)
            } else {
                None
            }
        })
    }

    fn get_mut_node(&mut self, key: NodeKey) -> Option<&mut Node> {
        self.nodes.get_mut(key.index).and_then(|n| {
            if n.is_alive && n.generation == key.generation {
                Some(n)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.size
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doubly_linked_list_arena() {
        let mut list = DoublyLinkedList::new();
        let k1 = list.push_back(100);
        let k2 = list.push_back(200);
        let k3 = list.push_back(300);

        assert_eq!(list.len(), 3);
        assert_eq!(list.get(k1), Some(&100));
        assert_eq!(list.get(k2), Some(&200));
        assert_eq!(list.get(k3), Some(&300));
    }
}
