use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub line: usize,
    pub col: usize,
    pub byte_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedChar(char),
    UnexpectedEof,
    InvalidEscape(char),
    InvalidUnicodeEscape,
    LoneSurrogate,
    InvalidNumber,
    ControlCharInString,
    DepthExceeded,
    TrailingData,
    ExpectedValue,
    ExpectedColon,
    ExpectedCommaOrEnd,
    ExpectedString,
}

impl ParseError {
    pub fn new(kind: ErrorKind, line: usize, col: usize, byte_offset: usize) -> Self {
        Self { kind, line, col, byte_offset }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}, column {}", self.kind, self.line, self.col)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnexpectedChar(c) => write!(f, "unexpected character {:?}", c),
            ErrorKind::UnexpectedEof => write!(f, "unexpected end of input"),
            ErrorKind::InvalidEscape(c) => write!(f, "invalid escape sequence \\{}", c),
            ErrorKind::InvalidUnicodeEscape => write!(f, "invalid unicode escape"),
            ErrorKind::LoneSurrogate => write!(f, "lone UTF-16 surrogate"),
            ErrorKind::InvalidNumber => write!(f, "invalid number"),
            ErrorKind::ControlCharInString => write!(f, "unescaped control character in string"),
            ErrorKind::DepthExceeded => write!(f, "maximum nesting depth exceeded"),
            ErrorKind::TrailingData => write!(f, "unexpected trailing data after value"),
            ErrorKind::ExpectedValue => write!(f, "expected a JSON value"),
            ErrorKind::ExpectedColon => write!(f, "expected ':' after object key"),
            ErrorKind::ExpectedCommaOrEnd => write!(f, "expected ',' or closing bracket"),
            ErrorKind::ExpectedString => write!(f, "expected a string"),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_position() {
        let err = ParseError::new(ErrorKind::UnexpectedEof, 3, 12, 42);
        assert_eq!(err.to_string(), "unexpected end of input at line 3, column 12");
    }

    #[test]
    fn display_unexpected_char() {
        let err = ParseError::new(ErrorKind::UnexpectedChar('}'), 1, 1, 0);
        assert_eq!(err.to_string(), "unexpected character '}' at line 1, column 1");
    }

    #[test]
    fn display_invalid_escape() {
        let err = ParseError::new(ErrorKind::InvalidEscape('q'), 2, 5, 10);
        assert_eq!(err.to_string(), "invalid escape sequence \\q at line 2, column 5");
    }

    #[test]
    fn is_std_error() {
        fn assert_error<E: std::error::Error>(_: &E) {}
        let err = ParseError::new(ErrorKind::UnexpectedEof, 1, 1, 0);
        assert_error(&err);
    }
}
