use crate::error::{ErrorKind, ParseError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Null,
    True,
    False,
    /// Raw string content between the quotes — escape sequences are still
    /// encoded. `string::unescape` decodes them.
    String(String),
    /// Validated JSON number lexeme.
    Number(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
    pub byte_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned {
    pub token: Token,
    pub start: Position,
}

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { src: input.as_bytes(), pos: 0, line: 1, col: 1 }
    }

    pub fn position(&self) -> Position {
        Position { line: self.line, col: self.col, byte_offset: self.pos }
    }

    pub fn next_token(&mut self) -> Result<Option<Spanned>, ParseError> {
        self.skip_whitespace();
        let start = self.position();
        let b = match self.peek() {
            Some(b) => b,
            None => return Ok(None),
        };
        let token = match b {
            b'{' => { self.bump(); Token::LBrace }
            b'}' => { self.bump(); Token::RBrace }
            b'[' => { self.bump(); Token::LBracket }
            b']' => { self.bump(); Token::RBracket }
            b':' => { self.bump(); Token::Colon }
            b',' => { self.bump(); Token::Comma }
            b't' => self.read_keyword(b"true", Token::True)?,
            b'f' => self.read_keyword(b"false", Token::False)?,
            b'n' => self.read_keyword(b"null", Token::Null)?,
            b'"' => self.read_string(start)?,
            b'-' | b'0'..=b'9' => self.read_number(start)?,
            _ => {
                let c = self.peek_char().unwrap_or(b as char);
                return Err(self.err(ErrorKind::UnexpectedChar(c)));
            }
        };
        Ok(Some(Spanned { token, start }))
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else if (b & 0xC0) != 0x80 {
            self.col += 1;
        }
        Some(b)
    }

    fn peek_char(&self) -> Option<char> {
        std::str::from_utf8(&self.src[self.pos..]).ok()?.chars().next()
    }

    fn err(&self, kind: ErrorKind) -> ParseError {
        ParseError::new(kind, self.line, self.col, self.pos)
    }

    fn err_at(&self, kind: ErrorKind, pos: Position) -> ParseError {
        ParseError::new(kind, pos.line, pos.col, pos.byte_offset)
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => { self.bump(); }
                _ => return,
            }
        }
    }

    fn read_keyword(&mut self, expected: &[u8], token: Token) -> Result<Token, ParseError> {
        for &exp in expected {
            match self.peek() {
                Some(b) if b == exp => { self.bump(); }
                Some(b) => return Err(self.err(ErrorKind::UnexpectedChar(b as char))),
                None => return Err(self.err(ErrorKind::UnexpectedEof)),
            }
        }
        Ok(token)
    }

    fn read_string(&mut self, start: Position) -> Result<Token, ParseError> {
        self.bump(); // opening "
        let content_start = self.pos;
        loop {
            match self.peek() {
                None => return Err(self.err_at(ErrorKind::UnexpectedEof, start)),
                Some(b'"') => {
                    let raw = std::str::from_utf8(&self.src[content_start..self.pos])
                        .expect("input is &str so byte slice is valid UTF-8")
                        .to_owned();
                    self.bump();
                    return Ok(Token::String(raw));
                }
                Some(b'\\') => {
                    self.bump();
                    match self.peek() {
                        None => return Err(self.err(ErrorKind::UnexpectedEof)),
                        Some(b'u') => {
                            self.bump();
                            for _ in 0..4 {
                                match self.peek() {
                                    Some(b) if b.is_ascii_hexdigit() => { self.bump(); }
                                    Some(_) => return Err(self.err(ErrorKind::InvalidUnicodeEscape)),
                                    None => return Err(self.err(ErrorKind::UnexpectedEof)),
                                }
                            }
                        }
                        Some(_) => { self.bump(); }
                    }
                }
                Some(b) if b < 0x20 => return Err(self.err(ErrorKind::ControlCharInString)),
                Some(_) => { self.bump(); }
            }
        }
    }

    fn read_number(&mut self, start: Position) -> Result<Token, ParseError> {
        let start_byte = self.pos;

        if self.peek() == Some(b'-') {
            self.bump();
        }

        match self.peek() {
            Some(b'0') => { self.bump(); }
            Some(b'1'..=b'9') => {
                self.bump();
                while let Some(b'0'..=b'9') = self.peek() {
                    self.bump();
                }
            }
            _ => return Err(self.err_at(ErrorKind::InvalidNumber, start)),
        }

        if self.peek() == Some(b'.') {
            self.bump();
            let digits_start = self.pos;
            while let Some(b'0'..=b'9') = self.peek() {
                self.bump();
            }
            if self.pos == digits_start {
                return Err(self.err_at(ErrorKind::InvalidNumber, start));
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.bump();
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.bump();
            }
            let digits_start = self.pos;
            while let Some(b'0'..=b'9') = self.peek() {
                self.bump();
            }
            if self.pos == digits_start {
                return Err(self.err_at(ErrorKind::InvalidNumber, start));
            }
        }

        let raw = std::str::from_utf8(&self.src[start_byte..self.pos])
            .expect("number lexeme is ASCII")
            .to_owned();
        Ok(Token::Number(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Result<Vec<Token>, ParseError> {
        let mut lex = Lexer::new(input);
        let mut out = Vec::new();
        while let Some(s) = lex.next_token()? {
            out.push(s.token);
        }
        Ok(out)
    }

    #[test]
    fn structural_tokens() {
        let t = tokens("{}[]:,").unwrap();
        assert_eq!(t, vec![
            Token::LBrace, Token::RBrace,
            Token::LBracket, Token::RBracket,
            Token::Colon, Token::Comma,
        ]);
    }

    #[test]
    fn keywords() {
        let t = tokens("null true false").unwrap();
        assert_eq!(t, vec![Token::Null, Token::True, Token::False]);
    }

    #[test]
    fn partial_keyword_errors() {
        let err = tokens("nul").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedEof);
    }

    #[test]
    fn keyword_typo_errors() {
        let err = tokens("tru1").unwrap_err();
        assert!(matches!(err.kind, ErrorKind::UnexpectedChar('1')));
    }

    #[test]
    fn simple_string() {
        let t = tokens(r#""hello""#).unwrap();
        assert_eq!(t, vec![Token::String("hello".into())]);
    }

    #[test]
    fn empty_string() {
        let t = tokens(r#""""#).unwrap();
        assert_eq!(t, vec![Token::String("".into())]);
    }

    #[test]
    fn string_keeps_raw_escapes() {
        // Lexer doesn't decode; it just scans past escapes.
        let t = tokens(r#""a\"b\\c""#).unwrap();
        assert_eq!(t, vec![Token::String(r#"a\"b\\c"#.into())]);
    }

    #[test]
    fn string_unterminated() {
        let err = tokens(r#""abc"#).unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedEof);
    }

    #[test]
    fn string_unescaped_control_rejected() {
        let err = tokens("\"a\tb\"").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ControlCharInString);
    }

    #[test]
    fn string_invalid_unicode_escape() {
        let err = tokens(r#""\uZZZZ""#).unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidUnicodeEscape);
    }

    #[test]
    fn string_unicode_escape_ok() {
        let t = tokens(r#""é""#).unwrap();
        assert_eq!(t, vec![Token::String(r#"é"#.into())]);
    }

    #[test]
    fn string_with_non_ascii_passthrough() {
        let t = tokens("\"안녕\"").unwrap();
        assert_eq!(t, vec![Token::String("안녕".into())]);
    }

    #[test]
    fn numbers() {
        let t = tokens("0 -1 3.14 1e10 1.5e-3 -2.5E+2").unwrap();
        assert_eq!(t, vec![
            Token::Number("0".into()),
            Token::Number("-1".into()),
            Token::Number("3.14".into()),
            Token::Number("1e10".into()),
            Token::Number("1.5e-3".into()),
            Token::Number("-2.5E+2".into()),
        ]);
    }

    #[test]
    fn leading_zero_rejected() {
        // "01" lexes "0" as Number, then "1" as another Number — both valid
        // individually. Parser will reject the trailing data at the document
        // level. But "00" likewise. The interesting RFC violation "012" gets
        // tokenized as Number("0"), Number("12"); the parser will fail with
        // trailing data when used as a top-level value.
        let t = tokens("01").unwrap();
        assert_eq!(t, vec![Token::Number("0".into()), Token::Number("1".into())]);
    }

    #[test]
    fn bare_minus_rejected() {
        let err = tokens("-").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidNumber);
    }

    #[test]
    fn trailing_dot_rejected() {
        let err = tokens("1.").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidNumber);
    }

    #[test]
    fn bare_exponent_rejected() {
        let err = tokens("1e").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidNumber);
    }

    #[test]
    fn exponent_sign_without_digits_rejected() {
        let err = tokens("1e+").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidNumber);
    }

    #[test]
    fn dot_first_rejected_as_unexpected_char() {
        let err = tokens(".5").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedChar('.'));
    }

    #[test]
    fn whitespace_skipped() {
        let t = tokens(" \t\n\r 1 \n 2 ").unwrap();
        assert_eq!(t, vec![Token::Number("1".into()), Token::Number("2".into())]);
    }

    #[test]
    fn position_tracks_newlines() {
        let mut lex = Lexer::new("1\n  2");
        let first = lex.next_token().unwrap().unwrap();
        assert_eq!(first.start.line, 1);
        assert_eq!(first.start.col, 1);
        let second = lex.next_token().unwrap().unwrap();
        assert_eq!(second.start.line, 2);
        assert_eq!(second.start.col, 3);
    }

    #[test]
    fn position_counts_codepoints_not_bytes() {
        // "안" is 3 bytes in UTF-8; col should advance by 1 char.
        let mut lex = Lexer::new("\"안\" 1");
        let _ = lex.next_token().unwrap().unwrap();
        let num = lex.next_token().unwrap().unwrap();
        assert_eq!(num.start.col, 5); // " 안 " 1 -> col 5 (after 3-char string + space)
    }

    #[test]
    fn empty_input_returns_none() {
        let mut lex = Lexer::new("");
        assert!(lex.next_token().unwrap().is_none());
    }

    #[test]
    fn whitespace_only_returns_none() {
        let mut lex = Lexer::new("   \n\t  ");
        assert!(lex.next_token().unwrap().is_none());
    }

    #[test]
    fn unexpected_char_reports_unicode() {
        // The first non-ASCII character should be reported as a char, not garbage.
        let err = tokens("é").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedChar('é'));
    }
}
