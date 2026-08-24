//! Architectural Memory Patterns & Safety Classifications

use serde::{Deserialize, Serialize};

/// Architectural transformation pattern assigned to a C construct.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryPattern {
    /// Standard Rust struct without cyclic references
    StandardStruct,
    /// Memory managed by malloc/free/realloc -> Vec<T> / Box<T>
    RaiiOwnership,
    /// Cyclic / self-referential graph or doubly-linked list -> Generational SlotMap
    GenerationalSlotMap,
    /// Tagged union / type discriminant -> Algebraic Enum
    AlgebraicEnum,
    /// OS file / socket descriptor -> I/O Safety RAII (OwnedFd / OwnedSocket)
    IoSafetyRaii,
}

impl MemoryPattern {
    pub fn description(&self) -> &'static str {
        match self {
            Self::StandardStruct => "Standard Rust Struct with immutable/mutable borrowing",
            Self::RaiiOwnership => "Automatic RAII Ownership (Vec<T> / Box<T>)",
            Self::GenerationalSlotMap => "Generational Arena (SlotMap with versioned NodeKey)",
            Self::AlgebraicEnum => "Rust Tagged Algebraic Enum with associated payload",
            Self::IoSafetyRaii => "I/O Safety RAII (std::os::fd::OwnedFd / std::os::windows::io::OwnedSocket)",
        }
    }
}
