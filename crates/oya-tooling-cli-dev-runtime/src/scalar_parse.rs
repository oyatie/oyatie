use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn parse_u32_field(value: &str, name: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .map_err(|_| format!("{name} must be an unsigned 32-bit integer"))
}

pub(crate) fn parse_u64_field(value: &str, name: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("{name} must be an unsigned 64-bit integer"))
}

pub(crate) fn required_scalar(path: &Path, contents: &str, key: &str) -> Result<String, String> {
    let mut found = None;
    for line in contents.lines() {
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(value) = scalar_value(trimmed, key) {
            if found.is_some() {
                return Err(format!("{}: duplicate field {key}", path.display()));
            }
            found = Some(value);
        }
    }
    found.ok_or_else(|| format!("{}: missing required field {key}", path.display()))
}

pub(crate) fn insert_scalar_field(
    path: &Path,
    fields: &mut BTreeMap<String, String>,
    line: &str,
) -> Result<(), String> {
    let Some((key, value)) = line.split_once(':') else {
        return Err(format!(
            "{}: expected key: value line: {line}",
            path.display()
        ));
    };
    let key = key.trim();
    if fields.contains_key(key) {
        return Err(format!("{}: duplicate field {key}", path.display()));
    }
    fields.insert(key.to_string(), clean_scalar_value(value));
    Ok(())
}

pub(crate) fn required_field(
    path: &Path,
    fields: &BTreeMap<String, String>,
    key: &str,
) -> Result<String, String> {
    fields
        .get(key)
        .cloned()
        .ok_or_else(|| format!("{}: missing required field {key}", path.display()))
}

pub(crate) fn scalar_value(line: &str, key: &str) -> Option<String> {
    let (actual_key, value) = line.split_once(':')?;
    if actual_key.trim() == key {
        Some(clean_scalar_value(value))
    } else {
        None
    }
}

pub(crate) fn clean_scalar_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

pub(crate) fn parse_u8_percent(path: &Path, field: &str, value: &str) -> Result<u8, String> {
    let parsed = value
        .parse::<u8>()
        .map_err(|_| format!("{}: {field} must be 0-100 integer", path.display()))?;
    if parsed > 100 {
        Err(format!("{}: {field} must be 0-100 integer", path.display()))
    } else {
        Ok(parsed)
    }
}

pub(crate) fn parse_bool_field(path: &Path, field: &str, value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("{}: {field} must be true or false", path.display())),
    }
}
