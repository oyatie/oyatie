//! `Cargo.toml` transforms: the `[package].name` / `[lib].name` / `[[bin]].name` rewrite of
//! a moved crate, and the dependency-name + relative `path=` recompute across EVERY
//! workspace manifest (the move-fatal `../../../` class). Format-preserving via `toml_edit`.

use std::collections::BTreeMap;

use toml_edit::{DocumentMut, Item, Table, Value};

use crate::model::{CodemodError, CrateMove, recompute_rel_path_dep, snake};

/// The dependency table keys Cargo recognizes (workspace + target-cfg tables are handled by
/// the recursive walk in [`rewrite_dependencies_in_manifest`]).
const DEP_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Rewrite a MOVED crate's own `Cargo.toml`: `[package].name`, and the `[lib].name` /
/// `[[bin]].name` snake mirrors when they default to the package name. Returns the new
/// manifest text. Idempotent: re-running on an already-renamed manifest is a no-op.
pub fn rewrite_moved_manifest_package(
    manifest_text: &str,
    manifest_rel_path: &str,
    cm: &CrateMove,
) -> Result<String, CodemodError> {
    let mut doc = parse(manifest_text, manifest_rel_path)?;
    if let Some(pkg) = doc.get_mut("package").and_then(Item::as_table_mut)
        && pkg.get("name").and_then(Item::as_str) == Some(cm.old_cargo_name.as_str())
    {
        pkg["name"] = toml_edit::value(cm.new_cargo_name.clone());
    }
    // [lib].name (single table) snake mirror.
    let old_ident = snake(&cm.old_cargo_name);
    let new_ident = snake(&cm.new_cargo_name);
    if let Some(lib) = doc.get_mut("lib").and_then(Item::as_table_mut) {
        rewrite_name_field_if(lib, &old_ident, &new_ident);
    }
    // [[bin]] array-of-tables snake mirror.
    if let Some(bins) = doc.get_mut("bin").and_then(Item::as_array_of_tables_mut) {
        for bin in bins.iter_mut() {
            rewrite_name_field_if(bin, &old_ident, &new_ident);
        }
    }
    Ok(doc.to_string())
}

fn rewrite_name_field_if(table: &mut Table, old_ident: &str, new_ident: &str) {
    if table.get("name").and_then(Item::as_str) == Some(old_ident) {
        table["name"] = toml_edit::value(new_ident.to_string());
    }
}

/// Rewrite, in ANY workspace manifest, every dependency on a moved crate: rename the
/// dependency KEY to its new cargo name, and recompute its relative `path=` against the
/// post-move layout. `manifest_rel_path` is the manifest's repo-relative path (so its dir is
/// the recompute base). `name_to_move` maps an OLD kebab cargo name to its [`CrateMove`].
/// `resolve_target` maps an OLD repo-relative crate DIR to its NEW dir (identity for unmoved
/// crates). `this_manifest_moved_to` is `Some(new_dir)` when THIS manifest's own crate is
/// moving (so the recompute base is its NEW dir), else `None`.
///
/// Returns `(new_text, changed)`; `changed` is false when nothing matched (so the caller can
/// skip the write and keep the tree byte-identical for untouched manifests).
pub fn rewrite_dependencies_in_manifest(
    manifest_text: &str,
    manifest_rel_path: &str,
    this_manifest_old_dir: &str,
    this_manifest_moved_to: Option<&str>,
    name_to_move: &BTreeMap<&str, &CrateMove>,
    resolve_target: &dyn Fn(&str) -> Option<String>,
) -> Result<(String, bool), CodemodError> {
    let mut doc = parse(manifest_text, manifest_rel_path)?;
    let mut changed = false;

    let new_manifest_dir = this_manifest_moved_to
        .map(str::to_string)
        .unwrap_or_else(|| this_manifest_old_dir.to_string());

    // Walk the document for any dependency table at any depth (handles plain dep tables and
    // `[target.'cfg(...)'.dependencies]`). We collect mutable table refs first to satisfy
    // the borrow checker, then mutate.
    rewrite_dep_tables_recursive(
        doc.as_table_mut(),
        manifest_rel_path,
        this_manifest_old_dir,
        &new_manifest_dir,
        name_to_move,
        resolve_target,
        &mut changed,
        false,
    )?;

    Ok((doc.to_string(), changed))
}

