use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn mailbox_creations(
    objects: &serde_json::Map<String, Value>,
) -> Vec<(&String, &Value)> {
    let mut pending: Vec<_> = objects.iter().collect();
    let mut ordered = vec![];
    let mut visited = std::collections::BTreeSet::new();
    while !pending.is_empty() {
        let next = pending.iter().position(|(_, value)| {
            value["parentId"]
                .as_str()
                .and_then(|s| s.strip_prefix('#'))
                .is_none_or(|id| !objects.contains_key(id) || visited.contains(id))
        });
        let Some(index) = next else {
            ordered.extend(pending);
            break;
        };
        let entry = pending.remove(index);
        visited.insert(entry.0.as_str());
        ordered.push(entry);
    }
    ordered
}

pub(super) fn created(value: &mut Value, ids: &BTreeMap<String, String>) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if matches!(key.as_str(), "mailboxIds" | "update") {
                    if let Some(map) = value.as_object_mut() {
                        *map = std::mem::take(map)
                            .into_iter()
                            .map(|(key, value)| {
                                (
                                    key.strip_prefix('#')
                                        .and_then(|key| ids.get(key))
                                        .cloned()
                                        .unwrap_or(key),
                                    value,
                                )
                            })
                            .collect();
                    }
                } else if matches!(
                    key.as_str(),
                    "id" | "blobId" | "parentId" | "emailId" | "identityId"
                ) && let Some(id) = value
                    .as_str()
                    .and_then(|s| s.strip_prefix('#'))
                    .and_then(|id| ids.get(id))
                {
                    *value = Value::String(id.clone());
                }
                if key != "bodyValues" {
                    created(value, ids);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                created(value, ids);
            }
        }
        _ => {}
    }
}

pub(super) fn resolve(args: &Value, responses: &[Value]) -> Result<Value, &'static str> {
    let mut remaining = super::MAX_REQUEST;
    super::limits::charge(args, &mut remaining)?;
    let mut result = args.as_object().cloned().ok_or("invalidArguments")?;
    for (key, reference) in args.as_object().ok_or("invalidArguments")? {
        let Some(target) = key.strip_prefix('#') else {
            continue;
        };
        if result.contains_key(target) {
            return Err("invalidArguments");
        }
        let id = reference["resultOf"]
            .as_str()
            .ok_or("invalidResultReference")?;
        let name = reference["name"].as_str().ok_or("invalidResultReference")?;
        let path = reference["path"].as_str().ok_or("invalidResultReference")?;
        let response = responses
            .iter()
            .rev()
            .find(|r| r[2] == id && r[0] == name)
            .ok_or("invalidResultReference")?;
        result.insert(target.into(), pointer(&response[1], path, &mut remaining)?);
        result.remove(key);
    }
    Ok(Value::Object(result))
}

fn pointer(value: &Value, path: &str, remaining: &mut usize) -> Result<Value, &'static str> {
    if let Some((head, tail)) = path.split_once("/*") {
        if !tail.is_empty() && !tail.starts_with('/') {
            return Err("invalidResultReference");
        }
        let list = value
            .pointer(head)
            .and_then(Value::as_array)
            .ok_or("invalidResultReference")?;
        let values = list
            .iter()
            .map(|v| pointer(v, tail, remaining))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Array(values))
    } else {
        let value = value.pointer(path).ok_or("invalidResultReference")?;
        super::limits::charge(value, remaining)?;
        Ok(value.clone())
    }
}
