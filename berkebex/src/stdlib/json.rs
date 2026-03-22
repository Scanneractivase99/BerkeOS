use serde_json::Value;

pub fn dumps(value: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(value).map_err(|e| format!("JSON parse error: {}", e))?;
    serde_json::to_string(&v).map_err(|e| format!("JSON serialize error: {}", e))
}

pub fn loads(s: &str) -> Result<Value, String> {
    serde_json::from_str(s).map_err(|e| format!("JSON parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dumps_string() {
        let result = dumps(r#""hello""#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dumps_number() {
        let result = dumps("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_dumps_object() {
        let result = dumps(r#"{"key": "value"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dumps_array() {
        let result = dumps("[1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_loads_valid_json() {
        let result = loads(r#"{"name": "test", "value": 42}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_loads_invalid_json() {
        let result = loads("not valid json");
        assert!(result.is_err());
    }
}
