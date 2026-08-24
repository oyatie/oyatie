//! Cargo identity and dependency-shape checks for changed ADR-0719 manifests.

use std::collections::BTreeSet;

use super::dependency::resolved_dependency_path;
use super::{is_capability_root, path_parts};

pub fn cargo_manifest_violations(path: &str, contents: &str) -> Vec<String> {
    let Some((expected_name, face)) = expected_manifest_identity(path) else {
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
    if let Some(lib_path) = manifest
        .get("lib")
        .and_then(|library| library.get("path"))
        .and_then(toml::Value::as_str)
        && lib_path != "src/lib.rs"
    {
        violations.push(format!(
            "{path}: `[lib].path` must be `src/lib.rs`, got `{lib_path}`"
        ));
    }
    for target in ["bin", "example", "bench", "test"] {
        if manifest.get(target).is_some() {
            violations.push(format!(
                "{path}: explicit `[[{target}]]` targets bypass the canonical face entry point"
            ));
        }
    }
    if face == "facade"
        && manifest
            .get("package")
            .and_then(|package| package.get("autobins"))
            .and_then(toml::Value::as_bool)
            == Some(false)
    {
        violations.push(format!(
            "{path}: facade packages must discover the canonical `src/main.rs` target"
        ));
    }
    if let Some(build_path) = manifest
        .get("package")
        .and_then(|package| package.get("build"))
        .and_then(toml::Value::as_str)
        && build_path != "build.rs"
    {
        violations.push(format!(
            "{path}: package build target must be the D-41 `build.rs`, got `{build_path}`"
        ));
    }
    violations
}

pub fn cargo_entrypoint(path: &str) -> Option<String> {
    let (_, face) = expected_manifest_identity(path)?;
    let directory = path.strip_suffix("/Cargo.toml")?;
    let source = if face == "facade" {
        "src/main.rs"
    } else {
        "src/lib.rs"
    };
    Some(format!("{directory}/{source}"))
}

pub fn cargo_manifest_for_entrypoint(path: &str) -> Option<String> {
    let directory = path
        .strip_suffix("/src/lib.rs")
        .or_else(|| path.strip_suffix("/src/main.rs"))?;
    let manifest = format!("{directory}/Cargo.toml");
    (cargo_entrypoint(&manifest).as_deref() == Some(path)).then_some(manifest)
}

/// A first `base/` crate is below the capability graph only when at least
/// three distinct capability manifests consume it as a production path
/// dependency in the same reviewed change.
pub fn base_admission_violations(manifests: &[(String, String)]) -> Vec<String> {
    let consumers: BTreeSet<&str> = manifests
        .iter()
        .filter_map(|(path, contents)| {
            let parts = path_parts(path);
            let owner = *parts.first()?;
            if !is_capability_root(owner) {
                return None;
            }
            let manifest = contents.parse::<toml::Value>().ok()?;
            production_dependency_targets_base(path, &manifest).then_some(owner)
        })
        .collect();
    if consumers.len() >= 3 {
        Vec::new()
    } else {
        vec![format!(
            "base: first crate requires production path dependencies from at least three distinct capabilities in the same change; found {} ({})",
            consumers.len(),
            consumers.iter().copied().collect::<Vec<_>>().join(", ")
        )]
    }
}

fn production_dependency_targets_base(path: &str, manifest: &toml::Value) -> bool {
    manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .is_some_and(|dependencies| {
            dependencies.values().any(|dependency| {
                dependency
                    .get("path")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|dependency_path| resolves_under_base(path, dependency_path))
            })
        })
}

fn resolves_under_base(manifest_path: &str, dependency_path: &str) -> bool {
    let Ok(components) = resolved_dependency_path(manifest_path, dependency_path) else {
        return false;
    };
    components
        .first()
        .is_some_and(|component| component == "base")
        && components
            .get(1)
            .is_some_and(|component| component == "core")
        && components.get(2).is_some_and(|leaf| !leaf.is_empty())
}

pub(super) fn expected_manifest_identity(path: &str) -> Option<(String, &str)> {
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
    Some((format!("{owner}-{leaf}{suffix}"), face))
}

#[cfg(test)]
fn expected_package_name(path: &str) -> Option<String> {
    expected_manifest_identity(path).map(|(name, _)| name)
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

    #[test]
    fn custom_target_paths_are_fail_closed() {
        let path = "network/core/route/Cargo.toml";
        for manifest in [
            "[package]\nname='network-route'\n[lib]\npath='src/other.rs'\n",
            "[package]\nname='network-route'\n[[bin]]\nname='route'\npath='src/route.rs'\n",
            "[package]\nname='network-route'\nbuild='tools/generate.rs'\n",
        ] {
            assert!(!cargo_manifest_violations(path, manifest).is_empty());
        }
    }

    #[test]
    fn canonical_entrypoint_follows_the_face() {
        assert_eq!(
            cargo_entrypoint("network/core/route/Cargo.toml").as_deref(),
            Some("network/core/route/src/lib.rs")
        );
        assert_eq!(
            cargo_entrypoint("network/facade/edge-app/Cargo.toml").as_deref(),
            Some("network/facade/edge-app/src/main.rs")
        );
        assert_eq!(
            cargo_manifest_for_entrypoint("network/facade/edge-app/src/main.rs").as_deref(),
            Some("network/facade/edge-app/Cargo.toml")
        );
        assert!(cargo_manifest_for_entrypoint("network/facade/edge-app/src/lib.rs").is_none());
    }

    #[test]
    fn base_requires_three_distinct_production_consumers() {
        let manifests = ["network", "storage", "compute"].map(|owner| {
            (
                format!("{owner}/core/engine/Cargo.toml"),
                format!(
                    "[package]\nname='{owner}-engine'\n[dependencies]\nbase-bytes={{path='../../../base/core/bytes'}}\n"
                ),
            )
        });
        assert!(base_admission_violations(&manifests[..2]).len() == 1);
        assert!(base_admission_violations(&manifests).is_empty());
    }
}