#[allow(clippy::too_many_arguments)]
fn rewrite_dep_tables_recursive(
    table: &mut Table,
    manifest_rel_path: &str,
    old_dir: &str,
    new_dir: &str,
    name_to_move: &BTreeMap<&str, &CrateMove>,
    resolve_target: &dyn Fn(&str) -> Option<String>,
    changed: &mut bool,
    inside_target: bool,
) -> Result<(), CodemodError> {
    // First, process direct dependency tables at this level.
    for key in DEP_TABLES {
        if let Some(dep_table) = table.get_mut(key).and_then(Item::as_table_mut) {
            rewrite_one_dep_table(
                dep_table,
                manifest_rel_path,
                old_dir,
                new_dir,
                name_to_move,
                resolve_target,
                changed,
            )?;
        }
    }
    // Recurse into `[workspace.dependencies]` (and its dev/build siblings). These are REAL
    // path deps — in the ADR-0512 kernel carve-out 5 of 7 internal edges are declared here —
    // and they are relative to the WORKSPACE ROOT manifest's own dir, which is this manifest's
    // dir, so the same old_dir/new_dir base applies unchanged. Skipping them left every such
    // edge pointing at the emptied source dir after a move.
    if !inside_target
        && let Some(workspace) = table.get_mut("workspace").and_then(Item::as_table_mut)
    {
        for key in DEP_TABLES {
            if let Some(dep_table) = workspace.get_mut(key).and_then(Item::as_table_mut) {
                rewrite_one_dep_table(
                    dep_table,
                    manifest_rel_path,
                    old_dir,
                    new_dir,
                    name_to_move,
                    resolve_target,
                    changed,
                )?;
            }
        }
    }
    // Recurse into `[target.<cfg>]` sub-tables (which themselves hold dep tables). Avoid
    // recursing into the dep tables we just handled or into `[package]`/`[workspace]` etc.
    if !inside_target && let Some(target) = table.get_mut("target").and_then(Item::as_table_mut) {
        // Collect cfg sub-table keys to iterate without aliasing.
        let cfg_keys: Vec<String> = target
            .iter()
            .filter(|(_, v)| v.is_table())
            .map(|(k, _)| k.to_string())
            .collect();
        for cfg in cfg_keys {
            if let Some(sub) = target.get_mut(&cfg).and_then(Item::as_table_mut) {
                rewrite_dep_tables_recursive(
                    sub,
                    manifest_rel_path,
                    old_dir,
                    new_dir,
                    name_to_move,
                    resolve_target,
                    changed,
                    true,
                )?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn rewrite_one_dep_table(
    dep_table: &mut Table,
    manifest_rel_path: &str,
    old_dir: &str,
    new_dir: &str,
    name_to_move: &BTreeMap<&str, &CrateMove>,
    resolve_target: &dyn Fn(&str) -> Option<String>,
    changed: &mut bool,
) -> Result<(), CodemodError> {
    // Phase 1: recompute every relative path= dep (the move-fatal class). We must touch a
    // path dep whenever EITHER this manifest moved OR the dep target moved.
    let manifest_moved = old_dir != new_dir;
    let dep_keys: Vec<String> = dep_table.iter().map(|(k, _)| k.to_string()).collect();
    for dep_key in &dep_keys {
        let item = match dep_table.get_mut(dep_key) {
            Some(i) => i,
            None => continue,
        };
        // Only inline-table / table deps can carry a `path=`.
        let path_val: Option<String> = item
            .as_table_like()
            .and_then(|t| t.get("path"))
            .and_then(Item::as_str)
            .map(str::to_string)
            .or_else(|| {
                item.as_value()
                    .and_then(Value::as_inline_table)
                    .and_then(|t| t.get("path"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });
        if let Some(rel_dep) = path_val {
            // Does the target move?
            let old_target = crate::model::join_rel(old_dir, &rel_dep);
            let target_moves = old_target
                .as_deref()
                .map(|t| resolve_target(t).is_some())
                .unwrap_or(false);
            if !manifest_moved && !target_moves {
                continue; // neither side moved -> path stays valid, leave byte-identical
            }
            let new_rel = recompute_rel_path_dep(old_dir, new_dir, &rel_dep, resolve_target)
                .ok_or_else(|| CodemodError::AmbiguousPathDep {
                    manifest: manifest_rel_path.to_string(),
                    dep: dep_key.clone(),
                    path: rel_dep.clone(),
                })?;
            if new_rel != rel_dep {
                set_dep_path(item, &new_rel);
                *changed = true;
            }
        }
    }

    // Phase 1b: an ALIASED dep (`hal = { package = "oya-hal", path = ... }`) names its real
    // crate in `package`, and its KEY is the identifier the source binds (`use hal::`). So the
    // rename must land on the `package` VALUE and the key must be left alone — renaming the key
    // would silently rebind every `use` of it. Phase 2 below therefore skips aliased entries.
    for dep_key in &dep_keys {
        let Some(item) = dep_table.get_mut(dep_key) else {
            continue;
        };
        let Some(package) = dep_package_field(item) else {
            continue;
        };
        if let Some(cm) = name_to_move.get(package.as_str())
            && cm.new_cargo_name != package
        {
            set_dep_package(item, &cm.new_cargo_name);
            *changed = true;
        }
    }

    // Phase 2: rename dependency keys whose crate moved (and got a new cargo name). A naive
    // remove+insert APPENDS (toml_edit has no in-place rekey), which would reorder the table
    // and break byte-identity on a forward/inverse round-trip. To preserve DECLARATION ORDER
    // we rebuild the table: walk the existing keys in order, re-inserting each (renamed where
    // it moved) so the relative order is unchanged. Only done when a rename is actually needed.
    //
    // An entry carrying an explicit `package` is EXCLUDED: Phase 1b already renamed its real
    // crate name, and its key is a binding identifier rather than a crate name.
    let is_aliased = |k: &str| dep_table.get(k).and_then(dep_package_field_ref).is_some();
    let needs_rename = dep_keys.iter().any(|k| {
        !is_aliased(k)
            && name_to_move
                .get(k.as_str())
                .is_some_and(|cm| cm.new_cargo_name != *k)
    });
    let aliased: std::collections::BTreeSet<String> =
        dep_keys.iter().filter(|k| is_aliased(k)).cloned().collect();
    if needs_rename {
        // Drain every dep entry (preserving its value/decor) in declaration order.
        let mut drained: Vec<(String, Item)> = Vec::with_capacity(dep_keys.len());
        for k in &dep_keys {
            if let Some(item) = dep_table.remove(k) {
                let new_key = if aliased.contains(k) {
                    k.clone()
                } else {
                    name_to_move
                        .get(k.as_str())
                        .filter(|cm| cm.new_cargo_name != *k)
                        .map(|cm| cm.new_cargo_name.clone())
                        .unwrap_or_else(|| k.clone())
                };
                drained.push((new_key, item));
            }
        }
        // Re-insert in the SAME order with the renamed keys.
        for (key, item) in drained {
            dep_table.insert(&key, item);
        }
        *changed = true;
    }
    Ok(())
}

/// The explicit `package = "..."` of a dependency entry, when present. Its presence means the
/// entry's KEY is a binding alias rather than the crate name.
fn dep_package_field(item: &Item) -> Option<String> {
    dep_package_field_ref(item)
}

fn dep_package_field_ref(item: &Item) -> Option<String> {
    item.as_table_like()
        .and_then(|t| t.get("package"))
        .and_then(Item::as_str)
        .map(str::to_string)
        .or_else(|| {
            item.as_value()
                .and_then(Value::as_inline_table)
                .and_then(|t| t.get("package"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

/// Set the `package` field of a dependency item (inline-table or table), preserving the rest.
fn set_dep_package(item: &mut Item, new_package: &str) {
    if let Some(value) = item.as_value_mut()
        && let Some(inline) = value.as_inline_table_mut()
    {
        inline.insert("package", new_package.into());
        return;
    }
    if let Some(table) = item.as_table_like_mut() {
        table.insert("package", Item::Value(Value::from(new_package.to_string())));
    }
}

/// Set the `path` field of a dependency item (inline-table or table), preserving the rest.
fn set_dep_path(item: &mut Item, new_path: &str) {
    if let Some(value) = item.as_value_mut()
        && let Some(inline) = value.as_inline_table_mut()
    {
        inline.insert("path", new_path.into());
        return;
    }
    if let Some(table) = item.as_table_like_mut() {
        table.insert("path", Item::Value(Value::from(new_path.to_string())));
    }
}

/// Rewrite the root workspace `[workspace].members` / `exclude` arrays so the post-move
/// member set still resolves, ONLY when a moved path is not already covered by an existing
/// glob/literal. The check uses the [`oya_workspace_members_kernel`] resolver semantics. We
/// take the resolved member dirs BEFORE and the moved new-dirs, and append literal entries
/// for any new-dir not covered. Excludes for vanished old-dirs are removed. Returns
/// `(new_text, changed)`.
///
/// `uncovered_new_dirs` are moved new dirs that no existing glob matches (added as literal
/// members). `globs_to_prune` are members entries that match ZERO crates post-move (a move
/// emptied a globbed dir, which makes Cargo error `failed to read <glob>/Cargo.toml`); they
/// are removed so the post-move workspace still resolves. `excludes_to_remove` are exclude
/// entries that pointed at now-moved old paths.
pub fn rewrite_root_workspace_members(
    root_manifest_text: &str,
    uncovered_new_dirs: &[String],
    globs_to_prune: &[String],
    new_excludes_to_add: &[String],
    excludes_to_remove: &[String],
) -> Result<(String, bool), CodemodError> {
    if uncovered_new_dirs.is_empty()
        && globs_to_prune.is_empty()
        && new_excludes_to_add.is_empty()
        && excludes_to_remove.is_empty()
    {
        return Ok((root_manifest_text.to_string(), false));
    }
    let mut doc = parse(root_manifest_text, "Cargo.toml")?;
    let workspace = doc
        .get_mut("workspace")
        .and_then(Item::as_table_mut)
        .ok_or_else(|| CodemodError::Parse {
            path: "Cargo.toml".to_string(),
            message: "missing [workspace] table".to_string(),
        })?;
    let mut changed = false;
    if let Some(members) = workspace.get_mut("members").and_then(Item::as_array_mut) {
        // Prune now-empty globs first (so an added literal is never collaterally pruned).
        if !globs_to_prune.is_empty() {
            let before = members.len();
            members.retain(|v| {
                v.as_str()
                    .map(|s| !globs_to_prune.iter().any(|g| g == s))
                    .unwrap_or(true)
            });
            if members.len() != before {
                changed = true;
            }
        }
        for dir in uncovered_new_dirs {
            if !members.iter().any(|v| v.as_str() == Some(dir.as_str())) {
                members.push(dir.as_str());
                changed = true;
            }
        }
    }
    if !new_excludes_to_add.is_empty() || !excludes_to_remove.is_empty() {
        let excl = workspace
            .entry("exclude")
            .or_insert_with(|| Item::Value(Value::Array(toml_edit::Array::new())));
        if let Some(arr) = excl.as_array_mut() {
            arr.retain(|v| {
                v.as_str()
                    .map(|s| !excludes_to_remove.iter().any(|r| r == s))
                    .unwrap_or(true)
            });
            for e in new_excludes_to_add {
                if !arr.iter().any(|v| v.as_str() == Some(e.as_str())) {
                    arr.push(e.as_str());
                }
            }
            changed = true;
        }
    }
    Ok((doc.to_string(), changed))
}

/// True if this manifest declares a cargo WORKSPACE ROOT. Any `[workspace...]` table counts:
/// a manifest carrying only `[workspace.package]` or `[workspace.dependencies]` is still a
/// workspace root as far as cargo's upward search is concerned. Member crates instead carry
/// `workspace = true` INSIDE `[package]`/`[dependencies]` entries, which never creates a
/// top-level `workspace` table — so this does not mistake a member for a root.
pub fn manifest_declares_workspace(
    manifest_text: &str,
    rel_path: &str,
) -> Result<bool, CodemodError> {
    Ok(parse(manifest_text, rel_path)?.get("workspace").is_some())
}

/// The `exclude` entries of a workspace manifest (empty when it declares none).
pub fn workspace_excludes(
    manifest_text: &str,
    rel_path: &str,
) -> Result<Vec<String>, CodemodError> {
    let doc = parse(manifest_text, rel_path)?;
    Ok(doc
        .get("workspace")
        .and_then(Item::as_table)
        .and_then(|w| w.get("exclude"))
        .and_then(Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

fn parse(text: &str, rel_path: &str) -> Result<DocumentMut, CodemodError> {
    text.parse::<DocumentMut>()
        .map_err(|e| CodemodError::Parse {
            path: rel_path.to_string(),
            message: e.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cm(old_path: &str, new_path: &str, old_name: &str, new_name: &str) -> CrateMove {
        CrateMove {
            old_path: old_path.to_string(),
            new_path: new_path.to_string(),
            old_cargo_name: old_name.to_string(),
            new_cargo_name: new_name.to_string(),
        }
    }

    #[test]
    fn package_name_and_lib_bin_snake_mirror_rewritten() {
        let text = r#"[package]
name = "oya-cloud-iam"
version = "0.1.0"

[lib]
name = "oya_cloud_iam"
path = "src/lib.rs"

[[bin]]
name = "oya_cloud_iam"
path = "src/main.rs"
"#;
        let cm = cm(
            "cloud/cloud-iam/crates/oya-cloud-iam",
            "iam/core/iam",
            "oya-cloud-iam",
            "iam-core",
        );
        let out = rewrite_moved_manifest_package(text, "x/Cargo.toml", &cm).unwrap();
        assert!(out.contains("name = \"iam-core\""));
        assert!(out.contains("name = \"iam_core\""), "lib/bin snake mirror");
        assert!(!out.contains("oya-cloud-iam"));
        assert!(!out.contains("oya_cloud_iam"));
        // version untouched (format-preserving).
        assert!(out.contains("version = \"0.1.0\""));
    }

    #[test]
    fn dep_path_recompute_for_moved_manifest_unmoved_target() {
        // manifest moves; one dep (../oya-domain) target does NOT move.
        let text = r#"[package]
name = "oya-iam-app"

[dependencies]
oya-domain = { path = "../oya-domain" }
serde = { workspace = true }
"#;
        let domain_move = cm(
            "cloud/cloud-iam/crates/oya-iam-app",
            "iam/facade/iam-app",
            "oya-iam-app",
            "iam-app",
        );
        let _ = &domain_move;
        let name_to_move: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        let (out, changed) = rewrite_dependencies_in_manifest(
            text,
            "iam/facade/iam-app/Cargo.toml",
            "cloud/cloud-iam/crates/oya-iam-app",
            Some("iam/facade/iam-app"),
            &name_to_move,
            &|_old| None,
        )
        .unwrap();
        assert!(changed);
        // old target cloud/cloud-iam/crates/oya-domain, new manifest iam/facade/iam-app
        // (3 segments) -> ../../../cloud/cloud-iam/crates/oya-domain
        assert!(
            out.contains("path = \"../../../cloud/cloud-iam/crates/oya-domain\""),
            "recomputed path: {out}"
        );
        assert!(out.contains("serde = { workspace = true }"));
    }

    #[test]
    fn dep_key_renamed_and_path_recomputed_when_target_moves() {
        // manifest cloud/other/crates/oya-other points at cloud/cloud-iam/crates/oya-domain
        // (3 ups to root, then down): ../../../cloud-iam/crates/oya-domain.
        let text = r#"[package]
name = "oya-other"

[dependencies]
oya-domain = { path = "../../../cloud-iam/crates/oya-domain" }
"#;
        let domain = cm(
            "cloud/cloud-iam/crates/oya-domain",
            "iam/core/domain",
            "oya-domain",
            "iam-domain",
        );
        let mut name_to_move: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        name_to_move.insert("oya-domain", &domain);
        let (out, changed) = rewrite_dependencies_in_manifest(
            text,
            "cloud/other/crates/oya-other/Cargo.toml",
            "cloud/other/crates/oya-other",
            None, // this manifest itself does NOT move
            &name_to_move,
            &|old| {
                if old == "cloud/cloud-iam/crates/oya-domain" {
                    Some("iam/core/domain".to_string())
                } else {
                    None
                }
            },
        )
        .unwrap();
        assert!(changed);
        // dep key renamed
        assert!(out.contains("iam-domain = "), "renamed key: {out}");
        assert!(!out.contains("oya-domain ="));
        // path recomputed: from cloud/other/crates/oya-other to iam/core/domain
        // = ../../../../iam/core/domain
        assert!(
            out.contains("path = \"../../../../iam/core/domain\""),
            "recomputed deep path: {out}"
        );
    }

    #[test]
    fn untouched_manifest_is_byte_identical_no_change_flag() {
        let text = r#"[package]
name = "oya-unrelated"

[dependencies]
serde = { workspace = true }
"#;
        let name_to_move: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        let (out, changed) = rewrite_dependencies_in_manifest(
            text,
            "x/Cargo.toml",
            "x",
            None,
            &name_to_move,
            &|_| None,
        )
        .unwrap();
        assert!(!changed);
        assert_eq!(out, text, "no-op must be byte-identical");
    }

    #[test]
    fn target_cfg_dependency_path_is_recomputed() {
        let text = r#"[package]
name = "oya-iam"

[target.'cfg(unix)'.dependencies]
oya-domain = { path = "../oya-domain" }
"#;
        let name_to_move: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        let (out, changed) = rewrite_dependencies_in_manifest(
            text,
            "iam/core/iam/Cargo.toml",
            "cloud/cloud-iam/crates/oya-iam",
            Some("iam/core/iam"),
            &name_to_move,
            &|_| None,
        )
        .unwrap();
        assert!(changed, "cfg-target dep path must be recomputed");
        assert!(out.contains("../../cloud/cloud-iam/crates/oya-domain"));
    }

    /// D2 (RED before `[workspace.dependencies]` was walked): the EXACT shape carried by
    /// `cloud/cloud-kernel/Cargo.toml`, where 5 of 7 internal edges are path deps declared in
    /// `[workspace.dependencies]` under a SHORT ALIAS key with `package = ` naming the real
    /// crate. The old walker skipped `[workspace]` entirely, so every one of these survived a
    /// move still pointing at the emptied `crates/` dir.
    #[test]
    fn workspace_dependencies_path_and_package_alias_are_rewritten() {
        let text = r#"[workspace]
members = ["crates/*"]

[workspace.dependencies]
hal = { package = "oya-cloud-kernel-hal-kernel", path = "crates/oya-cloud-kernel-hal-kernel" }
tock-registers = "0.8"
"#;
        let hal = cm(
            "cloud/cloud-kernel/crates/oya-cloud-kernel-hal-kernel",
            "cloud/cloud-kernel/core/hal",
            "oya-cloud-kernel-hal-kernel",
            "kernel-hal",
        );
        let mut name_to_move: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        name_to_move.insert("oya-cloud-kernel-hal-kernel", &hal);
        let (out, changed) = rewrite_dependencies_in_manifest(
            text,
            "cloud/cloud-kernel/Cargo.toml",
            "cloud/cloud-kernel",
            None, // the workspace ROOT manifest is not itself a moved crate
            &name_to_move,
            &|old| {
                (old == "cloud/cloud-kernel/crates/oya-cloud-kernel-hal-kernel")
                    .then(|| "cloud/cloud-kernel/core/hal".to_string())
            },
        )
        .unwrap();
        assert!(changed, "[workspace.dependencies] must be rewritten");
        assert!(
            out.contains(r#"path = "core/hal""#),
            "path must be recomputed off the emptied crates/ dir: {out}"
        );
        assert!(
            out.contains(r#"package = "kernel-hal""#),
            "the `package = ` rename must follow the crate: {out}"
        );
        assert!(
            out.contains("hal = {"),
            "the ALIAS key is what `use hal::` binds; it must be PRESERVED: {out}"
        );
        assert!(
            out.contains(r#"tock-registers = "0.8""#),
            "registry deps untouched: {out}"
        );
    }

    /// A dependency declared under an alias (`foo = {{ package = "real-name" }}`) must have its
    /// `package` value renamed and its KEY left alone — renaming the key would silently rebind
    /// every `use foo::` in the source. Applies to ordinary `[dependencies]`, not just
    /// `[workspace.dependencies]`.
    #[test]
    fn aliased_dependency_renames_package_field_not_the_key() {
        let text = r#"[package]
name = "oya-consumer"

[dependencies]
hal = { package = "oya-hal", path = "../oya-hal" }
"#;
        let hal = cm("libs/oya-hal", "kernel/core/hal", "oya-hal", "kernel-hal");
        let mut name_to_move: BTreeMap<&str, &CrateMove> = BTreeMap::new();
        name_to_move.insert("oya-hal", &hal);
        let (out, changed) = rewrite_dependencies_in_manifest(
            text,
            "libs/oya-consumer/Cargo.toml",
            "libs/oya-consumer",
            None,
            &name_to_move,
            &|old| (old == "libs/oya-hal").then(|| "kernel/core/hal".to_string()),
        )
        .unwrap();
        assert!(changed);
        assert!(
            out.contains(r#"package = "kernel-hal""#),
            "package renamed: {out}"
        );
        assert!(out.contains("hal = {"), "alias key preserved: {out}");
        assert!(
            !out.contains("kernel-hal = {"),
            "the alias key must NOT be rewritten to the new crate name: {out}"
        );
        assert!(
            out.contains(r#"path = "../../kernel/core/hal""#),
            "path recomputed: {out}"
        );
    }

    #[test]
    fn root_members_gets_uncovered_new_dir_and_drops_stale_exclude() {
        let text = r#"[workspace]
members = ["libs/oya-*", "cloud/*/crates/oya-*"]
exclude = ["cloud/cloud-iam"]
resolver = "2"
"#;
        let (out, changed) = rewrite_root_workspace_members(
            text,
            &["iam/core/iam".to_string()],
            &[],
            &[],
            &["cloud/cloud-iam".to_string()],
        )
        .unwrap();
        assert!(changed);
        assert!(out.contains("\"iam/core/iam\""));
        assert!(!out.contains("cloud/cloud-iam"));
    }

    #[test]
    fn root_members_no_op_when_all_covered() {
        let text = "[workspace]\nmembers = [\"libs/oya-*\"]\n";
        let (out, changed) = rewrite_root_workspace_members(text, &[], &[], &[], &[]).unwrap();
        assert!(!changed);
        assert_eq!(out, text);
    }

    #[test]
    fn root_members_prunes_a_now_empty_glob() {
        // A move empties crates/ -> the crates/* glob would make cargo error; prune it.
        let text = "[workspace]\nmembers = [\"crates/*\", \"libs/*\", \"cap/core/cap-core\"]\n";
        let (out, changed) =
            rewrite_root_workspace_members(text, &[], &["crates/*".to_string()], &[], &[]).unwrap();
        assert!(changed);
        assert!(!out.contains("crates/*"), "empty glob pruned: {out}");
        assert!(out.contains("libs/*"));
        assert!(out.contains("cap/core/cap-core"));
    }
}
