use serde_json::Value;

pub(crate) fn append_canonical_json(bytes: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Null => bytes.extend_from_slice(b"<NULL>"),
        Value::Bool(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        Value::Number(value) => bytes.extend_from_slice(value.to_string().as_bytes()),
        Value::String(value) => bytes.extend_from_slice(value.as_bytes()),
        Value::Array(values) => {
            bytes.push(b'[');
            for value in values {
                append_canonical_json(bytes, value);
                bytes.push(0);
            }
            bytes.push(b']');
        }
        Value::Object(values) => {
            bytes.push(b'{');
            let mut canonical = values.iter().collect::<Vec<_>>();
            canonical.sort_by_key(|(name, _)| *name);
            for (name, value) in canonical {
                bytes.extend_from_slice(name.as_bytes());
                bytes.push(b'=');
                append_canonical_json(bytes, value);
                bytes.push(0);
            }
            bytes.push(b'}');
        }
    }
}
