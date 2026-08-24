//! Static AST Analyzer & Pattern Classifier for C Sources

use crate::patterns::MemoryPattern;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub type_str: String,
    pub is_pointer: bool,
    pub is_self_ref: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<StructField>,
    pub is_cyclic: bool,
    pub has_union: bool,
    pub has_os_descriptor: bool,
    pub pattern: MemoryPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstReport {
    pub filename: String,
    pub total_lines: usize,
    pub has_malloc: bool,
    pub has_free: bool,
    pub has_realloc: bool,
    pub has_os_socket: bool,
    pub structs: Vec<StructInfo>,
    pub suggested_patterns: Vec<MemoryPattern>,
}

pub struct CSourceAnalyzer<'a> {
    source: &'a str,
    filename: &'a str,
}

impl<'a> CSourceAnalyzer<'a> {
    pub fn new(source: &'a str, filename: &'a str) -> Self {
        Self { source, filename }
    }

    pub fn analyze(&self) -> AstReport {
        let total_lines = self.source.lines().count();

        let has_malloc = self.source.contains("malloc(");
        let has_free = self.source.contains("free(");
        let has_realloc = self.source.contains("realloc(");

        let os_regex = Regex::new(r"\b(socket|closesocket|close|SOCKET|socket_t|RawFd|OwnedFd|open|read|write)\b").unwrap();
        let has_os_socket = os_regex.is_match(self.source);

        let mut structs = Vec::new();
        let struct_regex = Regex::new(r"(?s)(?:typedef\s+)?struct\s*([A-Za-z0-9_]*)\s*\{([^}]+)\}\s*([A-Za-z0-9_]*)\s*;").unwrap();

        for cap in struct_regex.captures_iter(self.source) {
            let tag_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let alias_name = cap.get(3).map(|m| m.as_str()).unwrap_or("");

            let struct_name = if !alias_name.is_empty() {
                alias_name.to_string()
            } else if !tag_name.is_empty() {
                tag_name.to_string()
            } else {
                "AnonymousStruct".to_string()
            };

            let has_union = body.contains("union");
            let mut is_cyclic = false;
            let mut has_os_descriptor = false;
            let mut fields = Vec::new();

            for field_line in body.split(';') {
                let clean = field_line.trim();
                if clean.is_empty() || clean.contains("union") {
                    continue;
                }

                let is_pointer = clean.contains('*');
                let is_self_ref = clean.contains(&struct_name) || (!tag_name.is_empty() && clean.contains(tag_name));

                if is_self_ref && is_pointer {
                    is_cyclic = true;
                }

                if clean.contains("socket") || clean.contains("sock_fd") || clean.contains("fd") || clean.contains("SOCKET") {
                    has_os_descriptor = true;
                }

                fields.push(StructField {
                    name: clean.split_whitespace().last().unwrap_or("").to_string(),
                    type_str: clean.to_string(),
                    is_pointer,
                    is_self_ref,
                });
            }

            let pattern = if is_cyclic {
                MemoryPattern::GenerationalSlotMap
            } else if has_os_descriptor {
                MemoryPattern::IoSafetyRaii
            } else if has_union {
                MemoryPattern::AlgebraicEnum
            } else if has_malloc && has_free {
                MemoryPattern::RaiiOwnership
            } else {
                MemoryPattern::StandardStruct
            };

            structs.push(StructInfo {
                name: struct_name,
                fields,
                is_cyclic,
                has_union,
                has_os_descriptor,
                pattern,
            });
        }

        let mut patterns = std::collections::HashSet::new();
        if has_os_socket {
            patterns.insert(MemoryPattern::IoSafetyRaii);
        }
        for s in &structs {
            patterns.insert(s.pattern.clone());
        }

        AstReport {
            filename: self.filename.to_string(),
            total_lines,
            has_malloc,
            has_free,
            has_realloc,
            has_os_socket,
            structs,
            suggested_patterns: patterns.into_iter().collect(),
        }
    }
}
