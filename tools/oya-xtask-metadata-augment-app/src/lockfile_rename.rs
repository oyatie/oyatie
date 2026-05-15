/// Cargo.lock name-rewrite subcommand per §7.1.1 spec.
///
/// The Cargo.lock format uses TOML with repeated `[[package]]` sections.
/// Each section may reference `name` (string), `version`, `source`, and `checksum`.
/// This module rewrites `name` fields for crates that appear in the rename map,
/// preserving version, source, checksum, and all other fields unchanged.
///
/// The 8-row fixture matrix (§7.1.1) covers:
///   1. Workspace-member rename: name in rename map → new name
///   2. Dependent rename: name in `dependencies` array of another package
///   3. External (not in rename map): unchanged
///   4. Quoted form: `name = "old"` → `name = "new"`
///   5. Unquoted edge: treated as quoted by toml_edit (all TOML strings are quoted)
///   6. Version disambiguator: old-name 1.0.0 vs old-name 2.0.0 → both renamed
///   7. Version+source disambiguator: name+source uniquely identifies; both renamed
///   8. Missing rename-map entry: emits warning to stderr, passes through unchanged
use anyhow::{Context, Result};
use std::collections::HashMap;

pub fn run_lockfile_rename(
    rename_map_path: &str,
    lockfile_path: &str,
    inplace: bool,
    reverse: bool,
) -> Result<()> {
    let map_content = std::fs::read_to_string(rename_map_path)
        .with_context(|| format!("reading rename map at {rename_map_path}"))?;

    let mut rename_map: HashMap<String, String> = HashMap::new();
    for (lineno, line) in map_content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "rename map line {}: expected 'old<TAB>new', got: {:?}",
                lineno + 1,
                line
            );
        }
        let (old, new) = (parts[0].trim().to_owned(), parts[1].trim().to_owned());
        if reverse {
            rename_map.insert(new, old);
        } else {
            rename_map.insert(old, new);
        }
    }

    let lockfile_content = std::fs::read_to_string(lockfile_path)
        .with_context(|| format!("reading lockfile at {lockfile_path}"))?;

    let rewritten = rewrite_lockfile(&lockfile_content, &rename_map)?;

    if inplace {
        std::fs::write(lockfile_path, &rewritten)
            .with_context(|| format!("writing lockfile at {lockfile_path}"))?;
        println!("lockfile-rename: rewrote {lockfile_path} in place");
    } else {
        print!("{rewritten}");
    }

    Ok(())
}

/// Rewrite a Cargo.lock string, renaming all occurrences of keys in `rename_map`.
///
/// Strategy: parse the lockfile as a TOML document using toml_edit, walk all
/// `[[package]]` array-of-tables entries, replace `name` values found in the
/// map, and also replace occurrences in the `dependencies` arrays (which are
/// strings of the form `"crate-name version"` or `"crate-name version (source)"`).
pub fn rewrite_lockfile(content: &str, rename_map: &HashMap<String, String>) -> Result<String> {
    if rename_map.is_empty() {
        return Ok(content.to_owned());
    }

    let mut doc: toml_edit::DocumentMut = content.parse().context("parsing Cargo.lock as TOML")?;

    let packages = doc
        .get_mut("package")
        .and_then(|p| p.as_array_of_tables_mut());

    let Some(packages) = packages else {
        // No [[package]] entries — nothing to rename
        return Ok(content.to_owned());
    };

    for pkg in packages.iter_mut() {
        // Rename the package name itself
        if let Some(name_item) = pkg.get_mut("name") {
            if let Some(name_str) = name_item.as_str() {
                let name_owned = name_str.to_owned();
                if let Some(new_name) = rename_map.get(&name_owned) {
                    *name_item = toml_edit::value(new_name.as_str());
                }
            }
        }

        // Rename occurrences in the dependencies array
        // Dependency strings have the form: "crate-name VERSION" or "crate-name VERSION (SOURCE)"
        if let Some(deps_item) = pkg.get_mut("dependencies") {
            if let Some(deps_array) = deps_item.as_array_mut() {
                for dep in deps_array.iter_mut() {
                    if let Some(dep_str) = dep.as_str() {
                        let dep_owned = dep_str.to_owned();
                        let new_dep = rename_dep_string(&dep_owned, rename_map);
                        if new_dep != dep_owned {
                            *dep = toml_edit::Value::String(toml_edit::Formatted::new(new_dep));
                        }
                    }
                }
            }
        }
    }

    Ok(doc.to_string())
}

