//! Config patching: strategic-merge patches and RFC6902 JSON-patch operations.
//!
//! Mirrors the Talos `configpatcher` package, which can apply either a strategic
//! merge patch (a partial document deep-merged into the config) or an ordered
//! list of RFC6902 JSON-patch operations (`add` / `remove` / `replace` / ...).
//!
//! To stay dependency-free and `serde`-free, the document model here is a small
//! JSON-pointer-addressable tree ([`Value`]) rather than a full YAML/JSON
//! parser. It is enough to model and unit-test the patch semantics faithfully.

use os_kernel::error::{Error, Result};
use std::collections::BTreeMap;

/// A minimal JSON-like document value, addressable by RFC6901 JSON pointer.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Integer scalar (sufficient for config values modeled here).
    Int(i64),
    /// String scalar.
    Str(String),
    /// Ordered array.
    Array(Vec<Value>),
    /// Object with string keys (sorted for deterministic iteration).
    Object(BTreeMap<String, Value>),
}

impl Value {
    /// Build an object value from key/value pairs.
    pub fn object<I, K>(pairs: I) -> Value
    where
        I: IntoIterator<Item = (K, Value)>,
        K: Into<String>,
    {
        Value::Object(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// Resolve a JSON pointer (RFC6901) to a reference, if it exists.
    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        if pointer.is_empty() {
            return Some(self);
        }
        let mut cur = self;
        for token in pointer.trim_start_matches('/').split('/') {
            let token = unescape_token(token);
            cur = match cur {
                Value::Object(map) => map.get(&token)?,
                Value::Array(arr) => {
                    let idx: usize = token.parse().ok()?;
                    arr.get(idx)?
                }
                _ => return None,
            };
        }
        Some(cur)
    }
}

/// Unescape an RFC6901 pointer token (`~1` -> `/`, `~0` -> `~`).
fn unescape_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// A single RFC6902 JSON-patch operation kind.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonPatchOp {
    /// Add (or, for an existing object key, replace) a value at the path.
    Add { path: String, value: Value },
    /// Remove the value at the path.
    Remove { path: String },
    /// Replace an existing value at the path.
    Replace { path: String, value: Value },
    /// Test that the value at the path equals `value`; fails the patch if not.
    Test { path: String, value: Value },
    /// Move the value at `from` to `path` (remove then add).
    Move { from: String, path: String },
    /// Copy the value at `from` to `path` (read then add).
    Copy { from: String, path: String },
}

impl JsonPatchOp {
    /// The target path of the operation.
    pub fn path(&self) -> &str {
        match self {
            JsonPatchOp::Add { path, .. }
            | JsonPatchOp::Remove { path }
            | JsonPatchOp::Replace { path, .. }
            | JsonPatchOp::Test { path, .. }
            | JsonPatchOp::Move { path, .. }
            | JsonPatchOp::Copy { path, .. } => path,
        }
    }

    /// The operation name as it appears in an RFC6902 patch document.
    pub fn op_name(&self) -> &'static str {
        match self {
            JsonPatchOp::Add { .. } => "add",
            JsonPatchOp::Remove { .. } => "remove",
            JsonPatchOp::Replace { .. } => "replace",
            JsonPatchOp::Test { .. } => "test",
            JsonPatchOp::Move { .. } => "move",
            JsonPatchOp::Copy { .. } => "copy",
        }
    }
}

/// Split a pointer into the parent pointer and the final key token.
fn split_pointer(pointer: &str) -> Result<(String, String)> {
    if pointer.is_empty() || !pointer.starts_with('/') {
        return Err(Error::invalid(format!("invalid json pointer '{pointer}'")));
    }
    let trimmed = pointer.trim_start_matches('/');
    match trimmed.rsplit_once('/') {
        Some((parent, last)) => Ok((format!("/{parent}"), unescape_token(last))),
        None => Ok((String::new(), unescape_token(trimmed))),
    }
}

