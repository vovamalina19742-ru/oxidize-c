//! Oxidize-C Transpiled Module
//! Source: mini_json.c
//! Pattern: Rust Algebraic Enum & RAII (Zero Raw Pointers, Zero Unsafe)

/// A 100% Safe algebraic JSON value representation replacing C tagged union.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

impl JsonValue {
    #[must_use]
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    #[must_use]
    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }

    #[must_use]
    pub fn boolean(b: bool) -> Self {
        Self::Bool(b)
    }

    #[must_use]
    pub fn null() -> Self {
        Self::Null
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_values() {
        let str_val = JsonValue::string("Oxidize-C");
        let num_val = JsonValue::number(42.5);
        let null_val = JsonValue::null();

        assert_eq!(str_val.as_str(), Some("Oxidize-C"));
        assert_eq!(num_val.as_number(), Some(42.5));
        assert_eq!(null_val, JsonValue::Null);
    }
}
