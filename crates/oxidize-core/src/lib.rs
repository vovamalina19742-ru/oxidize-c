//! Oxidize-C Core Engine
//!
//! Provides static AST analysis, pointer escape analysis, memory pattern classification,
//! and safety score calculation.

pub mod analyzer;
pub mod metrics;
pub mod patterns;

pub use analyzer::{AstReport, CSourceAnalyzer, StructField, StructInfo};
pub use metrics::SafetyScoreReport;
pub use patterns::MemoryPattern;