/// Resolve a mutable reference to the parent container of `pointer`.
fn parent_mut<'a>(root: &'a mut Value, parent_ptr: &str) -> Option<&'a mut Value> {
    if parent_ptr.is_empty() {
        return Some(root);
    }
    let mut cur = root;
    for token in parent_ptr.trim_start_matches('/').split('/') {
        let token = unescape_token(token);
        cur = match cur {
            Value::Object(map) => map.get_mut(&token)?,
            Value::Array(arr) => {
                let idx: usize = token.parse().ok()?;
                arr.get_mut(idx)?
            }
            _ => return None,
        };
    }
    Some(cur)
}

/// A patch to apply to a config document: either a strategic merge or an ordered
/// list of JSON-patch operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigPatch {
    /// A strategic merge patch: deep-merge the overlay object into the base.
    StrategicMerge(Value),
    /// An ordered list of RFC6902 operations.
    Json(Vec<JsonPatchOp>),
}

/// A reusable alias for the JSON-patch op enum exported at the crate root.
pub type PatchOp = JsonPatchOp;

impl ConfigPatch {
    /// Apply the patch to `doc`, mutating it in place.
    pub fn apply(&self, doc: &mut Value) -> Result<()> {
        match self {
            ConfigPatch::StrategicMerge(overlay) => {
                strategic_merge(doc, overlay);
                Ok(())
            }
            ConfigPatch::Json(ops) => {
                for op in ops {
                    apply_op(doc, op)?;
                }
                Ok(())
            }
        }
    }
}

