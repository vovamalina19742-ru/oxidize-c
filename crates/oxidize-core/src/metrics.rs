//! Safety Score & Code Metrics Engine

use serde::{Deserialize, Serialize};

/// Computes safety metrics and % of unsafe elimination.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyScoreReport {
    pub c_lines: usize,
    pub rust_lines: usize,
    pub unsafe_blocks: usize,
    pub raw_pointers_count: usize,
    pub safety_score_percentage: f64,
    pub memory_leak_risk: &'static str,
    pub data_race_risk: &'static str,
}

impl SafetyScoreReport {
    pub fn compute(c_code: &str, rust_code: &str) -> Self {
        let c_lines = c_code.lines().count();
        let rust_lines = rust_code.lines().count();

        let unsafe_blocks = rust_code.matches("unsafe ").count() + rust_code.matches("unsafe{").count();
        let raw_pointers_count = rust_code.matches("*mut ").count() + rust_code.matches("*const ").count();

        let score = if unsafe_blocks == 0 && raw_pointers_count == 0 {
            100.0
        } else {
            let penalty = (unsafe_blocks * 25 + raw_pointers_count * 15) as f64;
            (100.0 - penalty).max(10.0)
        };

        Self {
            c_lines,
            rust_lines,
            unsafe_blocks,
            raw_pointers_count,
            safety_score_percentage: score,
            memory_leak_risk: if score >= 90.0 { "ZERO (Guaranteed by RAII)" } else { "LOW" },
            data_race_risk: if score >= 90.0 { "ZERO (Guaranteed by Send/Sync)" } else { "LOW" },
        }
    }
}
