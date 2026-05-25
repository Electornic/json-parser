use crate::error::{ErrorKind, ParseError};
use crate::lexer::{Lexer, Position, Spanned, Token};
use crate::number::Number;
use crate::string;
use crate::value::JsonValue;

const MAX_DEPTH: usize = 128;

pub fn parse(input: &str) -> Result<JsonValue, ParseError> {
    let mut p = Parser::new(input);
    let value = p.parse_value()?;
    if let Some(spanned) = p.next_token()? {
        return Err(err_at(ErrorKind::TrailingData, spanned.start));
    }
    Ok(value)
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Spanned>,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { lexer: Lexer::new(input), peeked: None, depth: 0 }
    }

    fn next_token(&mut self) -> Result<Option<Spanned>, ParseError> {
        if let Some(t) = self.peeked.take() {
            Ok(Some(t))
        } else {
            self.lexer.next_token()
        }
    }

    fn peek_token(&mut self) -> Result<Option<&Spanned>, ParseError> {
        if self.peeked.is_none() {
            self.peeked = self.lexer.next_token()?;
        }
        Ok(self.peeked.as_ref())
    }

    fn current_pos(&self) -> Position {
        self.lexer.position()
    }

    fn parse_value(&mut self) -> Result<JsonValue, ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(err_at(ErrorKind::DepthExceeded, self.current_pos()));
        }
        let spanned = self.next_token()?
            .ok_or_else(|| err_at(ErrorKind::ExpectedValue, self.current_pos()))?;
        let value = match spanned.token {
            Token::Null => JsonValue::Null,
            Token::True => JsonValue::Bool(true),
            Token::False => JsonValue::Bool(false),
            Token::Number(s) => JsonValue::Number(Number::from_raw(s)),
            Token::String(raw) => JsonValue::String(string::unescape(&raw, spanned.start)?),
            Token::LBracket => self.parse_array_body()?,
            Token::LBrace => self.parse_object_body()?,
            Token::RBrace | Token::RBracket | Token::Comma | Token::Colon => {
                return Err(err_at(ErrorKind::ExpectedValue, spanned.start));
            }
        };
        self.depth -= 1;
        Ok(value)
    }

    fn parse_array_body(&mut self) -> Result<JsonValue, ParseError> {
        let mut items = Vec::new();
        match self.peek_token()? {
            Some(s) if s.token == Token::RBracket => {
                self.next_token()?;
                return Ok(JsonValue::Array(items));
            }
            Some(_) => {}
            None => return Err(err_at(ErrorKind::UnexpectedEof, self.current_pos())),
        }
        loop {
            items.push(self.parse_value()?);
            let sep = self.next_token()?
                .ok_or_else(|| err_at(ErrorKind::UnexpectedEof, self.current_pos()))?;
            match sep.token {
                Token::Comma => continue,
                Token::RBracket => return Ok(JsonValue::Array(items)),
                _ => return Err(err_at(ErrorKind::ExpectedCommaOrEnd, sep.start)),
            }
        }
    }

    fn parse_object_body(&mut self) -> Result<JsonValue, ParseError> {
        let mut items: Vec<(String, JsonValue)> = Vec::new();
        match self.peek_token()? {
            Some(s) if s.token == Token::RBrace => {
                self.next_token()?;
                return Ok(JsonValue::Object(items));
            }
            Some(_) => {}
            None => return Err(err_at(ErrorKind::UnexpectedEof, self.current_pos())),
        }
        loop {
            let key_tok = self.next_token()?
                .ok_or_else(|| err_at(ErrorKind::UnexpectedEof, self.current_pos()))?;
            let key = match key_tok.token {
                Token::String(raw) => string::unescape(&raw, key_tok.start)?,
                _ => return Err(err_at(ErrorKind::ExpectedString, key_tok.start)),
            };
            let colon = self.next_token()?
                .ok_or_else(|| err_at(ErrorKind::UnexpectedEof, self.current_pos()))?;
            if colon.token != Token::Colon {
                return Err(err_at(ErrorKind::ExpectedColon, colon.start));
            }
            let value = self.parse_value()?;
            items.push((key, value));
            let sep = self.next_token()?
                .ok_or_else(|| err_at(ErrorKind::UnexpectedEof, self.current_pos()))?;
            match sep.token {
                Token::Comma => continue,
                Token::RBrace => return Ok(JsonValue::Object(items)),
                _ => return Err(err_at(ErrorKind::ExpectedCommaOrEnd, sep.start)),
            }
        }
    }
}

