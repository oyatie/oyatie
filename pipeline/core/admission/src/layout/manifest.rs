//! Cargo identity and dependency-shape checks for changed repository manifests.

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
    let package = manifest.get("package");
    let discovery_switch = if face == "facade" {
        "autobins"
    } else {
        "autolib"
    };
    if package
        .and_then(|package| package.get(discovery_switch))
        .and_then(toml::Value::as_bool)
        == Some(false)
    {
        violations.push(format!(
            "{path}: package `{discovery_switch} = false` disables the canonical face target"
        ));
    }
    if package
        .and_then(|package| package.get("autotests"))
        .and_then(toml::Value::as_bool)
        == Some(false)
    {
        violations.push(format!(
            "{path}: package `autotests = false` disables required integration-test discovery"
        ));
    }
    for switch in ["test", "harness"] {
        if manifest
            .get("lib")
            .and_then(|library| library.get(switch))
            .and_then(toml::Value::as_bool)
            == Some(false)
        {
            violations.push(format!(
                "{path}: `[lib].{switch} = false` disables required test discovery"
            ));
        }
    }
    if let Some(build) = manifest
        .get("package")
        .and_then(|package| package.get("build"))
    {
        match build.as_str() {
            Some("build.rs") => {}
            Some(build_path) => violations.push(format!(
                "{path}: package build target must be the stable item-scanner `build.rs`, got `{build_path}`"
            )),
            None => violations.push(format!(
                "{path}: package build target must be the stable item-scanner `build.rs`; boolean or non-string overrides are forbidden"
            )),
        }
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

/// Map any path below a canonical face leaf back to that crate's manifest.
pub fn cargo_manifest_for_crate_path(path: &str) -> Option<String> {
    let parts = path_parts(path);
    (1..=parts.len()).find_map(|end| {
        let manifest = format!("{}/Cargo.toml", parts[..end].join("/"));
        if expected_manifest_identity(&manifest).is_some() {
            Some(manifest)
        } else {
            None
        }
    })
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
    if !super::inner::crate_leaf_ok(face, leaf) {
        return None;
    }
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
            "[package]\nname='network-route'\nbuild=false\n",
            "[package]\nname='network-route'\nautolib=false\n",
            "[package]\nname='network-route'\nautotests=false\n",
            "[package]\nname='network-route'\n[lib]\ntest=false\n",
            "[package]\nname='network-route'\n[lib]\nharness=false\n",
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
    fn every_path_below_a_face_leaf_maps_to_its_manifest() {
        for path in [
            "network/ports/blob/tests/fixture.rs",
            "network/ports/blob/OWNERS",
            "network/ports/blob/build.rs",
            "network/ports/draft/blob/src/lib.rs",
            "app/drive/adapters/blob-s3/tests/live.rs",
        ] {
            assert!(cargo_manifest_for_crate_path(path).is_some(), "{path}");
        }
        assert!(
            cargo_manifest_for_crate_path("network/facade/proto/network/api/v1/a.proto").is_none()
        );
        assert!(
            cargo_manifest_for_crate_path("build/port-engine/core/analysis/src/lib.rs").is_none()
        );
    }
}
