//! Pure, I/O-free Cargo.lock transform kernel.
//!
//! Two operations, both deterministic pure functions of their `&str` input (NO filesystem, NO
//! cargo, NO version resolution):
//!
//! * [`rewrite_lockfile`] — rename crate names (the `[[package]].name` and every
//!   `dependencies` reference) per a rename map, preserving version/source/checksum and every
//!   byte it does not rename (format-preserving via `toml_edit`).
//! * [`move_lockfile`] — the move-aware transform: rename, then re-canonicalize (add new edges,
//!   re-sort dependency arrays, register new local members, re-sort packages into Cargo's
//!   canonical order). This is the owned replacement for shelling out to `cargo metadata` after
//!   a capability move: a move renames crates and may register newly-created local members, but
//!   introduces NO new version resolution (renames preserve the version graph; registered
//!   members carry already-resolved dependency lists).
//!
//! # Canonical order
//!
//! Canonical order is Cargo's: packages ordered by `name` (byte `Ord`), ties broken by version
//! then source. We reproduce it with a STABLE sort by name only: entries with distinct names
//! order by name (matching Cargo); entries that share a name (multi-version crates) inherit
//! their existing relative order, which — because the input lockfile was itself Cargo-canonical
//! — is already Cargo's (name, semver, source) order. The same reasoning applies to the
//! dependency arrays. This needs neither a semver dependency nor a version parser, and
//! reproduces Cargo's output byte-for-byte.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` / `panic!()` to assert
// invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use anyhow::{Context, Result};
use std::collections::HashMap;

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
        if let Some(name_item) = pkg.get_mut("name")
            && let Some(name_str) = name_item.as_str()
        {
            let name_owned = name_str.to_owned();
            if let Some(new_name) = rename_map.get(&name_owned) {
                *name_item = toml_edit::value(new_name.as_str());
            }
        }

        // Rename occurrences in the dependencies array
        // Dependency strings have the form: "crate-name VERSION" or "crate-name VERSION (SOURCE)"
        if let Some(deps_item) = pkg.get_mut("dependencies")
            && let Some(deps_array) = deps_item.as_array_mut()
        {
            for dep in deps_array.iter_mut() {
                if let Some(dep_str) = dep.as_str() {
                    let dep_owned = dep_str.to_owned();
                    let new_dep = rename_dep_string(&dep_owned, rename_map);
                    if new_dep != dep_owned {
                        // Preserve the element's decor (the `\n ` prefix that
                        // keeps each dependency on its own line); a bare
                        // `Formatted::new` resets it and collapses the array.
                        let decor = dep.decor().clone();
                        let mut replacement =
                            toml_edit::Value::String(toml_edit::Formatted::new(new_dep));
                        *replacement.decor_mut() = decor;
                        *dep = replacement;
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

// ---------------------------------------------------------------------------
// Move-aware Cargo.lock maintenance (rename + register + canonicalize)
// ---------------------------------------------------------------------------
//
// A capability move renames crates AND may add newly-created local members.
// After renaming, the lockfile is no longer in Cargo's canonical order (a
// renamed `[[package]]` block, and each renamed entry in another package's
// `dependencies` array, must move to its new sorted position). Historically
// this re-sort was done by shelling out to Cargo — re-admitting Cargo into the
// authoring loop for a task that is pure, deterministic text manipulation
// (NO version resolution: renames preserve the version graph, and registered
// members carry already-resolved dependency lists). This owns that loop.

/// A newly-created local workspace member to register in the lockfile. Local
/// path crates have no `source`/`checksum`; `dependencies` are the already
/// resolved lock dependency strings (`"name"` / `"name version"` / with source).
pub struct NewMember {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

/// The graph mutations a capability move introduces beyond renames: new local
/// members (new `[[package]]` blocks) and new dependency EDGES added to the
/// arrays of packages that already exist (e.g. an existing facade crate now
/// consuming a newly-created port). No version resolution: every edge names an
/// already-locked crate.
pub struct GraphAdditions {
    pub new_members: Vec<NewMember>,
    /// package-name -> dependency strings to add to that package's array.
    pub add_dependencies: HashMap<String, Vec<String>>,
}

impl GraphAdditions {
    pub fn empty() -> Self {
        Self {
            new_members: Vec::new(),
            add_dependencies: HashMap::new(),
        }
    }
}

/// Full move-aware transform: rename (names + dependency references) then
/// canonicalize (add new edges, re-sort dependency arrays, register new
/// members, re-sort packages). Reuses the tested [`rewrite_lockfile`] for the
/// rename step, which is byte-lossless for everything it does not rename.
pub fn move_lockfile(
    content: &str,
    rename_map: &HashMap<String, String>,
    additions: &GraphAdditions,
) -> Result<String> {
    let renamed = rewrite_lockfile(content, rename_map)?;
    canonicalize(&renamed, additions)
}

/// Add new edges, re-sort dependency arrays, append new members, and re-sort
/// packages into Cargo's canonical order. Operates on the raw text (splitting
/// on the blank-line record separator Cargo emits between the header and every
/// `[[package]]`), so untouched blocks are moved verbatim.
fn canonicalize(content: &str, additions: &GraphAdditions) -> Result<String> {
    let trimmed = content.trim_end();
    let mut records = trimmed.split("\n\n");
    let header = records.next().context("lockfile is empty")?.to_owned();

    let mut keyed: Vec<(String, String)> = Vec::new();
    // add_dependencies target name -> number of [[package]] blocks it matched.
    let mut target_block_count: HashMap<String, usize> = HashMap::new();
    // existing (name, version) pairs, to reject a new member colliding with a real package.
    let mut existing_nv: std::collections::HashSet<(String, Option<String>)> =
        std::collections::HashSet::new();
    for (i, record) in records.enumerate() {
        if !record.starts_with("[[package]]") {
            anyhow::bail!(
                "lockfile record {} is not a [[package]] block (unexpected blank line inside a \
                 block, or non-canonical formatting): starts with {:?}",
                i,
                head(record, 48)
            );
        }
        let name = block_name(record)?;
        existing_nv.insert((name.clone(), block_version(record)));
        let extra = additions.add_dependencies.get(&name);
        if extra.is_some() {
            *target_block_count.entry(name.clone()).or_insert(0) += 1;
        }
        keyed.push((name, process_block(record, extra.map(Vec::as_slice))?));
    }

    // An add_dependencies target must resolve to EXACTLY ONE block: 0 = not a package;
    // >1 = an ambiguous multi-version name the tool must not guess an edge into. Fail-closed.
    for target in additions.add_dependencies.keys() {
        match target_block_count.get(target).copied().unwrap_or(0) {
            0 => {
                anyhow::bail!("add_dependencies target {target:?} is not a package in the lockfile")
            }
            1 => {}
            n => anyhow::bail!(
                "add_dependencies target {target:?} is ambiguous — matches {n} version blocks; \
                 refusing to inject an edge into multiple, fail-closed"
            ),
        }
    }

    // A new member must not collide with an existing package or another new member.
    let mut new_nv: std::collections::HashSet<(String, Option<String>)> =
        std::collections::HashSet::new();
    for member in &additions.new_members {
        let nv = (member.name.clone(), Some(member.version.clone()));
        if existing_nv.contains(&nv) {
            anyhow::bail!(
                "new member {:?} v{:?} already exists as a package in the lockfile, fail-closed",
                member.name,
                member.version
            );
        }
        if !new_nv.insert(nv) {
            anyhow::bail!(
                "new member {:?} v{:?} is declared more than once, fail-closed",
                member.name,
                member.version
            );
        }
        keyed.push((member.name.clone(), render_new_member(member)));
    }

    // Stable sort by package name (byte Ord). Stability preserves the input
    // (Cargo-canonical) order among entries that share a name.
    keyed.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let body: Vec<String> = keyed.into_iter().map(|(_, block)| block).collect();
    Ok(format!("{}\n\n{}\n", header, body.join("\n\n")))
}

/// The first `n` characters of `s` (char-boundary-safe, for error-message context).
fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Extract the `name` field of a `[[package]]` block.
fn block_name(block: &str) -> Result<String> {
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("name = \"")
            && let Some(name) = rest.strip_suffix('"')
        {
            return Ok(name.to_owned());
        }
    }
    anyhow::bail!("package block missing a name field: {:?}", head(block, 60))
}

/// Extract the `version` field of a `[[package]]` block, if present.
fn block_version(block: &str) -> Option<String> {
    for line in block.lines() {
        if let Some(rest) = line.strip_prefix("version = \"")
            && let Some(v) = rest.strip_suffix('"')
        {
            return Some(v.to_owned());
        }
    }
    None
}

/// Sort key for a dependency: the crate-name token (first whitespace-delimited
/// token of the dependency string), which is what Cargo orders by.
fn dep_name(dep_string: &str) -> &str {
    dep_string.split(' ').next().unwrap_or(dep_string)
}

/// Sort key for a raw dependency LINE (` "crate-name version",`).
fn dep_line_key(line: &str) -> &str {
    let inner = line.trim().trim_start_matches('"');
    let inner = inner.split('"').next().unwrap_or(inner);
    dep_name(inner)
}

/// Inject any `extra` dependency edges into a package block, then stably
/// re-sort its `dependencies` array by crate name. If the block has no
/// dependencies array and `extra` is non-empty, the array is created (Cargo
/// emits `dependencies` last, after `source`/`checksum`). A block with neither
/// an array nor extras is returned unchanged.
fn process_block(block: &str, extra: Option<&[String]>) -> Result<String> {
    let extra = extra.unwrap_or(&[]);
    let lines: Vec<&str> = block.lines().collect();

    let open = lines.iter().position(|l| *l == "dependencies = [");

    let Some(open) = open else {
        if extra.is_empty() {
            return Ok(block.to_owned());
        }
        // Create the array at the end of the block (Cargo's field order).
        let mut dep_lines: Vec<String> = extra.iter().map(|d| format!(" \"{d}\",")).collect();
        dep_lines.sort_by(|a, b| dep_line_key(a).as_bytes().cmp(dep_line_key(b).as_bytes()));
        dep_lines.dedup();
        let mut out: Vec<String> = lines.iter().map(|s| (*s).to_owned()).collect();
        out.push("dependencies = [".to_owned());
        out.extend(dep_lines);
        out.push("]".to_owned());
        return Ok(out.join("\n"));
    };

    let rel_close = lines[open + 1..]
        .iter()
        .position(|l| *l == "]")
        .context("malformed dependencies array: no closing ]")?;
    let close = open + 1 + rel_close;

    let mut dep_lines: Vec<String> = lines[open + 1..close]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    dep_lines.extend(extra.iter().map(|d| format!(" \"{d}\",")));
    dep_lines.sort_by(|a, b| dep_line_key(a).as_bytes().cmp(dep_line_key(b).as_bytes()));
    // A dependency array is a set: an injected edge that already exists must not duplicate
    // (Cargo never emits a repeated entry). Sort places equal lines adjacent for dedup.
    dep_lines.dedup();

    let mut out: Vec<String> = Vec::with_capacity(lines.len() + extra.len());
    out.extend(lines[..=open].iter().map(|s| (*s).to_owned()));
    out.extend(dep_lines);
    out.extend(lines[close..].iter().map(|s| (*s).to_owned()));
    Ok(out.join("\n"))
}

/// Render a new local member as a canonical `[[package]]` block (dependencies
/// sorted; no `source`/`checksum`, matching how Cargo emits local path crates).
fn render_new_member(member: &NewMember) -> String {
    let mut block = format!(
        "[[package]]\nname = \"{}\"\nversion = \"{}\"",
        member.name, member.version
    );
    if !member.dependencies.is_empty() {
        let mut deps = member.dependencies.clone();
        deps.sort_by(|a, b| dep_name(a).as_bytes().cmp(dep_name(b).as_bytes()));
        block.push_str("\ndependencies = [");
        for dep in &deps {
            block.push_str(&format!("\n \"{dep}\","));
        }
        block.push_str("\n]");
    }
    block
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
name = "platform-tenant-kernel"
version = "0.1.0"
"#;
        let m = map(&[("platform-tenant-kernel", "shared-tenant-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("shared-tenant-domain"),
            "expected new name in output: {out}"
        );
        assert!(
            !out.contains("platform-tenant-kernel"),
            "old name should be gone: {out}"
        );
    }

    /// Row 2: dependent rename (name appearing in another package's dependencies)
    #[test]
    fn test_dependent_rename() {
        let content = r#"
[[package]]
name = "cloud-region-kernel"
version = "0.1.0"
dependencies = [
 "platform-cell-kernel 0.1.0",
 "platform-data-boundary-kernel 0.1.0",
]
"#;
        let m = map(&[
            ("platform-cell-kernel", "shared-cell-domain"),
            (
                "platform-data-boundary-kernel",
                "shared-data-boundary-kernel",
            ),
        ]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("shared-cell-domain 0.1.0"),
            "cell dep renamed: {out}"
        );
        assert!(
            out.contains("shared-data-boundary-kernel 0.1.0"),
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
        let m = map(&[("platform-tenant-kernel", "shared-tenant-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("\"serde\"") || out.contains("name = \"serde\""),
            "serde unchanged: {out}"
        );
    }

    /// Row 4: quoted form works (toml_edit always emits quoted strings)
    #[test]
    fn test_quoted_form() {
        let content =
            "[[package]]\nname = \"intelligence-evidence-kernel\"\nversion = \"0.1.0\"\n";
        let m = map(&[(
            "intelligence-evidence-kernel",
            "intelligence-evidence-domain",
        )]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("intelligence-evidence-domain"),
            "quoted rename: {out}"
        );
    }

    /// Row 5: unquoted edge — toml_edit parses all TOML strings as quoted; same as row 4
    #[test]
    fn test_toml_strings_are_always_quoted() {
        // TOML requires string values to be quoted; toml_edit handles this transparently
        let content = "[[package]]\nname = \"cloud-compute-kernel\"\nversion = \"0.1.0\"\n";
        let m = map(&[("cloud-compute-kernel", "cloud-compute-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("cloud-compute-domain"),
            "unquoted edge via toml_edit: {out}"
        );
    }

    /// Row 6: version disambiguator — same crate name, two versions, both renamed
    #[test]
    fn test_version_disambiguator() {
        let content = r#"
[[package]]
name = "platform-secrets-kernel"
version = "0.1.0"

[[package]]
name = "platform-secrets-kernel"
version = "0.2.0"
"#;
        let m = map(&[("platform-secrets-kernel", "shared-secrets-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        let count = out.matches("shared-secrets-domain").count();
        assert_eq!(count, 2, "both versions renamed: {out}");
    }

    /// Row 7: version+source disambiguator — name+source combo, both renamed
    #[test]
    fn test_version_source_disambiguator() {
        let content = r#"
[[package]]
name = "platform-eventing-kernel"
version = "0.1.0"
source = "path+file:///workspace/crates/platform-eventing-kernel"

[[package]]
name = "platform-eventing-kernel"
version = "0.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
        let m = map(&[("platform-eventing-kernel", "shared-eventing-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        let count = out.matches("shared-eventing-domain").count();
        assert_eq!(count, 2, "both source variants renamed: {out}");
    }

    /// Row 8: missing rename-map entry → pass through unchanged
    #[test]
    fn test_missing_rename_map_entry_passes_through() {
        let content = "[[package]]\nname = \"unknown-crate\"\nversion = \"0.1.0\"\n";
        // rename_map has no entry for unknown-crate
        let m = map(&[("platform-tenant-kernel", "shared-tenant-domain")]);
        let out = rewrite_lockfile(content, &m).unwrap();
        assert!(
            out.contains("unknown-crate"),
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

    /// Full move transform exercises all four operations against an exact
    /// expected output: (1) rename a crate, (2) re-sort a dependency array whose
    /// order changed because of the rename, (3) register a new local member,
    /// (4) re-sort packages into canonical name order — while (5) a stable sort
    /// preserves the relative order of two same-named (multi-version) entries
    /// that were already in Cargo's semver order.
    #[test]
    fn test_move_lockfile_full_transform_byte_exact() {
        let input = "\
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = \"aggregator\"
version = \"0.1.0\"
dependencies = [
 \"mmm\",
 \"cloud-ci-zed-app\",
]

[[package]]
name = \"dup\"
version = \"0.2.0\"

[[package]]
name = \"dup\"
version = \"0.10.0\"

[[package]]
name = \"mmm\"
version = \"0.1.0\"

[[package]]
name = \"cloud-ci-zed-app\"
version = \"0.1.0\"

[[package]]
name = \"serde\"
version = \"1.0.0\"
source = \"registry+https://github.com/rust-lang/crates.io-index\"
checksum = \"abc123\"
";

        let rename = map(&[("cloud-ci-zed-app", "ci-alpha-gate")]);
        let mut add_dependencies = HashMap::new();
        // Existing `aggregator` gains an edge to the new member `ci-new-lib`.
        add_dependencies.insert("aggregator".to_owned(), vec!["ci-new-lib".to_owned()]);
        let additions = GraphAdditions {
            new_members: vec![NewMember {
                name: "ci-new-lib".to_owned(),
                version: "0.1.0".to_owned(),
                dependencies: vec!["serde".to_owned()],
            }],
            add_dependencies,
        };

        let expected = "\
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = \"aggregator\"
version = \"0.1.0\"
dependencies = [
 \"ci-alpha-gate\",
 \"ci-new-lib\",
 \"mmm\",
]

[[package]]
name = \"ci-alpha-gate\"
version = \"0.1.0\"

[[package]]
name = \"ci-new-lib\"
version = \"0.1.0\"
dependencies = [
 \"serde\",
]

[[package]]
name = \"dup\"
version = \"0.2.0\"

[[package]]
name = \"dup\"
version = \"0.10.0\"

[[package]]
name = \"mmm\"
version = \"0.1.0\"

[[package]]
name = \"serde\"
version = \"1.0.0\"
source = \"registry+https://github.com/rust-lang/crates.io-index\"
checksum = \"abc123\"
";

        let out = move_lockfile(input, &rename, &additions).unwrap();
        assert_eq!(out, expected, "move transform output must be byte-exact");
    }

    /// An ambiguous `add_dependencies` target (name with 2+ version blocks) fails closed
    /// rather than injecting the edge into every version.
    #[test]
    fn test_add_dependencies_ambiguous_target_fails_closed() {
        let input = "# h\nversion = 4\n\n[[package]]\nname = \"dup\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"dup\"\nversion = \"0.2.0\"\n";
        let mut add = HashMap::new();
        add.insert("dup".to_owned(), vec!["x".to_owned()]);
        let additions = GraphAdditions {
            new_members: vec![],
            add_dependencies: add,
        };
        let err = move_lockfile(input, &HashMap::new(), &additions).unwrap_err();
        assert!(err.to_string().contains("ambiguous"), "unexpected: {err}");
    }

    /// A new member colliding with an existing package (same name+version) fails closed.
    #[test]
    fn test_new_member_collision_fails_closed() {
        let input = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\n";
        let additions = GraphAdditions {
            new_members: vec![NewMember {
                name: "a".to_owned(),
                version: "0.1.0".to_owned(),
                dependencies: vec![],
            }],
            add_dependencies: HashMap::new(),
        };
        let err = move_lockfile(input, &HashMap::new(), &additions).unwrap_err();
        assert!(
            err.to_string().contains("already exists"),
            "unexpected: {err}"
        );
    }

    /// `add_dependencies` targeting a package that does not exist fails closed.
    #[test]
    fn test_add_dependencies_unknown_package_fails_closed() {
        let input = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\n";
        let mut add = HashMap::new();
        add.insert("does-not-exist".to_owned(), vec!["x".to_owned()]);
        let additions = GraphAdditions {
            new_members: vec![],
            add_dependencies: add,
        };
        let err = move_lockfile(input, &HashMap::new(), &additions).unwrap_err();
        assert!(
            err.to_string().contains("not a package in the lockfile"),
            "unexpected error: {err}"
        );
    }

    /// An injected edge that already exists in the target array is not duplicated.
    #[test]
    fn test_add_dependencies_dedups_existing_edge() {
        let input = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\ndependencies = [\n \"serde\",\n]\n";
        let mut add = HashMap::new();
        add.insert("a".to_owned(), vec!["serde".to_owned(), "beta".to_owned()]);
        let additions = GraphAdditions {
            new_members: vec![],
            add_dependencies: add,
        };
        let out = move_lockfile(input, &HashMap::new(), &additions).unwrap();
        // "serde" appears once, "beta" added, sorted.
        let expected = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\ndependencies = [\n \"beta\",\n \"serde\",\n]\n";
        assert_eq!(out, expected);
    }

    /// `add_dependencies` creates the array when the target has none.
    #[test]
    fn test_add_dependencies_creates_array() {
        let input = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\n";
        let mut add = HashMap::new();
        add.insert("a".to_owned(), vec!["zeta".to_owned(), "beta".to_owned()]);
        let additions = GraphAdditions {
            new_members: vec![],
            add_dependencies: add,
        };
        let out = move_lockfile(input, &HashMap::new(), &additions).unwrap();
        let expected = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\ndependencies = [\n \"beta\",\n \"zeta\",\n]\n";
        assert_eq!(out, expected);
    }

    /// A blank line inside a package block (non-canonical input) must fail
    /// closed rather than silently corrupt block boundaries.
    #[test]
    fn test_canonicalize_rejects_non_canonical_blank_lines() {
        let bad = "\
# header
version = 4

[[package]]
name = \"a\"

version = \"0.1.0\"
";
        let err = canonicalize(bad, &GraphAdditions::empty()).unwrap_err();
        assert!(
            err.to_string().contains("not a [[package]] block"),
            "unexpected error: {err}"
        );
    }

    /// An empty rename map with a new member still canonicalizes + registers.
    #[test]
    fn test_move_lockfile_register_only() {
        let input = "# h\nversion = 4\n\n[[package]]\nname = \"b\"\nversion = \"0.1.0\"\n";
        let additions = GraphAdditions {
            new_members: vec![NewMember {
                name: "a".to_owned(),
                version: "0.1.0".to_owned(),
                dependencies: vec![],
            }],
            add_dependencies: HashMap::new(),
        };
        let out = move_lockfile(input, &HashMap::new(), &additions).unwrap();
        // "a" sorts before "b".
        let expected = "# h\nversion = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"b\"\nversion = \"0.1.0\"\n";
        assert_eq!(out, expected);
    }
}
