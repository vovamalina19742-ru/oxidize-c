# Contributing to Oxidize-C 🦀

Thank you for your interest in contributing to **Oxidize-C**!

## How to Add New C Benchmarks

1. Place your C source file in `poc/benchmarks/<your_file>.c`.
2. Run the AST analyzer to identify its structural memory patterns:
   ```bash
   python poc/ast_analyzer.py
   ```
3. Generate the 100% Safe Rust transpilation in `poc/output/<your_file>.rs`.
4. Add the entry to `BENCHMARKS` list in `poc/diff_test_oracle.py`.
5. Run the verification test suite:
   ```bash
   python poc/diff_test_oracle.py
   ```

## Development Guidelines

- **Zero Unsafe:** All high-level translations must avoid `unsafe` blocks in business logic.
- **I/O Safety:** OS file descriptors and sockets must use `OwnedFd` (Unix) and `OwnedSocket` (Windows).
- **Generational SlotMap:** Cyclic graphs must use versioned keys to prevent Use-After-Free.
