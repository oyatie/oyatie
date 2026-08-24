//! Cross-owner dependency-stage checks for Cargo manifests.

use super::manifest::expected_manifest_identity;
use super::path_parts;

/// Reject dependencies that escape an owner's local draft contract stage.
/// Workspace-inherited path dependencies are resolved from the root manifest
/// so `workspace = true` cannot hide the coupling.
pub fn draft_dependency_violations(
    path: &str,
    contents: &str,
    workspace_contents: &str,
) -> Vec<String> {
    let consumer = manifest_owner(path);
    let Ok(manifest) = contents.parse::<toml::Value>() else {
        return Vec::new();
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
    let mut violations = Vec::new();
    visit_dependency_entries(&manifest, |name, dependency| {
        let (dependency_path, declaring_manifest) =
            if let Some(dependency_path) = dependency.get("path").and_then(toml::Value::as_str) {
                (dependency_path, path)
            } else if dependency.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
                let Some(dependency_path) = workspace_dependencies
                    .get(name)
                    .and_then(|dependency| dependency.get("path"))
                    .and_then(toml::Value::as_str)
                else {
                    return;
                };
                (dependency_path, "Cargo.toml")
            } else {
                return;
            };
        let components = match resolved_dependency_path(declaring_manifest, dependency_path) {
            Ok(components) => components,
            Err(reason) => {
                violations.push(format!(
                    "{path}: dependency `{name}` has invalid path `{dependency_path}`: {reason}"
                ));
                return;
            }
        };
        let Some(provider) = draft_dependency_owner(&components) else {
            return;
        };
        if consumer.as_deref() != Some(provider.as_str()) {
            let consumer = consumer.as_deref().unwrap_or("unclassified manifest");
            violations.push(format!(
                "{path}: dependency `{name}` crosses from `{consumer}` into owner-local draft `{provider}`"
            ));
        }
    });
    violations
}

/// Root workspace dependencies cannot point at any owner-local draft. A root
/// alias has no owner boundary, and would let an unchanged consumer inherit a
/// newly redirected draft dependency without changing its own manifest.
pub fn workspace_draft_dependency_violations(contents: &str) -> Vec<String> {
    let Ok(workspace) = contents.parse::<toml::Value>() else {
        return vec!["Cargo.toml: invalid workspace manifest".to_owned()];
    };
    let mut violations = Vec::new();
    if let Some(dependencies) = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
    {
        inspect_root_dependency_table("workspace dependency", dependencies, &mut violations);
    }
    if let Some(patches) = workspace.get("patch").and_then(toml::Value::as_table) {
        for (source, dependencies) in patches {
            if let Some(dependencies) = dependencies.as_table() {
                inspect_root_dependency_table(
                    &format!("patch source `{source}`"),
                    dependencies,
                    &mut violations,
                );
            }
        }
    }
    if let Some(replacements) = workspace.get("replace").and_then(toml::Value::as_table) {
        inspect_root_dependency_table("replace", replacements, &mut violations);
    }
    violations
}

fn inspect_root_dependency_table(
    surface: &str,
    dependencies: &toml::map::Map<String, toml::Value>,
    violations: &mut Vec<String>,
) {
    for (name, dependency) in dependencies {
        let Some(dependency_path) = dependency.get("path").and_then(toml::Value::as_str) else {
            continue;
        };
        let components = match resolved_dependency_path("Cargo.toml", dependency_path) {
            Ok(components) => components,
            Err(reason) => {
                violations.push(format!(
                    "Cargo.toml: {surface} `{name}` has invalid path `{dependency_path}`: {reason}"
                ));
                continue;
            }
        };
        let Some(provider) = draft_dependency_owner(&components) else {
            continue;
        };
        violations.push(format!(
            "Cargo.toml: {surface} `{name}` exposes owner-local draft `{provider}`"
        ));
    }
}

pub(super) fn resolved_dependency_path(
    manifest_path: &str,
    dependency_path: &str,
) -> Result<Vec<String>, &'static str> {
    if dependency_path_is_absolute(dependency_path) {
        return Err("absolute paths are forbidden");
    }
    if dependency_path.contains('\\') {
        return Err("backslash separators are forbidden");
    }
    let directory = manifest_path
        .strip_suffix("/Cargo.toml")
        .or_else(|| (manifest_path == "Cargo.toml").then_some(""))
        .ok_or("declaring manifest is not repository-relative")?;
    let mut components: Vec<String> = directory
        .split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_owned)
        .collect();
    for component in dependency_path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err("path escapes the repository root");
                }
            }
            component => components.push(component.to_owned()),
        }
    }
    Ok(components)
}

fn dependency_path_is_absolute(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with(['/', '\\'])
        || matches!(
            bytes,
            [drive, b':', separator, ..]
                if drive.is_ascii_alphabetic() && matches!(separator, b'/' | b'\\')
        )
}

fn manifest_owner(path: &str) -> Option<String> {
    expected_manifest_identity(path)?;
    let parts = path_parts(path);
    if parts.first() == Some(&"app") {
        Some(format!("app/{}", parts.get(1)?))
    } else {
        Some((*parts.first()?).to_owned())
    }
}

fn draft_dependency_owner(components: &[String]) -> Option<String> {
    match components {
        [app, product, face, draft, leaf, ..]
            if app == "app"
                && matches!(face.as_str(), "ports" | "adapters")
                && draft == "draft"
                && !leaf.is_empty() =>
        {
            Some(format!("app/{product}"))
        }
        [owner, face, draft, leaf, ..]
            if matches!(face.as_str(), "ports" | "adapters")
                && draft == "draft"
                && !leaf.is_empty() =>
        {
            Some(owner.clone())
        }
        _ => None,
    }
}

fn visit_dependency_entries(manifest: &toml::Value, mut visit: impl FnMut(&str, &toml::Value)) {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        visit_dependency_table(manifest.get(section), &mut visit);
    }
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                visit_dependency_table(target.get(section), &mut visit);
            }
        }
    }
}

fn visit_dependency_table(
    dependencies: Option<&toml::Value>,
    visit: &mut impl FnMut(&str, &toml::Value),
) {
    if let Some(dependencies) = dependencies.and_then(toml::Value::as_table) {
        for (name, dependency) in dependencies {
            visit(name, dependency);
        }
    }
}

#[cfg(test)]
#[path = "dependency_tests.rs"]
mod tests;
