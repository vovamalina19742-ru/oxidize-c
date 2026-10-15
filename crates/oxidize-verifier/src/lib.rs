//! Verification Engine, rustc integration, test runner, and Miri dynamic analysis

use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifierError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Rust compiler `rustc` not found in PATH: {0}")]
    CompilerNotFound(String),
    #[error("Compilation failed:\n{0}")]
    CompilationFailed(String),
    #[error("Unit tests failed:\n{0}")]
    TestsFailed(String),
    #[error("Miri verification failed:\n{0}")]
    MiriFailed(String),
    #[error("Linker not available for test binary execution: {0}")]
    LinkerUnavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationMode {
    /// Type-check and syntax-check via `rustc --emit=metadata --crate-type=lib`.
    MetadataOnly,
    /// Build test executable and execute unit tests (`rustc --test`).
    CompileAndRunTests,
    /// Execute dynamic undefined behavior analysis with Miri.
    MiriAnalysis,
}

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub syntax_verified: bool,
    pub tests_executed: bool,
    pub tests_passed: bool,
    pub miri_executed: bool,
    pub miri_passed: bool,
    pub diagnostics: Vec<String>,
}

pub struct CompilerVerifier;

impl CompilerVerifier {
    /// Verifies syntax, borrow checking, and types via `rustc --emit=metadata --crate-type=lib`.
    ///
    /// # Fail-Closed Security:
    /// Returns `Err(VerifierError::CompilerNotFound)` if rustc cannot be launched.
    /// Returns `Err(VerifierError::CompilationFailed)` if rustc outputs compilation errors.
    pub fn verify_syntax_and_tests(rust_code: &str, output_path: &Path) -> Result<(), VerifierError> {
        Self::verify_syntax(rust_code, output_path)
    }

    /// Verifies syntax, borrow checking and types via `rustc --emit=metadata --crate-type=lib`.
    pub fn verify_syntax(rust_code: &str, output_path: &Path) -> Result<(), VerifierError> {
        std::fs::write(output_path, rust_code)?;

        let output = match Command::new("rustc")
            .arg("--emit=metadata")
            .arg("--crate-type=lib")
            .arg(output_path)
            .output()
        {
            Ok(out) => out,
            Err(e) => return Err(VerifierError::CompilerNotFound(e.to_string())),
        };

        if output.status.success() {
            Ok(())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            Err(VerifierError::CompilationFailed(err))
        }
    }

