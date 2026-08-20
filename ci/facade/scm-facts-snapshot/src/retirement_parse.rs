//! Git tree / OID / closed-JSON parsers for retirement facts.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde_json::Value;

use super::TreeEntry;

pub(crate) fn parse_ls_tree(bytes: &[u8]) -> Result<Vec<TreeEntry>, String> {
    let mut entries = Vec::new();
    for record in bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| "git ls-tree record has no path separator".to_owned())?;
        let header = std::str::from_utf8(&record[..tab])
            .map_err(|error| format!("git ls-tree header is not UTF-8: {error}"))?;
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|error| format!("git path is not UTF-8: {error}"))?;
        let mut fields = header.split_ascii_whitespace();
        let mode = fields.next().unwrap_or_default();
        let kind = fields.next().unwrap_or_default();
        let oid = fields.next().unwrap_or_default();
        if fields.next().is_some() || mode.is_empty() || kind.is_empty() {
            return Err("git ls-tree record has invalid metadata".to_owned());
        }
        validate_oid(oid, "git tree object")?;
        entries.push(TreeEntry {
            mode: mode.to_owned(),
            kind: kind.to_owned(),
            oid: oid.to_owned(),
            path: path.to_owned(),
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

pub(crate) fn parse_oid_text(bytes: &[u8], label: &str) -> Result<String, String> {
    let value = std::str::from_utf8(bytes)
        .map_err(|error| format!("{label} is not UTF-8: {error}"))?
        .trim();
    validate_oid(value, label)?;
    Ok(value.to_owned())
}

pub(crate) fn validate_oid(value: &str, label: &str) -> Result<(), String> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(format!("{label} is not a lowercase SHA-1 object id"))
    }
}

pub(crate) fn validate_sha256(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err("retirement SHA-256 must use sha256: prefix".to_owned());
    };
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err("retirement SHA-256 is not canonical lowercase hex".to_owned())
    }
}

pub(crate) fn validate_repo_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with("./")
        || path.contains("//")
        || path.split('/').any(|part| part == ".." || part.is_empty())
        || path.contains('\0')
    {
        Err(format!(
            "retirement path {path:?} is not canonical repo-relative"
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn required_value_string<'a>(
    value: Option<&'a Value>,
    label: &str,
) -> Result<&'a str, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("retirement {label} is missing"))
}

pub(crate) fn parse_closed_json<T>(bytes: &[u8]) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut duplicate_deserializer = serde_json::Deserializer::from_slice(bytes);
    DuplicateKeyFreeJson::deserialize(&mut duplicate_deserializer)
        .map_err(|error| format!("retirement JSON duplicate-key check: {error}"))?;
    duplicate_deserializer
        .end()
        .map_err(|error| format!("retirement JSON trailing data: {error}"))?;
    serde_json::from_slice(bytes).map_err(|error| format!("retirement JSON parse: {error}"))
}

pub(crate) struct DuplicateKeyFreeJson;

impl<'de> Deserialize<'de> for DuplicateKeyFreeJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateKeyFreeJsonVisitor)
    }
}

pub(crate) struct DuplicateKeyFreeJsonVisitor;

impl<'de> Visitor<'de> for DuplicateKeyFreeJsonVisitor {
    type Value = DuplicateKeyFreeJson;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DuplicateKeyFreeJson::deserialize(deserializer)
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element::<DuplicateKeyFreeJson>()?.is_some() {}
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate object key: {key}")));
            }
            map.next_value::<DuplicateKeyFreeJson>()?;
        }
        Ok(DuplicateKeyFreeJson)
    }
}
