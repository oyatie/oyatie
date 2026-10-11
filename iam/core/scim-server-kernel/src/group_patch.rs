use super::*;

pub(super) fn apply_patch_group(group: &mut Group, op: &PatchOperation) -> Result<(), ScimError> {
    let path = op.path.as_deref().unwrap_or("");
    match (op.op, path) {
        (PatchOpKind::Replace, "displayName") => {
            let v = op.value.as_ref().and_then(|v| v.as_str()).ok_or_else(|| {
                ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "displayName expects string",
                )
            })?;
            group.display_name = v.to_owned();
            Ok(())
        }
        (PatchOpKind::Add, "members") => {
            let arr = op
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    ScimError::new(400, Some(ScimType::InvalidValue), "members expects array")
                })?;
            for m in arr {
                let parsed: GroupMembership = serde_json::from_value(m.clone()).map_err(|err| {
                    ScimError::new(400, Some(ScimType::InvalidSyntax), err.to_string())
                })?;
                group.members.push(parsed);
            }
            Ok(())
        }
        (PatchOpKind::Remove, p) if p.starts_with("members[") => {
            if let Some(needle) = parse_filter_value_in_brackets(p) {
                group.members.retain(|m| m.value.0 != needle);
            }
            Ok(())
        }
        (op_kind, p) => Err(ScimError::new(
            400,
            Some(ScimType::InvalidPath),
            format!("unsupported {op_kind:?} on path '{p}'"),
        )),
    }
}

/// Extract the bare value out of `members[value eq "abc"]` → `Some("abc")`.
pub(super) fn parse_filter_value_in_brackets(p: &str) -> Option<String> {
    let lb = p.find('[')?;
    let rb = p.rfind(']')?;
    let inner = &p[lb + 1..rb];
    // Look for `eq` and quoted value.
    let q1 = inner.find('"')?;
    let q2 = inner.rfind('"')?;
    if q1 < q2 {
        Some(inner[q1 + 1..q2].to_owned())
    } else {
        None
    }
}