/// Deep-merge `overlay` into `base`. Objects merge key-by-key; every other kind
/// (scalars, arrays) replaces wholesale — matching Kubernetes strategic-merge
/// behavior for untagged lists.
fn strategic_merge(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (k, v) in overlay_map {
                match base_map.get_mut(k) {
                    Some(existing) => strategic_merge(existing, v),
                    None => {
                        base_map.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (base_slot, other) => {
            *base_slot = other.clone();
        }
    }
}

/// Remove and return the value at `path` from `doc`.
fn take_at(doc: &mut Value, path: &str) -> Result<Value> {
    let (parent_ptr, key) = split_pointer(path)?;
    let parent = parent_mut(doc, &parent_ptr)
        .ok_or_else(|| Error::not_found(format!("parent of '{path}' not found")))?;
    match parent {
        Value::Object(map) => map
            .remove(&key)
            .ok_or_else(|| Error::not_found(format!("key '{path}' not found"))),
        Value::Array(arr) => {
            let idx: usize = key
                .parse()
                .map_err(|_| Error::invalid(format!("invalid array index '{key}'")))?;
            if idx >= arr.len() {
                return Err(Error::not_found(format!(
                    "array index '{path}' out of range"
                )));
            }
            Ok(arr.remove(idx))
        }
        _ => Err(Error::invalid(format!(
            "cannot remove from scalar at '{parent_ptr}'"
        ))),
    }
}

/// Apply a single RFC6902 operation.
fn apply_op(doc: &mut Value, op: &JsonPatchOp) -> Result<()> {
    match op {
        JsonPatchOp::Move { from, path } => {
            let value = take_at(doc, from)?;
            apply_op(
                doc,
                &JsonPatchOp::Add {
                    path: path.clone(),
                    value,
                },
            )
        }
        JsonPatchOp::Copy { from, path } => {
            let value = doc
                .pointer(from)
                .cloned()
                .ok_or_else(|| Error::not_found(format!("copy source '{from}' not found")))?;
            apply_op(
                doc,
                &JsonPatchOp::Add {
                    path: path.clone(),
                    value,
                },
            )
        }
        JsonPatchOp::Test { path, value } => match doc.pointer(path) {
            Some(found) if found == value => Ok(()),
            Some(_) => Err(Error::invalid(format!("test failed at '{path}'"))),
            None => Err(Error::not_found(format!("test path '{path}' not found"))),
        },
        JsonPatchOp::Remove { path } => {
            let (parent_ptr, key) = split_pointer(path)?;
            let parent = parent_mut(doc, &parent_ptr)
                .ok_or_else(|| Error::not_found(format!("parent of '{path}' not found")))?;
            match parent {
                Value::Object(map) => {
                    map.remove(&key)
                        .ok_or_else(|| Error::not_found(format!("key '{path}' not found")))?;
                    Ok(())
                }
                Value::Array(arr) => {
                    let idx: usize = key
                        .parse()
                        .map_err(|_| Error::invalid(format!("invalid array index '{key}'")))?;
                    if idx >= arr.len() {
                        return Err(Error::not_found(format!(
                            "array index '{path}' out of range"
                        )));
                    }
                    arr.remove(idx);
                    Ok(())
                }
                _ => Err(Error::invalid(format!(
                    "cannot remove from scalar at '{parent_ptr}'"
                ))),
            }
        }
        JsonPatchOp::Add { path, value } | JsonPatchOp::Replace { path, value } => {
            let is_replace = matches!(op, JsonPatchOp::Replace { .. });
            let (parent_ptr, key) = split_pointer(path)?;
            let parent = parent_mut(doc, &parent_ptr)
                .ok_or_else(|| Error::not_found(format!("parent of '{path}' not found")))?;
            match parent {
                Value::Object(map) => {
                    if is_replace && !map.contains_key(&key) {
                        return Err(Error::not_found(format!(
                            "replace target '{path}' not found"
                        )));
                    }
                    map.insert(key, value.clone());
                    Ok(())
                }
                Value::Array(arr) => {
                    if key == "-" {
                        arr.push(value.clone());
                        return Ok(());
                    }
                    let idx: usize = key
                        .parse()
                        .map_err(|_| Error::invalid(format!("invalid array index '{key}'")))?;
                    if is_replace {
                        if idx >= arr.len() {
                            return Err(Error::not_found(format!(
                                "replace index '{path}' out of range"
                            )));
                        }
                        arr[idx] = value.clone();
                    } else {
                        if idx > arr.len() {
                            return Err(Error::invalid(format!("add index '{path}' out of range")));
                        }
                        arr.insert(idx, value.clone());
                    }
                    Ok(())
                }
                _ => Err(Error::invalid(format!(
                    "cannot add into scalar at '{parent_ptr}'"
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Value {
        Value::object([
            (
                "machine",
                Value::object([
                    ("type", Value::Str("worker".to_string())),
                    (
                        "install",
                        Value::object([("disk", Value::Str("/dev/sda".to_string()))]),
                    ),
                ]),
            ),
            ("ports", Value::Array(vec![Value::Int(80), Value::Int(443)])),
        ])
    }

    #[test]
    fn pointer_resolves_nested() {
        let d = doc();
        assert_eq!(
            d.pointer("/machine/type"),
            Some(&Value::Str("worker".to_string()))
        );
        assert_eq!(d.pointer("/ports/1"), Some(&Value::Int(443)));
        assert_eq!(d.pointer("/missing"), None);
    }

    #[test]
    fn strategic_merge_deep_merges_objects() {
        let mut d = doc();
        let overlay = Value::object([(
            "machine",
            Value::object([("type", Value::Str("controlplane".to_string()))]),
        )]);
        ConfigPatch::StrategicMerge(overlay).apply(&mut d).unwrap();
        assert_eq!(
            d.pointer("/machine/type"),
            Some(&Value::Str("controlplane".to_string()))
        );
        // The untouched nested key survives.
        assert_eq!(
            d.pointer("/machine/install/disk"),
            Some(&Value::Str("/dev/sda".to_string()))
        );
    }

    #[test]
    fn json_add_replace_remove() {
        let mut d = doc();
        let patch = ConfigPatch::Json(vec![
            JsonPatchOp::Add {
                path: "/machine/token".to_string(),
                value: Value::Str("t".to_string()),
            },
            JsonPatchOp::Replace {
                path: "/machine/type".to_string(),
                value: Value::Str("init".to_string()),
            },
            JsonPatchOp::Remove {
                path: "/ports/0".to_string(),
            },
        ]);
        patch.apply(&mut d).unwrap();
        assert_eq!(
            d.pointer("/machine/token"),
            Some(&Value::Str("t".to_string()))
        );
        assert_eq!(
            d.pointer("/machine/type"),
            Some(&Value::Str("init".to_string()))
        );
        assert_eq!(d.pointer("/ports/0"), Some(&Value::Int(443)));
    }

    #[test]
    fn json_append_to_array() {
        let mut d = doc();
        ConfigPatch::Json(vec![JsonPatchOp::Add {
            path: "/ports/-".to_string(),
            value: Value::Int(8080),
        }])
        .apply(&mut d)
        .unwrap();
        assert_eq!(d.pointer("/ports/2"), Some(&Value::Int(8080)));
    }

    #[test]
    fn replace_missing_target_fails() {
        let mut d = doc();
        let err = ConfigPatch::Json(vec![JsonPatchOp::Replace {
            path: "/machine/nope".to_string(),
            value: Value::Null,
        }])
        .apply(&mut d)
        .unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn test_op_gates_application() {
        let mut d = doc();
        let ok = ConfigPatch::Json(vec![JsonPatchOp::Test {
            path: "/machine/type".to_string(),
            value: Value::Str("worker".to_string()),
        }]);
        assert!(ok.apply(&mut d).is_ok());

        let bad = ConfigPatch::Json(vec![JsonPatchOp::Test {
            path: "/machine/type".to_string(),
            value: Value::Str("controlplane".to_string()),
        }]);
        assert!(bad.apply(&mut d).is_err());
    }

    #[test]
    fn op_path_accessor() {
        let op = JsonPatchOp::Remove {
            path: "/a/b".to_string(),
        };
        assert_eq!(op.path(), "/a/b");
    }

    #[test]
    fn json_move_relocates_value() {
        let mut d = doc();
        ConfigPatch::Json(vec![JsonPatchOp::Move {
            from: "/machine/install/disk".to_string(),
            path: "/machine/disk".to_string(),
        }])
        .apply(&mut d)
        .unwrap();
        assert_eq!(
            d.pointer("/machine/disk"),
            Some(&Value::Str("/dev/sda".to_string()))
        );
        assert_eq!(d.pointer("/machine/install/disk"), None);
    }

    #[test]
    fn json_copy_duplicates_value() {
        let mut d = doc();
        ConfigPatch::Json(vec![JsonPatchOp::Copy {
            from: "/machine/type".to_string(),
            path: "/machine/role".to_string(),
        }])
        .apply(&mut d)
        .unwrap();
        assert_eq!(
            d.pointer("/machine/role"),
            Some(&Value::Str("worker".to_string()))
        );
        // The source is preserved on copy.
        assert_eq!(
            d.pointer("/machine/type"),
            Some(&Value::Str("worker".to_string()))
        );
    }

    #[test]
    fn move_missing_source_fails() {
        let mut d = doc();
        let err = ConfigPatch::Json(vec![JsonPatchOp::Move {
            from: "/machine/nope".to_string(),
            path: "/machine/here".to_string(),
        }])
        .apply(&mut d)
        .unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn copy_missing_source_fails() {
        let mut d = doc();
        let err = ConfigPatch::Json(vec![JsonPatchOp::Copy {
            from: "/x".to_string(),
            path: "/y".to_string(),
        }])
        .apply(&mut d)
        .unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn op_name_strings() {
        assert_eq!(
            JsonPatchOp::Add {
                path: "/a".into(),
                value: Value::Null
            }
            .op_name(),
            "add"
        );
        assert_eq!(
            JsonPatchOp::Remove { path: "/a".into() }.op_name(),
            "remove"
        );
        assert_eq!(
            JsonPatchOp::Move {
                from: "/a".into(),
                path: "/b".into()
            }
            .op_name(),
            "move"
        );
        assert_eq!(
            JsonPatchOp::Copy {
                from: "/a".into(),
                path: "/b".into()
            }
            .op_name(),
            "copy"
        );
    }

    #[test]
    fn move_then_test_in_sequence() {
        let mut d = doc();
        ConfigPatch::Json(vec![
            JsonPatchOp::Move {
                from: "/ports/0".to_string(),
                path: "/firstPort".to_string(),
            },
            JsonPatchOp::Test {
                path: "/firstPort".to_string(),
                value: Value::Int(80),
            },
        ])
        .apply(&mut d)
        .unwrap();
        assert_eq!(d.pointer("/firstPort"), Some(&Value::Int(80)));
        // The remaining array shrank by one.
        assert_eq!(d.pointer("/ports/0"), Some(&Value::Int(443)));
        assert_eq!(d.pointer("/ports/1"), None);
    }
}