fn err_at(kind: ErrorKind, pos: Position) -> ParseError {
    ParseError::new(kind, pos.line, pos.col, pos.byte_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> JsonValue {
        JsonValue::Number(Number::from_raw(s))
    }

    #[test]
    fn null_value() {
        assert_eq!(parse("null").unwrap(), JsonValue::Null);
    }

    #[test]
    fn bool_values() {
        assert_eq!(parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(parse("false").unwrap(), JsonValue::Bool(false));
    }

    #[test]
    fn number_value() {
        assert_eq!(parse("42").unwrap(), n("42"));
        assert_eq!(parse("-3.14e2").unwrap(), n("-3.14e2"));
    }

    #[test]
    fn string_value_with_escapes() {
        assert_eq!(
            parse(r#""hello\n\"world\"""#).unwrap(),
            JsonValue::String("hello\n\"world\"".to_owned())
        );
    }

    #[test]
    fn empty_array() {
        assert_eq!(parse("[]").unwrap(), JsonValue::Array(vec![]));
    }

    #[test]
    fn array_with_values() {
        assert_eq!(
            parse("[1, 2, true, null]").unwrap(),
            JsonValue::Array(vec![n("1"), n("2"), JsonValue::Bool(true), JsonValue::Null])
        );
    }

    #[test]
    fn nested_array() {
        assert_eq!(
            parse("[[1], [2, [3]]]").unwrap(),
            JsonValue::Array(vec![
                JsonValue::Array(vec![n("1")]),
                JsonValue::Array(vec![n("2"), JsonValue::Array(vec![n("3")])]),
            ])
        );
    }

    #[test]
    fn empty_object() {
        assert_eq!(parse("{}").unwrap(), JsonValue::Object(vec![]));
    }

    #[test]
    fn object_preserves_key_order() {
        let v = parse(r#"{"b": 1, "a": 2, "c": 3}"#).unwrap();
        let keys: Vec<&str> = v.as_object().unwrap().iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["b", "a", "c"]);
    }

    #[test]
    fn nested_object() {
        let v = parse(r#"{"outer": {"inner": [1, 2]}}"#).unwrap();
        let inner = v.get("outer").unwrap().get("inner").unwrap();
        assert_eq!(*inner, JsonValue::Array(vec![n("1"), n("2")]));
    }

    #[test]
    fn whitespace_tolerated_everywhere() {
        let v = parse("  {  \"a\"  :  [  1  ,  2  ]  }  ").unwrap();
        assert_eq!(
            v,
            JsonValue::Object(vec![("a".into(), JsonValue::Array(vec![n("1"), n("2")]))])
        );
    }

    #[test]
    fn trailing_comma_in_array_rejected() {
        let err = parse("[1,]").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedValue);
    }

    #[test]
    fn trailing_comma_in_object_rejected() {
        let err = parse(r#"{"a":1,}"#).unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedString);
    }

    #[test]
    fn missing_colon_rejected() {
        let err = parse(r#"{"a" 1}"#).unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedColon);
    }

    #[test]
    fn missing_comma_in_array_rejected() {
        let err = parse("[1 2]").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedCommaOrEnd);
    }

    #[test]
    fn non_string_key_rejected() {
        let err = parse("{1: 2}").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedString);
    }

    #[test]
    fn empty_input_rejected() {
        let err = parse("").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedValue);
    }

    #[test]
    fn whitespace_only_rejected() {
        let err = parse("   \n  ").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedValue);
    }

    #[test]
    fn trailing_data_rejected() {
        let err = parse("1 2").unwrap_err();
        assert_eq!(err.kind, ErrorKind::TrailingData);
    }

    #[test]
    fn trailing_data_after_array() {
        let err = parse("[1] junk").unwrap_err();
        // "junk" starts as 'j' which isn't a valid token start
        // so the lexer error fires before TrailingData
        assert!(matches!(err.kind, ErrorKind::UnexpectedChar(_)));
    }

    #[test]
    fn trailing_value_token_is_trailing_data() {
        let err = parse("1 true").unwrap_err();
        assert_eq!(err.kind, ErrorKind::TrailingData);
    }

    #[test]
    fn depth_limit_enforced() {
        // 200 levels of nesting; MAX_DEPTH = 128
        let deep = "[".repeat(200) + &"]".repeat(200);
        let err = parse(&deep).unwrap_err();
        assert_eq!(err.kind, ErrorKind::DepthExceeded);
    }

    #[test]
    fn depth_at_limit_succeeds() {
        let deep = "[".repeat(MAX_DEPTH) + &"]".repeat(MAX_DEPTH);
        assert!(parse(&deep).is_ok());
    }

    #[test]
    fn unclosed_array() {
        let err = parse("[1, 2").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedEof);
    }

    #[test]
    fn unclosed_object() {
        let err = parse(r#"{"a": 1"#).unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnexpectedEof);
    }

    #[test]
    fn empty_value_after_colon() {
        let err = parse(r#"{"a":}"#).unwrap_err();
        assert_eq!(err.kind, ErrorKind::ExpectedValue);
    }

    #[test]
    fn duplicate_keys_last_wins_via_get() {
        // Both kept, but get() returns the first per current contract.
        // This test documents that behavior.
        let v = parse(r#"{"a": 1, "a": 2}"#).unwrap();
        let obj = v.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(v.get("a"), Some(&n("1")));
    }
}
