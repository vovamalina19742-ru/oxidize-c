//! Multi-LLM Orchestrator & Safe Rust Prompt Synthesizer

use oxidize_core::AstReport;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("API Error: {0}")]
    Api(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    Gemini { api_key: String, model: String },
    OpenRouter { api_key: String, model: String },
    Ollama { host: String, model: String },
}

pub const SYSTEM_PROMPT: &str = r#"You are Oxidize-C, a world-class C-to-Safe-Rust transpiler and systems engineer.
Your mission: Translate the provided C code into idiomatic, 100% SAFE Rust.

CRITICAL RULES:
1. NO RAW POINTERS (*mut T, *const T) in business logic.
2. CYCLIC GRAPHS & LISTS: Use Generational Arena / SlotMap with NodeKey (index + generation).
3. TAGGED UNIONS: Convert C unions with discriminants into Rust algebraic `enum` with associated data.
4. OS DESCRIPTORS: Use I/O Safety RAII (`std::os::fd::OwnedFd` on unix, `std::os::windows::io::OwnedSocket` on windows).
5. AUTOMATIC RAII: Replace malloc/free with `Vec<T>`, `Box<T>`, `Option<T>`, or `Result<T, E>`.
6. SELF-CONTAINED & TESTABLE: Always include `#[cfg(test)] mod tests { ... }` verifying all functions.
7. OUTPUT FORMAT: Output ONLY the raw Rust source code inside a single ```rust ... ``` code block.
"#;

pub struct LlmOrchestrator {
    provider: LlmProvider,
}

impl LlmOrchestrator {
    pub fn new(provider: LlmProvider) -> Self {
        Self { provider }
    }

    pub fn format_prompt(&self, c_code: &str, report: &AstReport, compiler_error: Option<&str>) -> String {
        let mut prompt = String::new();
        prompt.push_str("=== OXIDIZE-C AST DIRECTIVES ===\n");
        for pattern in &report.suggested_patterns {
            prompt.push_str(&format!("• Apply Pattern: {}\n", pattern.description()));
        }
        prompt.push_str("================================\n\n");
        prompt.push_str("Translate this C code into 100% Safe Rust:\n```c\n");
        prompt.push_str(c_code);
        prompt.push_str("\n```\n");

        if let Some(err) = compiler_error {
            prompt.push_str("\n[COMPILER ERROR FEEDBACK]\nThe previous attempt produced the following rustc error. Fix it:\n```\n");
            prompt.push_str(err);
            prompt.push_str("\n```\n");
        }
        prompt
    }
}
