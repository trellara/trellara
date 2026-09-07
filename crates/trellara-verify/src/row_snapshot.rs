use serde::{Deserialize, Serialize};
use serde_json::Value;
use xxhash_rust::xxh3::xxh3_64;

use crate::canonical_json::append_canonical_json;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RowSnapshot {
    pub primary_key: String,
    pub row_hash: u64,
}

impl RowSnapshot {
    pub fn from_columns(primary_key: impl Into<String>, columns: &[(&str, Option<&str>)]) -> Self {
        let mut canonical = columns
            .iter()
            .map(|(name, value)| (*name, *value))
            .collect::<Vec<_>>();
        canonical.sort_by_key(|(name, _)| *name);

        let mut bytes = Vec::new();
        for (name, value) in canonical {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(b'=');
            if let Some(value) = value {
                bytes.extend_from_slice(value.as_bytes());
            } else {
                bytes.extend_from_slice(b"<NULL>");
            }
            bytes.push(0);
        }

        Self {
            primary_key: primary_key.into(),
            row_hash: xxh3_64(&bytes),
        }
    }

    pub fn from_json_value(primary_key: impl Into<String>, value: &Value) -> Option<Self> {
        let Value::Object(columns) = value else {
            return None;
        };
        let mut canonical = columns.iter().collect::<Vec<_>>();
        canonical.sort_by_key(|(name, _)| *name);

        let mut bytes = Vec::new();
        for (name, value) in canonical {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(b'=');
            append_canonical_json(&mut bytes, value);
            bytes.push(0);
        }

        Some(Self {
            primary_key: primary_key.into(),
            row_hash: xxh3_64(&bytes),
        })
    }
}
