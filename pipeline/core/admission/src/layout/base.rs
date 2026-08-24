//! Three-capability admission for each new `base/core/<leaf>` primitive.

use std::collections::BTreeSet;

use super::dependency::resolved_dependency_path;
use super::{is_capability_root, path_parts};

pub fn base_admission_violations(
    base_manifest: &str,
    manifests: &[(String, String)],
    workspace_contents: &str,
) -> Vec<String> {
    let Some(target) = base_target(base_manifest) else {
        return vec![format!("{base_manifest}: invalid base core manifest path")];
    };
    let workspace_dependencies = workspace_contents
        .parse::<toml::Value>()
        .ok()
        .and_then(|workspace| {
            workspace
                .get("workspace")?
                .get("dependencies")?
                .as_table()
                .cloned()
        })
        .unwrap_or_default();
    let consumers: BTreeSet<&str> = manifests
        .iter()
        .filter_map(|(path, contents)| {
            let parts = path_parts(path);
            let owner = *parts.first()?;
            if !is_capability_root(owner) {
                return None;
            }
            let manifest = contents.parse::<toml::Value>().ok()?;
            production_dependency_targets(path, &manifest, &workspace_dependencies, &target)
                .then_some(owner)
        })
        .collect();
    if consumers.len() >= 3 {
        Vec::new()
    } else {
        vec![format!(
            "{}: new base primitive requires production path dependencies from at least three distinct capabilities in the same change; found {} ({})",
            target.join("/"),
            consumers.len(),
            consumers.iter().copied().collect::<Vec<_>>().join(", ")
        )]
    }
}

fn base_target(manifest: &str) -> Option<Vec<String>> {
    let parts = path_parts(manifest);
    (parts.len() == 4
        && parts[0] == "base"
        && parts[1] == "core"
        && !parts[2].is_empty()
        && parts[3] == "Cargo.toml")
        .then(|| parts[..3].iter().map(|part| (*part).to_owned()).collect())
}

fn production_dependency_targets(
    manifest_path: &str,
    manifest: &toml::Value,
    workspace_dependencies: &toml::map::Map<String, toml::Value>,
    target: &[String],
) -> bool {
    if dependency_table_targets(
        manifest_path,
        manifest.get("dependencies"),
        workspace_dependencies,
        target,
    ) {
        return true;
    }
    manifest
        .get("target")
        .and_then(toml::Value::as_table)
        .is_some_and(|targets| {
            targets.values().any(|target_manifest| {
                dependency_table_targets(
                    manifest_path,
                    target_manifest.get("dependencies"),
                    workspace_dependencies,
                    target,
                )
            })
        })
}

fn dependency_table_targets(
    manifest_path: &str,
    dependencies: Option<&toml::Value>,
    workspace_dependencies: &toml::map::Map<String, toml::Value>,
    target: &[String],
) -> bool {
    dependencies
        .and_then(toml::Value::as_table)
        .is_some_and(|dependencies| {
            dependencies.iter().any(|(name, dependency)| {
                let (declaring_manifest, dependency_path) = if let Some(path) =
                    dependency.get("path").and_then(toml::Value::as_str)
                {
                    (manifest_path, path)
                } else if dependency.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                    let Some(path) = workspace_dependencies
                        .get(name)
                        .and_then(|entry| entry.get("path"))
                        .and_then(toml::Value::as_str)
                    else {
                        return false;
                    };
                    ("Cargo.toml", path)
                } else {
                    return false;
                };
                resolved_dependency_path(declaring_manifest, dependency_path)
                    .is_ok_and(|components| components == target)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consumer(owner: &str, leaf: &str) -> (String, String) {
        (
            format!("{owner}/core/engine/Cargo.toml"),
            format!(
                "[package]\nname='{owner}-engine'\n[dependencies]\nbase={{path='../../../base/core/{leaf}'}}\n"
            ),
        )
    }

    #[test]
    fn each_base_leaf_requires_three_exact_consumers() {
        let split = [
            consumer("network", "bytes"),
            consumer("storage", "ids"),
            consumer("compute", "time"),
        ];
        assert_eq!(
            base_admission_violations("base/core/bytes/Cargo.toml", &split, "").len(),
            1
        );
        let shared = ["network", "storage", "compute"].map(|owner| consumer(owner, "bytes"));
        assert!(base_admission_violations("base/core/bytes/Cargo.toml", &shared, "").is_empty());
    }

    #[test]
    fn target_specific_workspace_dependencies_count_as_production() {
        let manifests = ["network", "storage", "compute"].map(|owner| {
            (
                format!("{owner}/core/engine/Cargo.toml"),
                format!(
                    "[package]\nname='{owner}-engine'\n[target.'cfg(unix)'.dependencies]\nbase.workspace=true\n"
                ),
            )
        });
        let workspace = "[workspace]\n[workspace.dependencies]\nbase={path='base/core/bytes'}\n";
        assert!(
            base_admission_violations("base/core/bytes/Cargo.toml", &manifests, workspace)
                .is_empty()
        );
    }
}
