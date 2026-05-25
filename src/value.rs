use crate::number::Number;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let JsonValue::Bool(b) = self { Some(*b) } else { None }
    }

    pub fn as_number(&self) -> Option<&Number> {
        if let JsonValue::Number(n) = self { Some(n) } else { None }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self { Some(s) } else { None }
    }

    pub fn as_array(&self) -> Option<&[JsonValue]> {
        if let JsonValue::Array(a) = self { Some(a) } else { None }
    }

    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        if let JsonValue::Object(o) = self { Some(o) } else { None }
    }

    /// Look up the first value associated with `key` in an object.
    /// Returns `None` for non-objects or missing keys.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.as_object()?.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        JsonValue::Bool(b)
    }
}

impl From<String> for JsonValue {
    fn from(s: String) -> Self {
        JsonValue::String(s)
    }
}

impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        JsonValue::String(s.to_owned())
    }
}

impl From<Number> for JsonValue {
    fn from(n: Number) -> Self {
        JsonValue::Number(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_accessor() {
        assert!(JsonValue::Null.is_null());
        assert!(!JsonValue::Bool(true).is_null());
    }

    #[test]
    fn bool_accessor() {
        assert_eq!(JsonValue::Bool(true).as_bool(), Some(true));
        assert_eq!(JsonValue::Null.as_bool(), None);
    }

    #[test]
    fn string_accessor() {
        let v: JsonValue = "hello".into();
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn object_get_first_match() {
        let obj = JsonValue::Object(vec![
            ("a".into(), JsonValue::Bool(true)),
            ("b".into(), JsonValue::Bool(false)),
        ]);
        assert_eq!(obj.get("a"), Some(&JsonValue::Bool(true)));
        assert_eq!(obj.get("missing"), None);
    }

    #[test]
    fn object_preserves_insertion_order() {
        let obj = JsonValue::Object(vec![
            ("z".into(), JsonValue::Null),
            ("a".into(), JsonValue::Null),
        ]);
        let keys: Vec<&str> = obj.as_object().unwrap().iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["z", "a"]);
    }

    #[test]
    fn array_accessor() {
        let arr = JsonValue::Array(vec![JsonValue::Bool(true), JsonValue::Null]);
        let inner = arr.as_array().unwrap();
        assert_eq!(inner.len(), 2);
        assert_eq!(inner[0], JsonValue::Bool(true));
    }

    #[test]
    fn number_accessor() {
        let n = JsonValue::Number(Number::from_raw("123"));
        assert_eq!(n.as_number().unwrap().as_i64(), Some(123));
    }
}
