use json_parser::{parse, to_string, to_string_pretty, JsonValue, Number};

#[test]
fn parses_and_serializes_full_document() {
    let input = r#"{
        "name": "json-parser",
        "version": 1,
        "tags": ["rust", "rfc8259"],
        "meta": {
            "stable": true,
            "owner": null
        }
    }"#;

    let value = parse(input).expect("valid JSON");
    let compact = to_string(&value);
    let reparsed = parse(&compact).expect("compact JSON is valid");
    assert_eq!(value, reparsed);

    let pretty = to_string_pretty(&value, 4);
    let reparsed_pretty = parse(&pretty).expect("pretty JSON is valid");
    assert_eq!(value, reparsed_pretty);
}

#[test]
fn typed_accessors_work_through_public_api() {
    let v = parse(r#"{"a": [1, 2, "x"], "b": true}"#).unwrap();
    let a = v.get("a").unwrap().as_array().unwrap();
    assert_eq!(a.len(), 3);
    assert_eq!(a[0].as_number().unwrap().as_i64(), Some(1));
    assert_eq!(a[2].as_str(), Some("x"));
    assert_eq!(v.get("b").unwrap().as_bool(), Some(true));
}

#[test]
fn errors_carry_position() {
    let err = parse("{\n  \"a\": ,\n}").unwrap_err();
    assert_eq!(err.line, 2);
    assert!(err.col >= 1);
    let msg = err.to_string();
    assert!(msg.contains("line"), "error message should include position: {msg}");
}

#[test]
fn number_lexeme_preserved_through_api() {
    let v = parse("99999999999999999999").unwrap();
    let n = v.as_number().unwrap();
    assert_eq!(n.as_str(), "99999999999999999999");
    assert_eq!(n.as_i64(), None);
    assert!(n.as_f64().is_some());
}

#[test]
fn build_value_programmatically_and_serialize() {
    let v = JsonValue::Object(vec![
        ("ok".into(), JsonValue::Bool(true)),
        ("count".into(), JsonValue::Number(Number::from_raw("3"))),
    ]);
    assert_eq!(to_string(&v), r#"{"ok":true,"count":3}"#);
}
