use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use pipeline_admission::{
    REINDEER_QUALIFICATION_EXACT_PATHS, REINDEER_QUALIFICATION_PATH_PREFIXES,
};
use toml::Value;

#[test]
fn reindeer_package_prefixes_are_the_exact_local_dependency_closure() {
    let root = super::repo_root();
    let actual = local_package_closure(
        &root,
        [
            "build/dependency-declarations/adapters/generation-reindeer/",
            "pipeline/facade/change-gates-app/",
        ],
    );
    let expected = REINDEER_QUALIFICATION_PATH_PREFIXES
        .iter()
        .map(|path| (*path).to_owned())
        .collect();

    assert_eq!(actual, expected);
}

#[test]
fn reindeer_exact_inputs_are_complete_and_precise() {
    assert_eq!(
        REINDEER_QUALIFICATION_EXACT_PATHS,
        [
            ".cargo/config.toml",
            ".config/nextest.toml",
            ".github/workflows/presubmit.yml",
            "Cargo.lock",
            "Cargo.toml",
            "reindeer.toml",
            "rust-toolchain.toml",
        ]
    );
}

fn local_package_closure<const N: usize>(root: &Path, seeds: [&str; N]) -> BTreeSet<String> {
    let workspace_manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .expect("workspace package manifest")
        .parse::<Value>()
        .expect("valid workspace package manifest");
    let workspace_dependencies = workspace_manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .expect("workspace dependency table");
    let mut pending: VecDeque<PathBuf> = seeds.into_iter().map(|path| root.join(path)).collect();
    let mut closure = BTreeSet::new();
    while let Some(package) = pending.pop_front() {
        let package = package.canonicalize().expect("local package directory");
        let relative = repo_relative_directory(root, &package);
        if !closure.insert(relative) {
            continue;
        }
        let manifest =
            std::fs::read_to_string(package.join("Cargo.toml")).expect("local package manifest");
        let manifest: Value = manifest.parse().expect("valid local package manifest");
        for dependency in
            local_dependency_directories(root, &package, &manifest, workspace_dependencies)
        {
            let dependency = dependency
                .canonicalize()
                .expect("local dependency directory");
            assert!(
                dependency.starts_with(root),
                "local dependency escaped the repository: {}",
                dependency.display()
            );
            pending.push_back(dependency);
        }
    }
    closure
}

fn local_dependency_directories(
    root: &Path,
    package: &Path,
    manifest: &Value,
    workspace_dependencies: &toml::map::Map<String, Value>,
) -> Vec<PathBuf> {
    let Some(manifest_root) = manifest.as_table() else {
        return Vec::new();
    };
    let mut specifications = Vec::new();
    collect_dependency_tables(manifest_root, &mut specifications);
    if let Some(targets) = manifest_root.get("target").and_then(Value::as_table) {
        for target in targets.values().filter_map(Value::as_table) {
            collect_dependency_tables(target, &mut specifications);
        }
    }
    specifications
        .into_iter()
        .filter_map(|(name, dependency)| {
            let specification = dependency.as_table()?;
            if let Some(path) = specification.get("path").and_then(Value::as_str) {
                return Some(package.join(path));
            }
            if specification.get("workspace").and_then(Value::as_bool) == Some(true) {
                return workspace_dependencies
                    .get(name)
                    .and_then(Value::as_table)
                    .and_then(|dependency| dependency.get("path"))
                    .and_then(Value::as_str)
                    .map(|path| root.join(path));
            }
            None
        })
        .collect()
}

fn collect_dependency_tables<'a>(
    table: &'a toml::map::Map<String, Value>,
    output: &mut Vec<(&'a str, &'a Value)>,
) {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(dependency_table) = table.get(section).and_then(Value::as_table) else {
            continue;
        };
        output.extend(
            dependency_table
                .iter()
                .map(|(name, dependency)| (name.as_str(), dependency)),
        );
    }
}

fn repo_relative_directory(root: &Path, package: &Path) -> String {
    let mut relative = package
        .strip_prefix(root)
        .expect("repository-local package")
        .components()
        .map(|component| component.as_os_str().to_str().expect("UTF-8 package path"))
        .collect::<Vec<_>>()
        .join("/");
    relative.push('/');
    relative
}