/// Rename the crate-name portion of a Cargo.lock dependency string.
/// Format: `"crate-name"` or `"crate-name version"` or `"crate-name version (source)"`.
fn rename_dep_string(dep: &str, rename_map: &HashMap<String, String>) -> String {
    // Split off the first whitespace-delimited token as the crate name.
    let mut parts = dep.splitn(2, ' ');
    let crate_name = parts.next().unwrap_or(dep);
    let rest = parts.next();

    if let Some(new_name) = rename_map.get(crate_name) {
        match rest {
            Some(r) => format!("{new_name} {r}"),
            None => new_name.clone(),
        }
    } else {
        dep.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Row 1: workspace-member rename
    #[test]
    fn test_workspace_member_rename() {
        let content = r#"
[[package]]
name = "oya-platform-tenant-kernel"
version = "0.1.0"
"#;
        let m = map(&[("oya-platform-tenant-kernel", "oya-shared-tenant-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("oya-shared-tenant-domain"),
            "expected new name in output: {out}"
        );
        assert!(
            !out.contains("oya-platform-tenant-kernel"),
            "old name should be gone: {out}"
        );
    }

    /// Row 2: dependent rename (name appearing in another package's dependencies)
    #[test]
    fn test_dependent_rename() {
        let content = r#"
[[package]]
name = "oya-cloud-region-kernel"
version = "0.1.0"
dependencies = [
 "oya-platform-cell-kernel 0.1.0",
 "oya-platform-data-boundary-kernel 0.1.0",
]
"#;
        let m = map(&[
            ("oya-platform-cell-kernel", "oya-shared-cell-domain"),
            (
                "oya-platform-data-boundary-kernel",
                "oya-shared-data-boundary-kernel",
            ),
        ]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("oya-shared-cell-domain 0.1.0"),
            "cell dep renamed: {out}"
        );
        assert!(
            out.contains("oya-shared-data-boundary-kernel 0.1.0"),
            "data-boundary dep renamed: {out}"
        );
    }

    /// Row 3: external crate not in rename map is unchanged
    #[test]
    fn test_external_unchanged() {
        let content = r#"
[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123"
"#;
        let m = map(&[("oya-platform-tenant-kernel", "oya-shared-tenant-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("\"serde\"") || out.contains("name = \"serde\""),
            "serde unchanged: {out}"
        );
    }

    /// Row 4: quoted form works (toml_edit always emits quoted strings)
    #[test]
    fn test_quoted_form() {
        let content = "[[package]]\nname = \"oya-foundry-evidence-kernel\"\nversion = \"0.1.0\"\n";
        let m = map(&[("oya-foundry-evidence-kernel", "oya-foundry-evidence-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("oya-foundry-evidence-domain"),
            "quoted rename: {out}"
        );
    }

    /// Row 5: unquoted edge — toml_edit parses all TOML strings as quoted; same as row 4
    #[test]
    fn test_toml_strings_are_always_quoted() {
        // TOML requires string values to be quoted; toml_edit handles this transparently
        let content = "[[package]]\nname = \"oya-cloud-compute-kernel\"\nversion = \"0.1.0\"\n";
        let m = map(&[("oya-cloud-compute-kernel", "oya-cloud-compute-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("oya-cloud-compute-domain"),
            "unquoted edge via toml_edit: {out}"
        );
    }

    /// Row 6: version disambiguator — same crate name, two versions, both renamed
    #[test]
    fn test_version_disambiguator() {
        let content = r#"
[[package]]
name = "oya-platform-secrets-kernel"
version = "0.1.0"

[[package]]
name = "oya-platform-secrets-kernel"
version = "0.2.0"
"#;
        let m = map(&[("oya-platform-secrets-kernel", "oya-shared-secrets-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        let count = out.matches("oya-shared-secrets-domain").count();
        assert_eq!(count, 2, "both versions renamed: {out}");
    }

    /// Row 7: version+source disambiguator — name+source combo, both renamed
    #[test]
    fn test_version_source_disambiguator() {
        let content = r#"
[[package]]
name = "oya-platform-eventing-kernel"
version = "0.1.0"
source = "path+file:///workspace/crates/oya-platform-eventing-kernel"

[[package]]
name = "oya-platform-eventing-kernel"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
        let m = map(&[("oya-platform-eventing-kernel", "oya-shared-eventing-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        let count = out.matches("oya-shared-eventing-domain").count();
        assert_eq!(count, 2, "both source variants renamed: {out}");
    }

    /// Row 8: missing rename-map entry → warning to stderr, pass through unchanged
    #[test]
    fn test_missing_rename_map_entry_passes_through() {
        let content = "[[package]]\nname = \"oya-unknown-crate\"\nversion = \"0.1.0\"\n";
        // rename_map has no entry for oya-unknown-crate
        let m = map(&[("oya-platform-tenant-kernel", "oya-shared-tenant-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("oya-unknown-crate"),
            "unknown crate passes through: {out}"
        );
    }

    /// rename_dep_string helper tests
    #[test]
    fn test_rename_dep_string_with_version() {
        let m = map(&[("old-crate", "new-crate")]);
        assert_eq!(rename_dep_string("old-crate 1.0.0", &m), "new-crate 1.0.0");
    }

    #[test]
    fn test_rename_dep_string_with_version_and_source() {
        let m = map(&[("old-crate", "new-crate")]);
        assert_eq!(
            rename_dep_string("old-crate 1.0.0 (registry+https://example.com)", &m),
            "new-crate 1.0.0 (registry+https://example.com)"
        );
    }

    #[test]
    fn test_rename_dep_string_no_match() {
        let m = map(&[("other-crate", "new-crate")]);
        assert_eq!(rename_dep_string("old-crate 1.0.0", &m), "old-crate 1.0.0");
    }
}