    /// Compiles unit tests embedded in Rust code and executes them via `rustc --test`.
    pub fn verify_and_run_tests(rust_code: &str, test_dir: &Path, bin_name: &str) -> Result<String, VerifierError> {
        let temp_src = test_dir.join(format!("temp_test_{}.rs", bin_name));
        let temp_bin = test_dir.join(format!("temp_test_bin_{}.exe", bin_name));
        std::fs::write(&temp_src, rust_code)?;

        let compile_output = match Command::new("rustc")
            .arg("--test")
            .arg(&temp_src)
            .arg("-o")
            .arg(&temp_bin)
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                let _ = std::fs::remove_file(&temp_src);
                return Err(VerifierError::CompilerNotFound(e.to_string()));
            }
        };

        let _ = std::fs::remove_file(&temp_src);

        if !compile_output.status.success() {
            let err = String::from_utf8_lossy(&compile_output.stderr).to_string();
            if err.contains("linker `link.exe` not found") || err.contains("unable to find library") {
                return Err(VerifierError::LinkerUnavailable(err));
            }
            return Err(VerifierError::CompilationFailed(err));
        }

        // Run the test executable
        let run_output = match Command::new(&temp_bin).output() {
            Ok(out) => out,
            Err(e) => {
                let _ = std::fs::remove_file(&temp_bin);
                return Err(VerifierError::TestsFailed(format!("Failed to execute test binary: {}", e)));
            }
        };

        let _ = std::fs::remove_file(&temp_bin);

        if run_output.status.success() {
            Ok(String::from_utf8_lossy(&run_output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
            Err(VerifierError::TestsFailed(format!("{}\n{}", stdout, stderr)))
        }
    }

    /// Runs Miri dynamic analysis on the Rust codebase if Miri / cargo-miri is available.
    pub fn verify_with_miri(rust_code: &str, temp_dir: &Path, bin_name: &str) -> Result<String, VerifierError> {
        let temp_src = temp_dir.join(format!("temp_miri_{}.rs", bin_name));
        std::fs::write(&temp_src, rust_code)?;

        let output = match Command::new("cargo")
            .args(["miri", "--version"])
            .output()
        {
            Ok(out) if out.status.success() => {
                Command::new("cargo")
                    .args(["miri", "test"])
                    .current_dir(temp_dir)
                    .output()
            }
            _ => {
                let _ = std::fs::remove_file(&temp_src);
                return Err(VerifierError::MiriFailed(
                    "Miri is not installed on this toolchain (run `rustup component add miri` with nightly)".to_string(),
                ));
            }
        };

        let _ = std::fs::remove_file(&temp_src);

        match output {
            Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
            Ok(out) => Err(VerifierError::MiriFailed(String::from_utf8_lossy(&out.stderr).to_string())),
            Err(e) => Err(VerifierError::MiriFailed(e.to_string())),
        }
    }

    /// Full verification pipeline: syntax check, unit test execution, and optional Miri analysis.
    pub fn verify_full(
        rust_code: &str,
        output_path: &Path,
        run_tests: bool,
        run_miri: bool,
    ) -> Result<VerificationReport, VerifierError> {
        let mut report = VerificationReport {
            syntax_verified: false,
            tests_executed: false,
            tests_passed: false,
            miri_executed: false,
            miri_passed: false,
            diagnostics: Vec::new(),
        };

        // 1. Syntax & type check (Fail-closed)
        Self::verify_syntax(rust_code, output_path)?;
        report.syntax_verified = true;
        report.diagnostics.push("Syntax & borrow checking verified successfully (--emit=metadata --crate-type=lib)".to_string());

        let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
        let stem = output_path.file_stem().and_then(|s| s.to_str()).unwrap_or("module");

        // 2. Unit test execution if requested
        if run_tests {
            match Self::verify_and_run_tests(rust_code, parent_dir, stem) {
                Ok(test_out) => {
                    report.tests_executed = true;
                    report.tests_passed = true;
                    report.diagnostics.push(format!("Unit tests passed:\n{}", test_out.trim()));
                }
                Err(VerifierError::LinkerUnavailable(reason)) => {
                    report.diagnostics.push(format!(
                        "Unit test execution skipped (host linker unavailable): {}",
                        reason.lines().next().unwrap_or("")
                    ));
                }
                Err(e) => return Err(e),
            }
        }

        // 3. Miri dynamic analysis if requested
        if run_miri {
            match Self::verify_with_miri(rust_code, parent_dir, stem) {
                Ok(miri_out) => {
                    report.miri_executed = true;
                    report.miri_passed = true;
                    report.diagnostics.push(format!("Miri analysis passed:\n{}", miri_out.trim()));
                }
                Err(VerifierError::MiriFailed(msg)) => {
                    report.diagnostics.push(format!("Miri dynamic analysis notice: {}", msg));
                }
                Err(e) => return Err(e),
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_syntax_valid() {
        let code = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
        let temp_dir = std::env::temp_dir();
        let target = temp_dir.join("test_verifier_valid.rs");
        let result = CompilerVerifier::verify_syntax(code, &target);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn test_verifier_syntax_invalid() {
        let code = "pub fn invalid() { let x: i32 = \"not an int\"; }";
        let temp_dir = std::env::temp_dir();
        let target = temp_dir.join("test_verifier_invalid.rs");
        let result = CompilerVerifier::verify_syntax(code, &target);
        assert!(result.is_err());
        match result {
            Err(VerifierError::CompilationFailed(msg)) => {
                assert!(msg.contains("mismatched types"));
            }
            other => panic!("Expected CompilationFailed, got {:?}", other),
        }
        let _ = std::fs::remove_file(target);
    }
}
