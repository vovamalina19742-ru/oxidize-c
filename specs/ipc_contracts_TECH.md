# IPC Контракты — Oxidize-C Studio (Tauri 2.0)

## 1. Команда: `analyze_c_source`
**Описание:** Выполняет синтаксический AST-анализ сишного кода и возвращает классификацию структур, паттернов памяти и системных вызовов.

**Входные данные (TypeScript):**
```typescript
interface AnalyzeCSourceInput {
  source_code: string;
  filename: string;
}
```

**Выходные данные (TypeScript):**
```typescript
interface StructField {
  name: string;
  type_str: string;
  is_pointer: boolean;
  is_self_ref: boolean;
}

interface StructInfo {
  name: string;
  fields: StructField[];
  is_cyclic: boolean;
  has_union: boolean;
  has_os_descriptor: boolean;
  pattern: 'StandardStruct' | 'RaiiOwnership' | 'GenerationalSlotMap' | 'AlgebraicEnum' | 'IoSafetyRaii';
}

interface AstReport {
  filename: string;
  total_lines: number;
  has_malloc: boolean;
  has_free: boolean;
  has_realloc: boolean;
  has_os_socket: boolean;
  structs: StructInfo[];
  suggested_patterns: string[];
}
```

---

## 2. Команда: `transpile_c_to_rust`
**Описание:** Запускает трансляцию C в Safe Rust через AI-оркестратор с передачей AST-директив и автоматическим циклом самоисправления ошибок `rustc`.

**Входные данные (TypeScript):**
```typescript
interface TranspileInput {
  source_code: string;
  filename: string;
  model: 'gemini-1.5-flash' | 'gemini-1.5-pro' | 'deepseek-coder' | 'qwen2.5-coder' | 'claude-3.5-sonnet';
  provider: 'gemini' | 'ollama' | 'openrouter';
  auto_verify: boolean;
}
```

**Выходные данные (TypeScript):**
```typescript
interface TranspileOutput {
  status: 'success' | 'error';
  rust_code?: string;
  safety_score: number; // 0.0 to 100.0%
  unsafe_blocks_count: number;
  raw_pointers_count: number;
  compilation_verified: boolean;
  iterations_count: number;
  error_message?: string;
}
```

---

## 3. Команда: `calculate_safety_score`
**Описание:** Анализирует итоговый Rust-код на наличие `unsafe`, сырых указателей `*mut` и вычисляет индекс надежности.

**Выходные данные (TypeScript):**
```typescript
interface SafetyScoreReport {
  c_lines: number;
  rust_lines: number;
  unsafe_blocks: number;
  raw_pointers_count: number;
  safety_score_percentage: number;
  memory_leak_risk: 'ZERO' | 'LOW' | 'HIGH';
  data_race_risk: 'ZERO' | 'LOW' | 'HIGH';
}
```
