use anyhow::{Context, Result};
use std::path::Path;
use toml_edit::DocumentMut;

/// Required keys in [package.metadata.oya] per §3.0 schema (v4).
const REQUIRED_KEYS: &[&str] = &["bounded_context", "kind", "layer", "purpose"];

/// Layer enum per §2.2 (graphql de-blessed, ADR-0565; pre-existing app/usecase/api drift tracked separately).
const LAYER_VALUES: &[&str] = &[
    "kernel",
    "domain",
    "application",
    "app",
    "adapter",
    "infrastructure",
    "cli",
    "rest",
    "grpc",
    "worker",
    "sdk",
];

pub fn run_metadata_augment(check: bool, _apply: bool, shard: Option<&str>) -> Result<()> {
    if let Some(s) = shard {
        println!("metadata-augment: shard = {s}");
    }

    let repo_root = std::env::current_dir().context("resolving current repo root")?;
    let members = read_workspace_member_paths(&repo_root)?;

    let mut missing: Vec<String> = Vec::new();
    let mut invalid_layer: Vec<String> = Vec::new();

    for path in &members {
        let manifest_path = format!("{path}/Cargo.toml");
        let manifest = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("WARN: cannot read {manifest_path}: {e}");
                continue;
            }
        };
        let manifest_doc: DocumentMut = match manifest.parse() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("WARN: cannot parse {manifest_path}: {e}");
                continue;
            }
        };

        let Some(meta) = manifest_doc
            .get("package")
            .and_then(|item| item.get("metadata"))
            .and_then(|item| item.get("oya"))
        else {
            missing.push(manifest_path.clone());
            continue;
        };

        for key in REQUIRED_KEYS {
            if meta.get(key).is_none() {
                missing.push(format!("{manifest_path}: missing key {key}"));
            }
        }

        if let Some(layer) = meta.get("layer").and_then(|item| item.as_str())
            && !LAYER_VALUES.contains(&layer)
        {
            invalid_layer.push(format!(
                "{manifest_path}: invalid layer \"{layer}\" (must be one of: {})",
                LAYER_VALUES.join(", ")
            ));
        }
    }

    let total = members.len();
    if missing.is_empty() && invalid_layer.is_empty() {
        println!("metadata-augment: OK ({total} members, all have valid [package.metadata.oya])");
        if check {
            println!("(--check mode: no writes performed)");
        }
        return Ok(());
    }

    for m in &missing {
        eprintln!("MISSING: {m}");
    }
    for i in &invalid_layer {
        eprintln!("INVALID: {i}");
    }

    if check {
        println!(
            "(--check mode: {} issue(s) found, no writes performed)",
            missing.len() + invalid_layer.len()
        );
    }

    if !missing.is_empty() || !invalid_layer.is_empty() {
        anyhow::bail!(
            "metadata-augment: {} missing + {} invalid-layer issue(s)",
            missing.len(),
            invalid_layer.len()
        )
    }
    Ok(())
}

fn read_workspace_member_paths(repo_root: &Path) -> Result<Vec<String>> {
    workspace_members_kernel::resolve_member_dirs(repo_root)
        .map_err(anyhow::Error::from)
        .context("resolving glob-aware workspace members")
}

#[cfg(test)]
mod tests {
    use super::read_workspace_member_paths;
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "oya-metadata-augment-members-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("scratch root created");
        root
    }

    fn write_member(root: &Path, relative: &str) {
        let dir = root.join(relative);
        fs::create_dir_all(&dir).expect("member dir created");
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n\n[package.metadata.oya]\nbounded_context = \"test\"\nkind = \"shared\"\nlayer = \"kernel\"\npurpose = \"test fixture\"\n",
                relative.replace('/', "-")
            ),
        )
        .expect("member manifest written");
    }

    #[test]
    fn workspace_members_are_expanded_before_metadata_checks() {
        let root = scratch_root();
        write_member(&root, "tools/oya-one");
        write_member(&root, "tools/oya-two");
        write_member(&root, "cloud/cloud-kernel/crates/oya-excluded");
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"tools/oya-*\", \"cloud/*/crates/oya-*\"]\nexclude = [\"cloud/cloud-kernel\"]\n",
        )
        .expect("workspace manifest written");

        let members = read_workspace_member_paths(&root).expect("members resolved");
        assert_eq!(members, vec!["tools/oya-one", "tools/oya-two"]);

        fs::remove_dir_all(root).expect("scratch root removed");
    }
}
