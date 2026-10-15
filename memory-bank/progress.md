# Oxidize-C: Progress Tracker

## Что готово и работает (What Works)
- Базовая структура рабочего пространства (Cargo workspace).
- Парсер C AST и каркас генератора кода Rust.
- Контракты типов данных.

## Что сломано / требует исправления (Remediation Backlog)
- [ ] **1. Embedded Allocator:**
  - [ ] Добавить аппаратное выравнивание `#[repr(align(8))]` / `#[repr(align(16))]`.
  - [ ] `checked_add` для вычисления `aligned_offset + len`.
  - [ ] Замена `static mut` на безопасные примитивы синхронизации (`AtomicPtr`, `Spinlock` или Safe Wrapper).
- [ ] **2. CLI & Verifier:**
  - [ ] Реализовать боевую команду `translate` в `crates/oxidize-cli`.
  - [ ] Устранить Fail-Open в `oxidize-verifier` при отсутствии `rustc`.
  - [ ] Подключить запуск тестов `rustc --test`.
- [ ] **3. Differential Testing Oracle:**
  - [ ] Удалить захардкоженный `status: "PASS"` и счётчики `8/7`.
  - [ ] Интегрировать реальный запуск gcc/clang и сравнение вывода с Rust.
