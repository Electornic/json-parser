use crate::value::JsonValue;

pub fn to_string(value: &JsonValue) -> String {
    let mut out = String::new();
    write_compact(&mut out, value);
    out
}

pub fn to_string_pretty(value: &JsonValue, indent: usize) -> String {
    let mut out = String::new();
    write_pretty(&mut out, value, indent, 0);
    out
}

fn write_compact(out: &mut String, value: &JsonValue) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Number(n) => out.push_str(n.as_str()),
        JsonValue::String(s) => write_escaped(out, s),
        JsonValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_compact(out, item);
            }
            out.push(']');
        }
        JsonValue::Object(entries) => {
            out.push('{');
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_escaped(out, k);
                out.push(':');
                write_compact(out, v);
            }
            out.push('}');
        }
    }
}

fn write_pretty(out: &mut String, value: &JsonValue, indent: usize, level: usize) {
    match value {
        JsonValue::Array(items) if !items.is_empty() => {
            out.push('[');
            out.push('\n');
            for (i, item) in items.iter().enumerate() {
                push_indent(out, indent, level + 1);
                write_pretty(out, item, indent, level + 1);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, indent, level);
            out.push(']');
        }
        JsonValue::Object(entries) if !entries.is_empty() => {
            out.push('{');
            out.push('\n');
            for (i, (k, v)) in entries.iter().enumerate() {
                push_indent(out, indent, level + 1);
                write_escaped(out, k);
                out.push_str(": ");
                write_pretty(out, v, indent, level + 1);
                if i + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(out, indent, level);
            out.push('}');
        }
        _ => write_compact(out, value),
    }
}

fn push_indent(out: &mut String, indent: usize, level: usize) {
    for _ in 0..(indent * level) {
        out.push(' ');
    }
}

fn write_escaped(out: &mut String, s: &str) {
    use std::fmt::Write;
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).expect("writing to String");
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::Number;
    use crate::parser::parse;

    fn n(s: &str) -> JsonValue {
        JsonValue::Number(Number::from_raw(s))
    }

    #[test]
    fn null_bool_number() {
        assert_eq!(to_string(&JsonValue::Null), "null");
        assert_eq!(to_string(&JsonValue::Bool(true)), "true");
        assert_eq!(to_string(&JsonValue::Bool(false)), "false");
        assert_eq!(to_string(&n("3.14")), "3.14");
    }

    #[test]
    fn simple_string() {
        assert_eq!(to_string(&JsonValue::String("hi".into())), r#""hi""#);
    }

    #[test]
    fn string_escapes() {
        let s = JsonValue::String("a\"b\\c\nd\te".into());
        assert_eq!(to_string(&s), r#""a\"b\\c\nd\te""#);
    }

    #[test]
    fn string_passes_through_non_ascii() {
        let s = JsonValue::String("안녕 🦀".into());
        assert_eq!(to_string(&s), "\"안녕 🦀\"");
    }

    #[test]
    fn string_escapes_other_control_chars() {
        let s = JsonValue::String("\u{0001}\u{001F}".into());
        assert_eq!(to_string(&s), "\"\\u0001\\u001f\"");
    }

    #[test]
    fn empty_array_and_object() {
        assert_eq!(to_string(&JsonValue::Array(vec![])), "[]");
        assert_eq!(to_string(&JsonValue::Object(vec![])), "{}");
    }

    #[test]
    fn array_compact() {
        let v = JsonValue::Array(vec![n("1"), n("2"), JsonValue::Null]);
        assert_eq!(to_string(&v), "[1,2,null]");
    }

    #[test]
    fn object_compact() {
        let v = JsonValue::Object(vec![
            ("a".into(), n("1")),
            ("b".into(), JsonValue::Bool(true)),
        ]);
        assert_eq!(to_string(&v), r#"{"a":1,"b":true}"#);
    }

    #[test]
    fn nested_compact_preserves_key_order() {
        let v = JsonValue::Object(vec![
            ("z".into(), JsonValue::Array(vec![n("1")])),
            ("a".into(), JsonValue::Object(vec![("x".into(), JsonValue::Null)])),
        ]);
        assert_eq!(to_string(&v), r#"{"z":[1],"a":{"x":null}}"#);
    }

    #[test]
    fn pretty_indents_array() {
        let v = JsonValue::Array(vec![n("1"), n("2")]);
        let expected = "[\n  1,\n  2\n]";
        assert_eq!(to_string_pretty(&v, 2), expected);
    }

    #[test]
    fn pretty_indents_object() {
        let v = JsonValue::Object(vec![("a".into(), n("1")), ("b".into(), n("2"))]);
        let expected = "{\n  \"a\": 1,\n  \"b\": 2\n}";
        assert_eq!(to_string_pretty(&v, 2), expected);
    }

    #[test]
    fn pretty_empty_collections_stay_inline() {
        assert_eq!(to_string_pretty(&JsonValue::Array(vec![]), 2), "[]");
        assert_eq!(to_string_pretty(&JsonValue::Object(vec![]), 2), "{}");
    }

    #[test]
    fn pretty_nested() {
        let v = JsonValue::Object(vec![
            ("a".into(), JsonValue::Array(vec![n("1"), n("2")])),
        ]);
        let expected = "{\n  \"a\": [\n    1,\n    2\n  ]\n}";
        assert_eq!(to_string_pretty(&v, 2), expected);
    }

    #[test]
    fn roundtrip_complex() {
        let input = r#"{"name":"r","nums":[1,2.5,-3e2],"meta":{"ok":true,"empty":[]}}"#;
        let parsed = parse(input).unwrap();
        assert_eq!(to_string(&parsed), input);
    }

    #[test]
    fn roundtrip_pretty_then_parse_matches() {
        let input = r#"{"a":1,"b":[true,null,"x"]}"#;
        let parsed = parse(input).unwrap();
        let pretty = to_string_pretty(&parsed, 4);
        let reparsed = parse(&pretty).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn roundtrip_preserves_number_lexeme() {
        let input = r#"[1.000,1e100,99999999999999999999]"#;
        let parsed = parse(input).unwrap();
        assert_eq!(to_string(&parsed), input);
    }

    #[test]
    fn roundtrip_preserves_unicode_strings() {
        let input = r#"["안녕","🦀","a\nb"]"#;
        let parsed = parse(input).unwrap();
        let again = to_string(&parsed);
        assert_eq!(parse(&again).unwrap(), parsed);
    }
}
