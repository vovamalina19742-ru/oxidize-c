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
    #[error("No LLM provider configured (set GEMINI_API_KEY, OPENROUTER_API_KEY, or OLLAMA_HOST)")]
    NoProviderConfigured,
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
    client: reqwest::Client,
}

impl LlmOrchestrator {
    pub fn new(provider: LlmProvider) -> Self {
        Self {
            provider,
            client: reqwest::Client::new(),
        }
    }

    /// Automatically constructs an orchestrator from environment variables.
    pub fn from_env() -> Result<Self, LlmError> {
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.trim().is_empty() {
                let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string());
                return Ok(Self::new(LlmProvider::Gemini {
                    api_key: key.trim().to_string(),
                    model,
                }));
            }
        }

        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if !key.trim().is_empty() {
                let model = std::env::var("OPENROUTER_MODEL").unwrap_or_else(|_| "qwen/qwen3-coder:free".to_string());
                return Ok(Self::new(LlmProvider::OpenRouter {
                    api_key: key.trim().to_string(),
                    model,
                }));
            }
        }

        if let Ok(host) = std::env::var("OLLAMA_HOST") {
            if !host.trim().is_empty() {
                let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string());
                return Ok(Self::new(LlmProvider::Ollama {
                    host: host.trim().to_string(),
                    model,
                }));
            }
        }

        Err(LlmError::NoProviderConfigured)
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

    /// Executes the LLM query and extracts Safe Rust code from response.
    pub async fn transpile(&self, c_code: &str, report: &AstReport, compiler_error: Option<&str>) -> Result<String, LlmError> {
        let user_prompt = self.format_prompt(c_code, report, compiler_error);

        let response_text = match &self.provider {
            LlmProvider::Gemini { api_key, model } => {
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                    model, api_key
                );
                let payload = serde_json::json!({
                    "contents": [{
                        "parts": [{"text": format!("{}\n\n{}", SYSTEM_PROMPT, user_prompt)}]
                    }],
                    "generationConfig": {"temperature": 0.1}
                });

                let resp = self.client.post(&url).json(&payload).send().await?;
                if !resp.status().is_success() {
                    return Err(LlmError::Api(format!("Gemini API returned status {}", resp.status())));
                }
                let data: serde_json::Value = resp.json().await?;
                data["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .ok_or_else(|| LlmError::Api("Invalid Gemini API JSON response format".to_string()))?
                    .to_string()
            }
            LlmProvider::OpenRouter { api_key, model } => {
                let url = "https://openrouter.ai/api/v1/chat/completions";
                let payload = serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": SYSTEM_PROMPT},
                        {"role": "user", "content": user_prompt}
                    ],
                    "temperature": 0.1
                });

                let resp = self
                    .client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&payload)
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    return Err(LlmError::Api(format!("OpenRouter API returned status {}", resp.status())));
                }
                let data: serde_json::Value = resp.json().await?;
                data["choices"][0]["message"]["content"]
                    .as_str()
                    .ok_or_else(|| LlmError::Api("Invalid OpenRouter response format".to_string()))?
                    .to_string()
            }
            LlmProvider::Ollama { host, model } => {
                let url = format!("{}/api/generate", host.trim_end_matches('/'));
                let payload = serde_json::json!({
                    "model": model,
                    "prompt": format!("{}\n\n{}", SYSTEM_PROMPT, user_prompt),
                    "stream": false,
                    "options": {"temperature": 0.1}
                });

                let resp = self.client.post(&url).json(&payload).send().await?;
                if !resp.status().is_success() {
                    return Err(LlmError::Api(format!("Ollama API returned status {}", resp.status())));
                }
                let data: serde_json::Value = resp.json().await?;
                data["response"]
                    .as_str()
                    .ok_or_else(|| LlmError::Api("Invalid Ollama response format".to_string()))?
                    .to_string()
            }
        };

        Ok(extract_rust_code(&response_text))
    }
}

/// Helper function to extract Rust code blocks from markdown output.
pub fn extract_rust_code(text: &str) -> String {
    if let Some(start) = text.find("```rust") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    } else if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    text.trim().to_string()
}
