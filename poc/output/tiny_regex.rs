//! Oxidize-C Transpiled Module
//! Source: tiny_regex.c
//! Pattern: 100% Safe Rust String Slicing & Recursive Matcher

/// A lightweight regular expression matcher implementing ^, $, ., * operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegexEngine;

impl RegexEngine {
    /// Matches a regular expression pattern against input text.
    #[must_use]
    pub fn is_match(pattern: &str, text: &str) -> bool {
        let pattern_bytes = pattern.as_bytes();
        let text_bytes = text.as_bytes();

        if let Some(&b'^') = pattern_bytes.first() {
            return Self::match_here(&pattern_bytes[1..], text_bytes);
        }

        let mut curr_text = text_bytes;
        loop {
            if Self::match_here(pattern_bytes, curr_text) {
                return true;
            }
            if curr_text.is_empty() {
                break;
            }
            curr_text = &curr_text[1..];
        }
        false
    }

    fn match_here(pattern: &[u8], text: &[u8]) -> bool {
        if pattern.is_empty() {
            return true;
        }
        if pattern.len() >= 2 && pattern[1] == b'*' {
            return Self::match_star(pattern[0], &pattern[2..], text);
        }
        if pattern.len() == 1 && pattern[0] == b'$' {
            return text.is_empty();
        }
        if !text.is_empty() && (pattern[0] == b'.' || pattern[0] == text[0]) {
            return Self::match_here(&pattern[1..], &text[1..]);
        }
        false
    }

    fn match_star(c: u8, pattern: &[u8], mut text: &[u8]) -> bool {
        loop {
            if Self::match_here(pattern, text) {
                return true;
            }
            if !text.is_empty() && (text[0] == c || c == b'.') {
                text = &text[1..];
            } else {
                break;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_exact_match() {
        assert!(RegexEngine::is_match("hello", "hello"));
        assert!(RegexEngine::is_match("^hello$", "hello"));
        assert!(!RegexEngine::is_match("^hello$", "hello world"));
    }

    #[test]
    fn test_regex_wildcard_and_star() {
        assert!(RegexEngine::is_match("a.c", "abc"));
        assert!(RegexEngine::is_match("a*b", "aaaaab"));
        assert!(RegexEngine::is_match("a*b", "b"));
        assert!(RegexEngine::is_match(".*", "anything"));
    }
}
