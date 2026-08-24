//! Cargo identity checks for changed ADR-0719 D-30 crate manifests.

use super::{is_capability_root, path_parts};

pub fn cargo_manifest_violations(path: &str, contents: &str) -> Vec<String> {
    let Some(expected_name) = expected_package_name(path) else {
        return Vec::new();
    };
    let manifest = match contents.parse::<toml::Value>() {
        Ok(manifest) => manifest,
        Err(error) => return vec![format!("{path}: invalid Cargo manifest: {error}")],
    };
    let mut violations = Vec::new();
    let package_name = manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str);
    if package_name != Some(expected_name.as_str()) {
        violations.push(format!(
            "{path}: package name must be `{expected_name}`, got {}",
            package_name.unwrap_or("<missing or non-string>")
        ));
    }
    if manifest
        .get("lib")
        .and_then(|library| library.get("name"))
        .is_some()
    {
        violations.push(format!(
            "{path}: `[lib].name` must be omitted so Cargo derives the crate name"
        ));
    }
    violations
}

fn expected_package_name(path: &str) -> Option<String> {
    let parts = path_parts(path);
    let (owner, face_index) = if parts.first() == Some(&"app") {
        (*parts.get(1)?, 2)
    } else {
        let owner = *parts.first()?;
        if owner != "base" && !is_capability_root(owner) {
            return None;
        }
        (owner, 1)
    };
    let face = *parts.get(face_index)?;
    if !matches!(face, "core" | "ports" | "adapters" | "facade") {
        return None;
    }
    let mut leaf_index = face_index + 1;
    let draft = matches!(face, "ports" | "adapters") && parts.get(leaf_index) == Some(&"draft");
    if draft {
        leaf_index += 1;
    }
    if parts.len() != leaf_index + 2 || parts.last() != Some(&"Cargo.toml") {
        return None;
    }
    let leaf = parts[leaf_index];
    let suffix = if draft { "-draft" } else { "" };
    Some(format!("{owner}-{leaf}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_names_follow_owner_face_and_draft_identity() {
        let cases = [
            ("network/core/route/Cargo.toml", "network-route"),
            ("network/ports/blob/Cargo.toml", "network-blob"),
            ("network/ports/draft/blob/Cargo.toml", "network-blob-draft"),
            ("app/drive/adapters/blob-s3/Cargo.toml", "drive-blob-s3"),
            ("app/drive/facade/app/Cargo.toml", "drive-app"),
        ];
        for (path, expected) in cases {
            assert_eq!(
                expected_package_name(path).as_deref(),
                Some(expected),
                "{path}"
            );
        }
    }

    #[test]
    fn package_name_and_lib_override_are_fail_closed() {
        let path = "network/ports/blob/Cargo.toml";
        assert!(cargo_manifest_violations(path, "[package]\nname='network-blob'\n").is_empty());
        let violations =
            cargo_manifest_violations(path, "[package]\nname='anything'\n[lib]\nname='alias'\n");
        assert!(violations.iter().any(|item| item.contains("network-blob")));
        assert!(violations.iter().any(|item| item.contains("[lib].name")));
        assert!(!cargo_manifest_violations(path, "not = [valid").is_empty());
    }
}
