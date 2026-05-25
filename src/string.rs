use crate::error::{ErrorKind, ParseError};
use crate::lexer::Position;

/// Decode the raw content of a JSON string literal (the bytes between the
/// opening and closing `"`, with escapes still encoded).
///
/// `start` is the position of the string's opening quote and is used for
/// error reporting; finer-grained positions inside the string are not
/// tracked.
pub fn unescape(raw: &str, start: Position) -> Result<String, ParseError> {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            i += 1;
            if i >= bytes.len() {
                return Err(err(ErrorKind::UnexpectedEof, start));
            }
            match bytes[i] {
                b'"' => { out.push('"'); i += 1; }
                b'\\' => { out.push('\\'); i += 1; }
                b'/' => { out.push('/'); i += 1; }
                b'b' => { out.push('\u{0008}'); i += 1; }
                b'f' => { out.push('\u{000C}'); i += 1; }
                b'n' => { out.push('\n'); i += 1; }
                b'r' => { out.push('\r'); i += 1; }
                b't' => { out.push('\t'); i += 1; }
                b'u' => {
                    i += 1;
                    let cu = parse_hex4(bytes, &mut i, start)?;
                    push_unicode(cu, bytes, &mut i, &mut out, start)?;
                }
                other => return Err(err(ErrorKind::InvalidEscape(other as char), start)),
            }
        } else if b < 0x20 {
            return Err(err(ErrorKind::ControlCharInString, start));
        } else {
            let c = raw[i..].chars().next().expect("non-empty UTF-8 slice");
            out.push(c);
            i += c.len_utf8();
        }
    }
    Ok(out)
}

fn push_unicode(
    cu: u32,
    bytes: &[u8],
    i: &mut usize,
    out: &mut String,
    start: Position,
) -> Result<(), ParseError> {
    if (0xD800..=0xDBFF).contains(&cu) {
        // High surrogate — must be followed by a low surrogate \uXXXX.
        if *i + 2 > bytes.len() || bytes[*i] != b'\\' || bytes[*i + 1] != b'u' {
            return Err(err(ErrorKind::LoneSurrogate, start));
        }
        *i += 2;
        let low = parse_hex4(bytes, i, start)?;
        if !(0xDC00..=0xDFFF).contains(&low) {
            return Err(err(ErrorKind::LoneSurrogate, start));
        }
        let code = 0x10000 + (cu - 0xD800) * 0x400 + (low - 0xDC00);
        let c = char::from_u32(code).ok_or_else(|| err(ErrorKind::InvalidUnicodeEscape, start))?;
        out.push(c);
    } else if (0xDC00..=0xDFFF).contains(&cu) {
        return Err(err(ErrorKind::LoneSurrogate, start));
    } else {
        let c = char::from_u32(cu).ok_or_else(|| err(ErrorKind::InvalidUnicodeEscape, start))?;
        out.push(c);
    }
    Ok(())
}

fn parse_hex4(bytes: &[u8], i: &mut usize, start: Position) -> Result<u32, ParseError> {
    if *i + 4 > bytes.len() {
        return Err(err(ErrorKind::InvalidUnicodeEscape, start));
    }
    let mut val: u32 = 0;
    for _ in 0..4 {
        let b = bytes[*i];
        let d = match b {
            b'0'..=b'9' => (b - b'0') as u32,
            b'a'..=b'f' => (b - b'a' + 10) as u32,
            b'A'..=b'F' => (b - b'A' + 10) as u32,
            _ => return Err(err(ErrorKind::InvalidUnicodeEscape, start)),
        };
        val = val * 16 + d;
        *i += 1;
    }
    Ok(val)
}

fn err(kind: ErrorKind, pos: Position) -> ParseError {
    ParseError::new(kind, pos.line, pos.col, pos.byte_offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos() -> Position {
        Position { line: 1, col: 1, byte_offset: 0 }
    }

    fn decode(s: &str) -> Result<String, ParseError> {
        unescape(s, pos())
    }

    #[test]
    fn empty() {
        assert_eq!(decode("").unwrap(), "");
    }

    #[test]
    fn plain_ascii() {
        assert_eq!(decode("hello world").unwrap(), "hello world");
    }

    #[test]
    fn passthrough_non_ascii() {
        assert_eq!(decode("안녕 é 🦀").unwrap(), "안녕 é 🦀");
    }

    #[test]
    fn simple_escapes() {
        assert_eq!(decode(r#"\"\\\/\b\f\n\r\t"#).unwrap(), "\"\\/\u{0008}\u{000C}\n\r\t");
    }

    #[test]
    fn unicode_escape_bmp() {
        assert_eq!(decode(r"é").unwrap(), "é");
        assert_eq!(decode(r"가").unwrap(), "가");
    }

    #[test]
    fn unicode_escape_surrogate_pair() {
        // 😀 = U+1F600 = 😀
        assert_eq!(decode(r"😀").unwrap(), "😀");
    }

    #[test]
    fn unicode_escape_mixed_with_text() {
        assert_eq!(decode(r"hi 😀 !").unwrap(), "hi 😀 !");
    }

    #[test]
    fn lone_high_surrogate_rejected() {
        let err = decode(r"\uD83D").unwrap_err();
        assert_eq!(err.kind, ErrorKind::LoneSurrogate);
    }

    #[test]
    fn lone_low_surrogate_rejected() {
        let err = decode(r"\uDE00").unwrap_err();
        assert_eq!(err.kind, ErrorKind::LoneSurrogate);
    }

    #[test]
    fn high_surrogate_followed_by_non_surrogate_rejected() {
        let err = decode(r"\uD83DA").unwrap_err();
        assert_eq!(err.kind, ErrorKind::LoneSurrogate);
    }

    #[test]
    fn high_surrogate_followed_by_text_rejected() {
        let err = decode(r"\uD83Dxx").unwrap_err();
        assert_eq!(err.kind, ErrorKind::LoneSurrogate);
    }

    #[test]
    fn invalid_escape_rejected() {
        let err = decode(r"\q").unwrap_err();
        assert!(matches!(err.kind, ErrorKind::InvalidEscape('q')));
    }

    #[test]
    fn truncated_unicode_escape() {
        let err = decode(r"\u00").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidUnicodeEscape);
    }

    #[test]
    fn non_hex_in_unicode_escape() {
        let err = decode(r"\uZZZZ").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidUnicodeEscape);
    }

    #[test]
    fn control_char_in_raw_rejected() {
        let err = decode("a\nb").unwrap_err();
        assert_eq!(err.kind, ErrorKind::ControlCharInString);
    }

    #[test]
    fn position_propagates_to_error() {
        let p = Position { line: 5, col: 12, byte_offset: 99 };
        let err = unescape(r"\q", p).unwrap_err();
        assert_eq!(err.line, 5);
        assert_eq!(err.col, 12);
        assert_eq!(err.byte_offset, 99);
    }
}
