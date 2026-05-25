use std::fmt;

/// A JSON number, stored as its original lexeme to preserve full precision.
///
/// Conversions to `i64`/`u64`/`f64` are performed lazily on demand. The stored
/// string is always a syntactically valid JSON number — the lexer is the only
/// thing that constructs `Number` values in production code paths.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Number(String);

impl Number {
    /// Constructs a `Number` from a pre-validated JSON number lexeme.
    ///
    /// The lexer guarantees validity; this is exposed for tests and for
    /// callers building values programmatically.
    pub fn from_raw(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_i64(&self) -> Option<i64> {
        if self.is_integer_lexeme() {
            self.0.parse().ok()
        } else {
            None
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        if self.is_integer_lexeme() && !self.0.starts_with('-') {
            self.0.parse().ok()
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        self.0.parse().ok()
    }

    fn is_integer_lexeme(&self) -> bool {
        !self.0.contains(['.', 'e', 'E'])
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_lexeme_parses_as_i64_and_u64() {
        let n = Number::from_raw("42");
        assert_eq!(n.as_i64(), Some(42));
        assert_eq!(n.as_u64(), Some(42));
        assert_eq!(n.as_f64(), Some(42.0));
    }

    #[test]
    fn negative_integer_is_not_u64() {
        let n = Number::from_raw("-7");
        assert_eq!(n.as_i64(), Some(-7));
        assert_eq!(n.as_u64(), None);
        assert_eq!(n.as_f64(), Some(-7.0));
    }

    #[test]
    fn float_lexeme_is_not_integer() {
        let n = Number::from_raw("3.14");
        assert_eq!(n.as_i64(), None);
        assert_eq!(n.as_u64(), None);
        assert_eq!(n.as_f64(), Some(3.14));
    }

    #[test]
    fn exponent_is_not_integer() {
        let n = Number::from_raw("1e3");
        assert_eq!(n.as_i64(), None);
        assert_eq!(n.as_f64(), Some(1000.0));
    }

    #[test]
    fn i64_overflow_returns_none() {
        let n = Number::from_raw("99999999999999999999");
        assert_eq!(n.as_i64(), None);
        assert_eq!(n.as_u64(), None);
        assert!(n.as_f64().is_some());
    }

    #[test]
    fn as_str_returns_original_lexeme() {
        let n = Number::from_raw("1.0000");
        assert_eq!(n.as_str(), "1.0000");
    }

    #[test]
    fn display_uses_original_lexeme() {
        let n = Number::from_raw("0.1");
        assert_eq!(n.to_string(), "0.1");
    }
}
