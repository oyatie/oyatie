use super::*;

pub(super) fn build_user_schemas(input: &NewUser) -> Vec<String> {
    let mut s = vec![User::CORE_SCHEMA.to_owned()];
    if input.enterprise.is_some() {
        s.push(User::ENTERPRISE_SCHEMA.to_owned());
    }
    if input.oyatie.is_some() {
        s.push(User::OYATIE_SCHEMA.to_owned());
    }
    s
}

pub(super) fn validate_native_binding(
    extension: Option<&OyatieExtension>,
) -> Result<(), ScimError> {
    let Some(extension) = extension else {
        return Ok(());
    };
    if let Some(native_user_id) = &extension.native_user_id {
        UserId::new(native_user_id.clone()).map_err(|_| {
            ScimError::new(400, Some(ScimType::InvalidValue), "nativeUserId is invalid")
        })?;
    }
    if let Some(roles) = &extension.roles {
        if extension.native_user_id.is_none() {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidValue),
                "roles require nativeUserId",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for role in roles {
            if !provisioned_role_is_valid(role) {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "role identifier is invalid",
                ));
            }
            if !seen.insert(role) {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "duplicate role identifier",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn apply_patch_user(user: &mut User, op: &PatchOperation) -> Result<(), ScimError> {
    let path = op.path.as_deref().unwrap_or("");
    match (op.op, path) {
        (
            PatchOpKind::Add | PatchOpKind::Replace,
            "urn:oyatie:scim:extension:2.0:User:nativeUserId",
        ) => {
            let value = op
                .value
                .as_ref()
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    ScimError::new(
                        400,
                        Some(ScimType::InvalidValue),
                        "nativeUserId expects string",
                    )
                })?;
            user.oyatie
                .get_or_insert_with(OyatieExtension::default)
                .native_user_id = Some(value.to_owned());
            Ok(())
        }
        (PatchOpKind::Remove, "urn:oyatie:scim:extension:2.0:User:nativeUserId") => {
            if let Some(extension) = user.oyatie.as_mut() {
                extension.native_user_id = None;
            }
            Ok(())
        }
        (PatchOpKind::Add | PatchOpKind::Replace, "urn:oyatie:scim:extension:2.0:User:roles") => {
            let roles: Vec<String> = serde_json::from_value(op.value.clone().ok_or_else(|| {
                ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "roles expects string array",
                )
            })?)
            .map_err(|_| {
                ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "roles expects string array",
                )
            })?;
            user.oyatie
                .get_or_insert_with(OyatieExtension::default)
                .roles = Some(roles);
            Ok(())
        }
        (PatchOpKind::Remove, "urn:oyatie:scim:extension:2.0:User:roles") => {
            if let Some(extension) = user.oyatie.as_mut() {
                extension.roles = None;
            }
            Ok(())
        }
        (PatchOpKind::Replace, "active") => {
            let v = op.value.as_ref().and_then(|v| v.as_bool()).ok_or_else(|| {
                ScimError::new(400, Some(ScimType::InvalidValue), "active expects bool")
            })?;
            user.active = v;
            Ok(())
        }
        (PatchOpKind::Replace, "displayName") => {
            let v = op.value.as_ref().and_then(|v| v.as_str()).map(String::from);
            user.display_name = v;
            Ok(())
        }
        (PatchOpKind::Replace, "userName") => {
            let v = op.value.as_ref().and_then(|v| v.as_str()).ok_or_else(|| {
                ScimError::new(400, Some(ScimType::InvalidValue), "userName expects string")
            })?;
            user.user_name = v.to_owned();
            Ok(())
        }
        (PatchOpKind::Add, "emails") => {
            let arr = op
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    ScimError::new(400, Some(ScimType::InvalidValue), "emails expects array")
                })?;
            for e in arr {
                let parsed: Email = serde_json::from_value(e.clone()).map_err(|err| {
                    ScimError::new(400, Some(ScimType::InvalidSyntax), err.to_string())
                })?;
                user.emails.push(parsed);
            }
            Ok(())
        }
        (PatchOpKind::Remove, p) if p.starts_with("emails[") => {
            // Best-effort: only support emails[value eq "..."]; remove matching.
            if let Some(needle) = parse_filter_value_in_brackets(p) {
                user.emails.retain(|e| e.value != needle);
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
