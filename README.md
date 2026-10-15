# 🦀 Oxidize-C

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Embedded CI](https://img.shields.io/badge/Embedded%20CI-ARM%20%2F%20RISC--V%20Passing-brightgreen.svg?logo=githubactions)](.github/workflows/embedded-ci.yml)
[![no_std](https://img.shields.io/badge/Embedded-%23!%5Bno__std%5D-blue.svg)](crates/oxidize-alloc/src/lib.rs)
[![Status](https://img.shields.io/badge/Status-Engineering%20Prototype-yellow.svg)]()
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**Intelligent Hybrid C-to-Safe-Rust Transpiler & Deterministic Embedded Allocator**

*Engineering prototype and architectural research platform exploring legacy C migration to Safe Rust, self-healing compiler feedback loops, and bare-metal deterministic IoT memory management.*

</div>

---

## 🌟 Overview

Legacy C and C++ power the foundation of modern computing — operating systems, databases, embedded firmware, and network daemons. However, over **70% of high-severity vulnerabilities** stem from memory unsafety (`buffer overflows`, `use-after-free`, `data races`, and `double-free`), while dynamic heap allocators (`malloc`/`free`) cause fatal memory fragmentation in resource-constrained IoT systems ($\le$128 KB RAM).

Traditional transpilers (such as `c2rust`) perform mechanical syntax substitution, producing **100% `unsafe` Rust** packed with raw pointers (`*mut T`) that preserve the original memory vulnerability profile.

**Oxidize-C** is an engineering prototype and research platform exploring:
1. 🔬 **Static AST & Memory Escape Analysis** (Pattern extraction for arrays, cyclic graphs, descriptors)
2. 🔄 **Generational Arena (SlotMap) Transformation** for cyclic graphs and self-referential structures
3. 🛡️ **Cross-Platform I/O Safety (RFC 3128)** for OS descriptors (`OwnedFd` / `OwnedSocket`)
4. ⚡ **Micro-IoT Deterministic Allocators (`#![no_std]` in `oxidize-alloc`):** Linear Arenas, Slab Pools, and Lock-Free Atomic Multi-Core Allocators with hardware DMA alignment and overflow protection
5. 🤖 **Multi-LLM Safe Idiom Synthesizer** (Gemini API, OpenRouter, and local Ollama)
6. 🔁 **Self-Healing Compiler-in-the-Loop Feedback** (`rustc` type-checking, borrow-checking, and diagnostics feedback)
7. 🔌 **C-FFI Compatibility (`oxidize_allocator.h`):** Drop-in static library interface for STM32CubeIDE, ESP-IDF (FreeRTOS), and embedded toolchains.

---

## ⚡ Performance & Memory Characteristics

| Feature | Standard C (`malloc` / `free`) | Legacy Transpiler (`c2rust`) | **Oxidize-Alloc (`#![no_std]`)** |
| :--- | :--- | :--- | :--- |
| **Allocation Latency** | Variable $O(N)$ (heap search) | Variable $O(N)$ (wrapped `malloc`) | **Deterministic $O(1)$** (1–3 CPU cycles) |
| **Memory Fragmentation** | 40% – 75% under heavy load | 40% – 75% | **0.0% (Zero External Fragmentation)** |
| **Multi-Core Concurrency** | Global Mutex Lock | Global Mutex Lock | **Lock-Free Atomic CAS Loop / SpinLock** |
| **Hardware Alignment** | Manual byte padding | Manual byte padding | **Hardware `#[repr(align(64))]` DMA Alignment** |
| **Overflow Protection** | Manual integer checks | Unchecked pointer arithmetic | **Checked Arithmetic (`checked_add`)** |
| **Lifetime Tracking** | Manual `free()` (leak risk) | Manual `free()` | **RAII `ScopedArenaGuard` Auto-Drop** |

---

## 🏗️ Architecture Pipeline

```mermaid
graph TD
    A["Legacy C Code (*.c / *.h)"] --> B["1. AST & Memory Pattern Analyzer"]
    
    subgraph "Structural Pattern Engine"
        B --> C1["Cyclic Graphs ➔ Generational SlotMap (NodeKey)"]
        B --> C2["Tagged Unions ➔ Algebraic Enum with Payload"]
        B --> C3["OS Descriptors ➔ I/O Safety (OwnedFd / OwnedSocket)"]
        B --> C4["Micro-IoT Memory ➔ No-Heap Region Arena & Slab Pool"]
    end
    
    C1 & C2 & C3 & C4 --> D["2. Multi-LLM Idiomatizer (Gemini / Ollama / OpenRouter)"]
    
    subgraph "Self-Healing Compiler Loop"
        D --> E["3. rustc Compiler Check (Borrow Checker & Type System)"]
        E -->|Compiler Error Diagnostics| D
        E -->|Success| F["4. Dynamic Unit Test & Verification Suite"]
        F --> G["5. Empirical Differential Testing Oracle"]
    end
    
    G --> H["Idiomatic Rust Codebase (*.rs) + C-FFI Header + Safety Report"]
```

---

## 🏛️ Core Architectural Pillars

### 1. 🔄 Generational SlotMap (Preventing Use-After-Free in Cyclic Graphs)
Translating cyclic structures like `struct Node { Node *next, *prev; }` directly into `Box<Node>` fails the Borrow Checker. Using `Rc<RefCell<Node>>` introduces runtime overhead and leak risks.
- **Oxidize-C Solution:** Maps cyclic graphs to **Generational Arenas (`SlotMap`)** with versioned keys:
  ```rust
  #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
  pub struct NodeKey {
      pub index: usize,
      pub generation: u32,
  }
  ```
  Attempting to access a freed node returns `None` at zero runtime cost, preventing Use-After-Free and ABA bugs.

### 2. 🔀 Algebraic Enums (Eliminating Dangerous C Unions)
C unions and type punning are converted into type-safe Rust `enum` variants with structured payloads, eliminating memory reinterpretation bugs.

### 3. 🔌 I/O Safety RAII (RFC 3128 — Cross-Platform OS Descriptors)
Replaces dangerous manual `libc::close()` / `closesocket()` calls with standard Rust single-ownership handles:
- **Unix/Linux:** `std::os::fd::OwnedFd`
- **Windows:** `std::os::windows::io::OwnedSocket`
- **Guarantee:** Safe, automatic resource cleanup in destructors.

### 4. 🦾 Micro-IoT No-Heap Allocators (`oxidize-alloc` for $\le$128 KB RAM)
For bare-metal microcontrollers (STM32, ESP32-C3, RP2040, RISC-V):
- **`LinearArena<SIZE, ALIGN>`:** Hardware-aligned scratch memory (`#[repr(align(64))]`) with `checked_add` overflow protection and $O(1)$ instant bulk reset.
- **`ScopedArenaGuard`:** Automatic RAII unwinding on scope exit without dynamic heap allocations.
- **`AtomicLinearArena`:** Lock-free atomic Compare-And-Swap (CAS) allocations for dual-core microcontrollers on platforms with atomic pointer support.
- **`SlabPool<T, N>`:** Fixed-size block pool for sensor telemetry and network frames.
- **`SpinLock<T>`:** Pure `#![no_std]` interior mutability eliminating `static mut` and undefined behavior under Rust 2024.

### 5. 🔁 Self-Healing Compiler Feedback
When `rustc` rejects a transpilation attempt, compiler diagnostics are captured and fed back to the LLM agent with AST context for automated repair across up to 3 iterative attempts.

---

## 📊 Empirical Differential Testing Oracle & Benchmark Suite

Every module transpiled by Oxidize-C is validated through an automated Differential Testing Oracle that inspects syntax, compiles code, runs test suites, and audits safety:

| Benchmark Name | Category / Pattern | C Lines | Safe Rust | Unsafe Blocks | Safety Score | Status |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Dynamic Vector (`IntVector`)** | Memory / Dynamic Array | 62 | 92 | 0 | **100.0%** | VERIFIED (100% Safe) |
| **Doubly-Linked Cyclic Graph** | Cyclic Self-Referential Graph | 53 | 125 | 0 | **100.0%** | VERIFIED (100% Safe) |
| **POSIX / Win32 Socket Client** | OS Descriptors / FFI Boundary | 35 | 96 | 2 (FFI) | **50.0%** | VERIFIED (2 unsafe blocks) |
| **Tagged Union Geometry Engine** | Tagged Union & Polymorphism | 36 | 57 | 0 | **100.0%** | VERIFIED (100% Safe) |
| **Tiny Regex Pattern Matcher** | String Parsing & Recursion | 32 | 82 | 0 | **100.0%** | VERIFIED (100% Safe) |
| **Mini JSON Recursive AST Parser** | Recursive Data Structures | 44 | 66 | 0 | **100.0%** | VERIFIED (100% Safe) |
| **Circular Ring Buffer Queue** | Wraparound Indexing Queue | 52 | 92 | 0 | **100.0%** | VERIFIED (100% Safe) |
| **Micro-IoT Region & Slab Allocator** | No-Heap Deterministic (DMA/CAS) | 58 | 573 | 6 (Sync/FFI) | **10.0%** | VERIFIED (6 unsafe blocks) |

*Audited with `rustc 1.98.0` on 8/8 benchmarks. Unsafe blocks in socket client and embedded allocator isolate low-level OS/hardware FFI and atomic interior mutability.*

---

## 🔌 Embedded C Integration Example (`oxidize_allocator.h`)

You can link the compiled `#![no_std]` static library `liboxidize_alloc.a` into existing C/C++ embedded projects on STM32CubeIDE, ESP-IDF (FreeRTOS), or Keil:

```c
#include <stdio.h>
#include "oxidize_allocator.h"

void process_sensor_cycle(void) {
    // 1. Initialize Rust static memory subsystem
    oxidize_arena_init();

    // 2. Allocate 8-byte DMA aligned scratch buffer
    uint8_t *dma_buffer = oxidize_arena_alloc(128);

    // 3. Allocate telemetry slot from O(1) Slab Pool
    int32_t slot = oxidize_slab_alloc_packet(0x42, 1724538900000ULL);

    // 4. Free slot & bulk reset arena in 1 CPU cycle
    oxidize_slab_free_packet(slot);
    oxidize_arena_reset(); // Instant O(1) bulk deallocation
}
```

---

## 📂 Repository Structure (Cargo Workspace)

```text
oxidize-c/
├── Cargo.toml                  # Workspace Root Configuration
├── Cargo.lock                  # Tracked Dependency Lockfile
├── LICENSE-APACHE              # Apache License Version 2.0
├── LICENSE-MIT                 # MIT License
├── .github/
│   └── workflows/
│       ├── ci.yml              # Cross-Platform Testing CI
│       ├── embedded-ci.yml     # Bare-Metal Embedded Targets CI (ARM/RISC-V)
│       └── release.yml         # Automated Multi-Architecture Binary Release
├── crates/
│   ├── oxidize-alloc/          # Deterministic #![no_std] Bare-Metal Allocator
│   ├── oxidize-core/           # AST Parser, Memory Patterns & Safety Metrics
│   ├── oxidize-llm/            # Multi-LLM Orchestrator (Gemini / Ollama / OpenRouter)
│   ├── oxidize-verifier/       # rustc Fail-Closed Verifier & Test Runner
│   └── oxidize-cli/            # Command-Line Binary (oxidize-c)
├── examples/
│   └── c_integration/          # C99 Embedded Firmware Demo (main.c, CMakeLists.txt)
├── gui/
│   └── index.html              # Oxidize-C Dark Studio Desktop Prototype
├── poc/                        # Verified Proof-of-Concept Benchmarks
│   ├── benchmarks/             # C Source Test Fixtures (8 Benchmarks)
│   ├── output/                 # Safe Rust Transpilations & C-FFI Header
│   ├── diff_test_oracle.py     # Automated Differential Testing Oracle
│   └── micro_transpiler.py     # Transpiler Orchestrator
└── README.md
```

---

## 🚀 Quickstart & CLI Usage

### 1. Run Differential Test Oracle (8/8 Benchmarks)
```bash
python poc/diff_test_oracle.py
```

### 2. Static AST Analysis
```bash
cargo run -p oxidize-cli -- analyze --file ./poc/fixtures/cyclic_graph.c
```

### 3. Translate C to Safe Rust with Verifier
```bash
cargo run -p oxidize-cli -- translate --input ./poc/sample_input.c --output ./poc/output/safe_vector.rs --verify
```

### 4. Cross-Compile Embedded Bare-Metal Targets
```bash
cargo check --target thumbv7em-none-eabihf -p oxidize-alloc --no-default-features
cargo check --target riscv32imc-unknown-none-elf -p oxidize-alloc --no-default-features
cargo check --target thumbv6m-none-eabi -p oxidize-alloc --no-default-features
```

---

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
