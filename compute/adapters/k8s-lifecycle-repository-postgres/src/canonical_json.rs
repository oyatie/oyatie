use compute_k8s_api::CloudComputeK8sLifecycleRepositoryError;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::error::integrity;

pub(crate) fn json_digest(
    value: &Value,
) -> Result<String, CloudComputeK8sLifecycleRepositoryError> {
    let canonical = canonicalize(value);
    let bytes = serde_json::to_vec(&canonical).map_err(integrity)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys: Vec<&String> = object.keys().collect();
            keys.sort_unstable();
            let mut canonical = Map::new();
            for key in keys {
                canonical.insert(key.clone(), canonicalize(&object[key]));
            }
            Value::Object(canonical)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        scalar => scalar.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(entries: Vec<(&str, Value)>) -> Value {
        let mut value = Map::new();
        for (key, child) in entries {
            value.insert(key.to_string(), child);
        }
        Value::Object(value)
    }

    #[test]
    fn digest_is_stable_across_recursive_object_order() {
        let left = object(vec![
            (
                "receipt",
                object(vec![("z", Value::from(1)), ("a", Value::from(2))]),
            ),
            ("request_id", Value::from("request-one")),
        ]);
        let right = object(vec![
            ("request_id", Value::from("request-one")),
            (
                "receipt",
                object(vec![("a", Value::from(2)), ("z", Value::from(1))]),
            ),
        ]);

        assert_eq!(json_digest(&left), json_digest(&right));
        assert_eq!(
            serde_json::to_string(&canonicalize(&left)).unwrap(),
            r#"{"receipt":{"a":2,"z":1},"request_id":"request-one"}"#
        );
    }
}
