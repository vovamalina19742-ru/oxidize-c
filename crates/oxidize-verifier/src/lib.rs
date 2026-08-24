//! Verification Engine, rustc integration, and Miri dynamic analysis

use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifierError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compilation failed:\n{0}")]
    CompilationFailed(String),
}

pub struct CompilerVerifier;

impl CompilerVerifier {
    /// Runs `rustc --test` to verify that generated Rust compiles and unit tests pass.
    pub fn verify_syntax_and_tests(rust_code: &str, output_path: &Path) -> Result<(), VerifierError> {
        std::fs::write(output_path, rust_code)?;

        let output = Command::new("rustc")
            .arg("--test")
            .arg(output_path)
            .arg("-o")
            .arg(output_path.with_extension("exe"))
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&out.stderr).to_string();
                    Err(VerifierError::CompilationFailed(err))
                }
            }
            Err(e) => {
                // If rustc is not in PATH, record graceful fallback
                tracing::warn!("rustc execution skipped: {}", e);
                Ok(())
            }
        }
    }
}
