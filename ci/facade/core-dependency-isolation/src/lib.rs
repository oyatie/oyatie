//! # cloud-ci-kernel-purity (ADR-0547)
//!
//! The kernel-purity dependency gate. Founder clean-architecture doctrine: a crate named
//! `*-kernel` or `*-core` is the **cutover-stable** seam — its interfaces must not change when the
//! ADR-0510 transient infrastructure (kube, sqlx, rustls, the AWS SDKs, etcd) is replaced at
//! owned-stack cutover. This gate asserts that no such crate — nor any workspace-internal crate
//! reachable through its path-dependency closure — directly depends on a denylisted transient-tech
//! crate, unless an explicit reasoned exception is declared in policy DATA. It converts the
//! enforcement half of FRIC-1781129000 (the cloud-kms operator's pure orchestration traits had
//! been colocated with the throwaway kube adapter; only reviewer eyes caught the mis-drawn seam).
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the kernel-name globs, the transient
//! denylist, the per-crate exceptions, the liveness floor — are DATA in
//! `kernel-purity-policy.json`. Nothing oyatie-specific is hardcoded in Rust; a different repo
//! adopts the gate by repointing the policy at its own crate tree.
//!
//! ## Kernel contract
//! - [`collect_kernel_deps`] `(root, policy) -> {crates:[..]}` enumerates workspace members, keeps
//!   those whose crate name matches a kernel glob, and for each walks the workspace-internal
//!   path-dependency closure, recording every reached crate's direct external Cargo + BUCK deps.
//!   Read-only; writes no temp files.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without
//!   a filesystem; it applies the deny set to each reached node's external deps and folds the
//!   exception allowlist.
//! - [`evaluate`] is the bare-code projection of `evaluate_keyed`, the single source of the verdict.
//!
//! ## Automation-default doctrine (founder directive 2026-06-11)
//! Automation is the default path; the blocking gate is the backstop layer (face-settle precedent).
//! - **Derivable + auto-fixable**: a denylisted dep declared in a crate's OWN manifest that is NOT
//!   referenced anywhere in that crate's `src/**/*.rs` is a dead transient dep — removing the
//!   manifest line is purely mechanical and safe (no code moves), subject to the five sound bounds
//!   of ADR-0547 D6 (never a build-dep, renamed dep, feature-referenced dep, or `optional = true`
//!   dep — optional deps export an implicit feature a sibling member can request). [`plan_fixes`]
//!   produces the exact edits and the gate binary applies them under `--fix` — Cargo.toml line
//!   removal plus the dead `third-party//:<dep>` rust_library BUCK edge, the latter via the shared
//!   oya-buck-syntax-kernel sound parser + write-through fixer harness (ADR-0549).
//! - **Not safely derivable**: a denylisted dep that IS used in the kernel's source requires moving
//!   the using code into a sibling `*-adapter` crate — a design act, never auto-applied. The gate
//!   still prints the best next action (which adapter to move to, when inferable), never a bare FAIL.
//!
//! Every finding therefore carries `auto_fixable` + `next_action`; see [`Finding`].
//!
//! ## Ratchet semantics
//! All kernel/core crates are pure today, so every blocking code ships frozen-empty: any new
//! occurrence fails closed. There is no shrink-only legacy baseline.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `KP-TRANSIENT-DEP-CARGO` — a kernel's closure node lists a denylisted transient dep in its
//!   Cargo.toml `[dependencies]`/`[build-dependencies]`/`[target.*.dependencies]`.
//! - `KP-TRANSIENT-DEP-BUCK`  — a kernel's closure node lists a denylisted transient dep in its
//!   BUCK `rust_library` `deps`.
//! - `KP-UNRESOLVED-PATH-DEP` — a kernel's closure node path-depends on a crate that is NOT a
//!   resolved workspace member (outside the member globs or under `exclude`); the scan cannot see
//!   that crate's deps, so it fails closed rather than letting the unscanned subtree be a false-green.
//! - `KP-STALE-EXCEPTION`     — a declared exception matches no live finding (self-cleaning;
//!   shrink-only by construction).
//! - `KP-EMPTY-SCAN`          — the scan found fewer kernel crates than `min_expected_kernel_crates`
//!   (catches a silently broken glob / CWD / collect that would otherwise be a false-green).
//! - `KP-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use oya_buck_syntax_kernel::{
    Env, Expr, PreImageRegistry, Stmt, call_strings, guarded_rewrite, remove_list_element,
};
use oya_workspace_members_kernel::resolve_member_dirs;
use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-kernel-purity";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 7] = [
    "KP-TRANSIENT-DEP-CARGO",
    "KP-TRANSIENT-DEP-BUCK",
    "KP-UNRESOLVED-PATH-DEP",
    "KP-STALE-EXCEPTION",
    "KP-EMPTY-SCAN",
    "KP-POLICY-GATE-ID-MISMATCH",
    "KP-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-crate.
const POLICY_KEY: &str = "<policy>";

/// Errors collecting the observed kernel-dep graph. The kernel returns these instead of panicking
/// so the caller (CI / a controller) decides how to surface them — a malformed manifest is a
/// fail-closed error, never a silently skipped crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    ResolveMembers(String),
    Io(String),
    Parse { path: String, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::ResolveMembers(message) => {
                write!(f, "resolve workspace members: {message}")
            }
            CollectError::Io(message) => write!(f, "kernel-purity io: {message}"),
            CollectError::Parse { path, message } => {
                write!(f, "manifest {path} is not valid: {message}")
            }
        }
    }
}

impl std::error::Error for CollectError {}

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only)
// ---------------------------------------------------------------------------

/// A single crate manifest as the collector parses it: its declared name, its member directory
/// (repo-relative), its direct external Cargo deps, its workspace-internal path-dep directories,
/// and its direct external BUCK `rust_library` deps.
#[derive(Debug, Clone, Default)]
struct CrateManifest {
    name: String,
    member_path: String,
    cargo_external_deps: BTreeSet<String>,
    internal_path_dirs: Vec<String>,
    buck_external_deps: BTreeSet<String>,
    /// The crate-ident tokens referenced anywhere in this crate's `src/**/*.rs` AND `build.rs`
    /// (the underscore form, since Rust paths use `kube_runtime`, not `kube-runtime`). Used to
    /// decide whether a declared transient dep is dead (auto-fixable) or live (design act).
    src_referenced_idents: BTreeSet<String>,
    /// Dep names declared in any `[build-dependencies]`/`[target.*.build-dependencies]` table. These
    /// are NEVER auto-fixable: build-script dep liveness is hard to attribute per-dep, so the safe
    /// default is a design action, never a mechanical removal.
    build_dep_names: BTreeSet<String>,
    /// Deps that are renamed: maps real crate name -> local dep key (e.g. `kube` -> `k8s`).
    /// HIGH-2: when a dep is renamed, `src` uses the rename key (`k8s::`) not the real name (`kube::`).
    /// The liveness check must probe BOTH idents; if either is live the dep is not auto-fixable.
    /// Additionally, any renamed dep is conservatively demoted to design-action (never auto-fixed)
    /// because line removal by real-name key alone would leave orphaned `k8s.workspace = true` lines.
    cargo_dep_rename_keys: BTreeMap<String, String>,
    /// Real crate names of deps declared as `optional = true` that are also referenced by a
    /// `[features]` entry as `dep:<name>`. Removing such a dep line would leave the feature entry
    /// dangling (cargo error: "feature includes dep:X but X is not a dep"), so these are NEVER
    /// auto-fixed — they are demoted to design-action. CRITICAL-1 fix.
    feature_backed_optional_deps: BTreeSet<String>,
    /// Real names + dep keys of ALL deps declared `optional = true`, regardless of any [features]
    /// mention. MED-X1 sound bound: every optional dep exports an IMPLICIT cargo feature named
    /// after itself even when the owning manifest's [features] never references it; a SIBLING
    /// workspace member can request that feature (`features = ["kube"]`) on its path dep, which
    /// neither the own-manifest [features] scan (layer 1) nor `cargo metadata --no-deps` (layer 2,
    /// no cross-member feature resolution) can see. Optional deps are therefore NEVER auto-fixed.
    optional_dep_names: BTreeSet<String>,
}

/// Collect the kernel-purity dep graph described by the policy.
///
/// Enumerates workspace members, keeps the kernel-glob matches, and for each walks the
/// workspace-internal path-dep closure (so a kernel that absorbs a transient-carrying local adapter
/// is caught). Emits, for each kernel:
/// `{ "kernel": <name>, "member_path": <dir>, "closure": [ { "name", "member_path", "via":[..],
///    "cargo_deps":[..], "buck_deps":[..] } ] }`.
/// The output is `{ "kernel_crates_found": <usize>, "crates": [ <kernel>, .. ] }`.
pub fn collect_kernel_deps(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let globs = kernel_globs(policy);
    let member_dirs = resolve_member_dirs(root)
        .map_err(|error| CollectError::ResolveMembers(error.to_string()))?;

    // Parse every member manifest once; index by both name and directory for closure resolution.
    let mut by_name: BTreeMap<String, CrateManifest> = BTreeMap::new();
    let mut by_dir: BTreeMap<String, String> = BTreeMap::new();
    for member_dir in &member_dirs {
        let manifest = parse_member_manifest(root, member_dir)?;
        by_dir.insert(normalize_dir(member_dir), manifest.name.clone());
        by_name.insert(manifest.name.clone(), manifest);
    }

    let mut kernels = Vec::new();
    for manifest in by_name.values() {
        if !name_matches_any_glob(&manifest.name, &globs) {
            continue;
        }
        let closure = build_closure(manifest, &by_name, &by_dir);
        kernels.push(json!({
            "kernel": manifest.name,
            "member_path": manifest.member_path,
            "closure": closure,
        }));
    }

    Ok(json!({
        "kernel_crates_found": kernels.len(),
        "crates": kernels,
    }))
}

/// Walk the workspace-internal path-dep closure of `start`, deterministically (BTreeSet visited +
/// sorted iteration). Each closure node records the `via` chain (the path of crate names from the
/// kernel to the node) so a finding can name how a transient dep reaches the kernel.
fn build_closure(
    start: &CrateManifest,
    by_name: &BTreeMap<String, CrateManifest>,
    by_dir: &BTreeMap<String, String>,
) -> Vec<Value> {
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<Value> = Vec::new();
    // Stack of (crate-name, via-chain-to-this-crate).
    let mut stack: Vec<(String, Vec<String>)> =
        vec![(start.name.clone(), vec![start.name.clone()])];
    while let Some((name, via)) = stack.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(manifest) = by_name.get(&name) else {
            continue;
        };
        // For each external dep, record whether its crate ident (or rename key) is referenced in
        // this node's src. The evaluator uses this to classify a finding as auto-fixable (dead dep,
        // mechanical removal) vs a design act (live dep, code must move to an adapter).
        // HIGH-2: if the dep is renamed (foo = { package = "kube" }), src uses the rename key
        // `foo::` not the real name `kube::`. Probe both; if either is live the dep is not dead.
        let cargo_used: BTreeMap<String, bool> = manifest
            .cargo_external_deps
            .iter()
            .map(|dep| {
                let rename_key = manifest.cargo_dep_rename_keys.get(dep.as_str());
                let used = dep_used_in_src(dep, &manifest.src_referenced_idents)
                    || rename_key
                        .is_some_and(|k| dep_used_in_src(k, &manifest.src_referenced_idents));
                (dep.clone(), used)
            })
            .collect();
        // Path deps that do not resolve to a workspace member: a kernel could path-depend on a
        // crate OUTSIDE the member globs (or under `exclude`), which builds under cargo but would
        // vanish from this closure scan — a silent false-green. Record them so the evaluator emits
        // a fail-closed finding (honoring the crate's own "never silently skip" doctrine).
        let mut unresolved_path_deps: Vec<String> = Vec::new();
        for dep_dir in &manifest.internal_path_dirs {
            let normalized = normalize_dir(dep_dir);
            if let Some(dep_name) = by_dir.get(&normalized) {
                if !visited.contains(dep_name) {
                    let mut next_via = via.clone();
                    next_via.push(dep_name.clone());
                    stack.push((dep_name.clone(), next_via));
                }
            } else {
                unresolved_path_deps.push(normalized);
            }
        }
        unresolved_path_deps.sort();
        // Emit rename keys as a JSON object { real_name: dep_key, ... } so the evaluator can
        // conservatively demote any renamed dep to design-action (never auto-fix).
        let rename_keys_json: serde_json::Map<String, Value> = manifest
            .cargo_dep_rename_keys
            .iter()
            .map(|(real, key)| (real.clone(), Value::from(key.clone())))
            .collect();
        // Emit feature-backed optional dep names as a JSON array so the evaluator can refuse
        // auto-fix for deps whose removal would leave dangling `dep:X` feature entries.
        let feature_backed_json: Vec<Value> = manifest
            .feature_backed_optional_deps
            .iter()
            .map(|s| Value::from(s.clone()))
            .collect();
        // Emit ALL optional dep names so the evaluator can refuse auto-fix for every optional dep
        // (MED-X1: the implicit-feature export is invisible to both guard layers).
        let optional_json: Vec<Value> = manifest
            .optional_dep_names
            .iter()
            .map(|s| Value::from(s.clone()))
            .collect();
        out.push(json!({
            "name": manifest.name,
            "member_path": manifest.member_path,
            "via": via,
            "cargo_deps": manifest.cargo_external_deps.iter().cloned().collect::<Vec<_>>(),
            "buck_deps": manifest.buck_external_deps.iter().cloned().collect::<Vec<_>>(),
            "cargo_dep_used_in_src": cargo_used,
            "build_dep_names": manifest.build_dep_names.iter().cloned().collect::<Vec<_>>(),
            "cargo_dep_rename_keys": rename_keys_json,
            "feature_backed_optional_deps": feature_backed_json,
            "optional_dep_names": optional_json,
            "unresolved_path_deps": unresolved_path_deps,
        }));
    }
    // Deterministic order independent of stack traversal.
    out.sort_by(|a, b| {
        a["name"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["name"].as_str().unwrap_or_default())
    });
    out
}

fn parse_member_manifest(root: &Path, member_dir: &str) -> Result<CrateManifest, CollectError> {
    let cargo_path = root.join(member_dir).join("Cargo.toml");
    let cargo_text = fs::read_to_string(&cargo_path)
        .map_err(|e| CollectError::Io(format!("read {}: {e}", cargo_path.display())))?;
    let document: toml::Value = toml::from_str(&cargo_text).map_err(|e| CollectError::Parse {
        path: format!("{member_dir}/Cargo.toml"),
        message: e.to_string(),
    })?;

    let name = document
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| CollectError::Parse {
            path: format!("{member_dir}/Cargo.toml"),
            message: "missing [package].name".to_owned(),
        })?;

    let mut cargo_external_deps = BTreeSet::new();
    let mut internal_path_dirs = Vec::new();
    let mut build_dep_names = BTreeSet::new();
    let mut cargo_dep_rename_keys = BTreeMap::new();
    let mut feature_backed_optional_deps = BTreeSet::new();
    let mut optional_dep_names = BTreeSet::new();
    collect_cargo_deps(
        &document,
        member_dir,
        &mut cargo_external_deps,
        &mut internal_path_dirs,
        &mut build_dep_names,
        &mut cargo_dep_rename_keys,
        &mut feature_backed_optional_deps,
        &mut optional_dep_names,
    );

    let buck_external_deps = parse_buck_external_deps(root, member_dir)?;
    let src_referenced_idents = scan_src_referenced_idents(root, member_dir)?;

    Ok(CrateManifest {
        name,
        member_path: member_dir.to_owned(),
        cargo_external_deps,
        internal_path_dirs,
        buck_external_deps,
        src_referenced_idents,
        build_dep_names,
        cargo_dep_rename_keys,
        feature_backed_optional_deps,
        optional_dep_names,
    })
}

/// Collect the crate-ident tokens referenced in `<member>/src/**/*.rs` AND `<member>/build.rs`. A
/// dependency is "used" iff its crate ident (the dep name with `-` mapped to `_`, the Rust path
/// form) appears as a token in the source. This is a conservative over-approximation (a name in a
/// comment counts as used), which is the SAFE direction: it only ever marks a dep as "live" (not
/// auto-fixable), never wrongly auto-removes a used dep. `build.rs` is scanned because that is the
/// only legitimate usage site for a build-dependency; missing files are fine (returns nothing).
fn scan_src_referenced_idents(
    root: &Path,
    member_dir: &str,
) -> Result<BTreeSet<String>, CollectError> {
    let member_root = root.join(member_dir);
    let mut idents = BTreeSet::new();
    collect_rs_idents(&member_root.join("src"), &mut idents)?;
    collect_rs_file_idents(&member_root.join("build.rs"), &mut idents)?;
    Ok(idents)
}

/// Scan a single `.rs` file (e.g. a crate-root `build.rs`) for ident tokens. Missing file is fine.
fn collect_rs_file_idents(path: &Path, idents: &mut BTreeSet<String>) -> Result<(), CollectError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read {}: {e}", path.display()))),
    };
    for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        if !token.is_empty() {
            idents.insert(token.to_owned());
        }
    }
    Ok(())
}

fn collect_rs_idents(dir: &Path, idents: &mut BTreeSet<String>) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if file_type.is_dir() {
            collect_rs_idents(&path, idents)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            let text = fs::read_to_string(&path)
                .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
            for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                if !token.is_empty() {
                    idents.insert(token.to_owned());
                }
            }
        }
    }
    Ok(())
}

/// Whether a dependency crate name is referenced in a crate's source idents (dep `-` → `_`).
fn dep_used_in_src(dep: &str, src_idents: &BTreeSet<String>) -> bool {
    let ident = dep.replace('-', "_");
    src_idents.contains(&ident)
}

/// Collect a manifest's direct external Cargo deps + workspace-internal path-dep directories from
/// `[dependencies]`, `[build-dependencies]`, `[target.*.dependencies]`, and
/// `[target.*.build-dependencies]`. dev-dependencies are deliberately excluded (test-only, never
/// shipped in the kernel). A dep with a `path` key is
/// an internal path dep (recorded as a directory for closure resolution); every other dep is an
/// external crate (recorded by its dependency key, which is the crate name unless renamed — and a
/// rename still carries `package = "<real>"`, handled below).
#[allow(clippy::too_many_arguments)]
fn collect_cargo_deps(
    document: &toml::Value,
    member_dir: &str,
    external: &mut BTreeSet<String>,
    path_dirs: &mut Vec<String>,
    build_dep_names: &mut BTreeSet<String>,
    rename_keys: &mut BTreeMap<String, String>,
    feature_backed: &mut BTreeSet<String>,
    optional_names: &mut BTreeSet<String>,
) {
    if let Some(table) = document.get("dependencies").and_then(toml::Value::as_table) {
        collect_dep_table(
            table,
            member_dir,
            external,
            path_dirs,
            rename_keys,
            optional_names,
        );
    }
    if let Some(table) = document
        .get("build-dependencies")
        .and_then(toml::Value::as_table)
    {
        collect_dep_table(
            table,
            member_dir,
            external,
            path_dirs,
            rename_keys,
            optional_names,
        );
        record_build_dep_names(table, build_dep_names);
    }
    if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
        for target_cfg in targets.values() {
            if let Some(table) = target_cfg
                .get("dependencies")
                .and_then(toml::Value::as_table)
            {
                collect_dep_table(
                    table,
                    member_dir,
                    external,
                    path_dirs,
                    rename_keys,
                    optional_names,
                );
            }
            // `[target.'cfg(..)'.build-dependencies]` — a legal, rare placement that must also be
            // scanned so a transient build-dep behind a cfg cannot false-green.
            if let Some(table) = target_cfg
                .get("build-dependencies")
                .and_then(toml::Value::as_table)
            {
                collect_dep_table(
                    table,
                    member_dir,
                    external,
                    path_dirs,
                    rename_keys,
                    optional_names,
                );
                record_build_dep_names(table, build_dep_names);
            }
        }
    }
    // CRITICAL-A sound bound layer 1: collect all dep names/rename-keys referenced in ANY [features]
    // value string across ALL dep tables (handles dep:X, X, X/feat, X?/feat, target-cfg deps).
    // A dep whose name or rename key appears in any feature value cannot be safely auto-removed —
    // removing it would leave a dangling feature entry that cargo rejects.
    collect_features_referenced_deps(document, feature_backed);
}

/// Collect the real names + dep-keys of deps that are referenced by any `[features]` value string
/// in ANY syntax — `dep:X`, `X`, `X/feat`, `X?/feat` (the `?` optional-dep activation syntax).
/// CRITICAL-A sound bound layer 1: uses substring-token matching against all known dep names and
/// rename keys so that H1 (`["kube/client"]`), H2 (`["kube?/client"]`), H3 (`["kube"]` bare),
/// and H4 (any dep table section) are all caught. Scans ALL dep tables across the document so a
/// dep declared only in `[target.'cfg(unix)'.dependencies]` is covered.
fn collect_features_referenced_deps(document: &toml::Value, feature_backed: &mut BTreeSet<String>) {
    // Step 1: collect ALL known dep names and rename keys from ALL dep tables (every section).
    let mut all_dep_keys: BTreeSet<String> = BTreeSet::new();
    let section_names = ["dependencies", "build-dependencies", "dev-dependencies"];
    for &sec in &section_names {
        if let Some(table) = document.get(sec).and_then(toml::Value::as_table) {
            for (dep_key, spec) in table {
                all_dep_keys.insert(dep_key.clone());
                if let Some(spec_table) = spec.as_table()
                    && let Some(pkg) = spec_table.get("package").and_then(toml::Value::as_str)
                {
                    all_dep_keys.insert(pkg.to_owned());
                }
            }
        }
    }
    if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
        for target_val in targets.values() {
            for &sec in &["dependencies", "build-dependencies", "dev-dependencies"] {
                if let Some(table) = target_val.get(sec).and_then(toml::Value::as_table) {
                    for (dep_key, spec) in table {
                        all_dep_keys.insert(dep_key.clone());
                        if let Some(spec_table) = spec.as_table()
                            && let Some(pkg) =
                                spec_table.get("package").and_then(toml::Value::as_str)
                        {
                            all_dep_keys.insert(pkg.to_owned());
                        }
                    }
                }
            }
        }
    }
    if all_dep_keys.is_empty() {
        return;
    }

    // Step 2: scan ALL [features] value strings for any token that is a substring-match of a
    // known dep key, using the Cargo feature-entry token grammar:
    //   dep:X          → explicit dep activation (optional)
    //   dep:X/feat     → optional dep activation with sub-feature
    //   X              → bare dep name or feature name (not distinguishable without full resolver)
    //   X/feat         → dep with sub-feature
    //   X?/feat        → optional dep activation with sub-feature
    // We extract the lead token before any `/` or `?` and after any `dep:` prefix.
    // If that token matches any known dep key, the dep is feature-referenced.
    let mut feature_referenced: BTreeSet<String> = BTreeSet::new();
    if let Some(features) = document.get("features").and_then(toml::Value::as_table) {
        for values in features.values() {
            if let Some(list) = values.as_array() {
                for entry in list {
                    if let Some(s) = entry.as_str() {
                        // Strip `dep:` prefix if present.
                        let without_dep_prefix = s.strip_prefix("dep:").unwrap_or(s);
                        // Take the part before any `/` (sub-feature path), THEN strip the
                        // optional-activation `?` suffix (e.g. `kube?/client` → `kube?` → `kube`).
                        let lead = without_dep_prefix
                            .split('/')
                            .next()
                            .unwrap_or(without_dep_prefix);
                        let token = lead.trim_end_matches('?').trim();
                        if !token.is_empty() && all_dep_keys.contains(token) {
                            feature_referenced.insert(token.to_owned());
                        }
                    }
                }
            }
        }
    }

    // Step 3: for every feature-referenced token, add both the real name and the dep key (they may
    // differ for renamed deps) to feature_backed so the evaluator can refuse auto-fix for all forms.
    for &sec in &section_names {
        if let Some(table) = document.get(sec).and_then(toml::Value::as_table) {
            for (dep_key, spec) in table {
                let real = if let Some(spec_table) = spec.as_table() {
                    spec_table
                        .get("package")
                        .and_then(toml::Value::as_str)
                        .unwrap_or(dep_key)
                } else {
                    dep_key.as_str()
                };
                if feature_referenced.contains(dep_key.as_str())
                    || feature_referenced.contains(real)
                {
                    feature_backed.insert(real.to_owned());
                    feature_backed.insert(dep_key.clone());
                }
            }
        }
    }
    if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
        for target_val in targets.values() {
            for &sec in &section_names {
                if let Some(table) = target_val.get(sec).and_then(toml::Value::as_table) {
                    for (dep_key, spec) in table {
                        let real = if let Some(spec_table) = spec.as_table() {
                            spec_table
                                .get("package")
                                .and_then(toml::Value::as_str)
                                .unwrap_or(dep_key)
                        } else {
                            dep_key.as_str()
                        };
                        if feature_referenced.contains(dep_key.as_str())
                            || feature_referenced.contains(real)
                        {
                            feature_backed.insert(real.to_owned());
                            feature_backed.insert(dep_key.clone());
                        }
                    }
                }
            }
        }
    }
}

/// Record the real crate names of a build-dependency table (honoring `package = "<real>"`), so the
/// evaluator can keep build-deps out of the auto-fix class.
fn record_build_dep_names(
    table: &toml::map::Map<String, toml::Value>,
    build_dep_names: &mut BTreeSet<String>,
) {
    for (dep_key, spec) in table {
        if let Some(spec_table) = spec.as_table() {
            if spec_table.contains_key("path") {
                continue;
            }
            let real = spec_table
                .get("package")
                .and_then(toml::Value::as_str)
                .unwrap_or(dep_key);
            build_dep_names.insert(real.to_owned());
        } else {
            build_dep_names.insert(dep_key.clone());
        }
    }
}

fn collect_dep_table(
    table: &toml::map::Map<String, toml::Value>,
    member_dir: &str,
    external: &mut BTreeSet<String>,
    path_dirs: &mut Vec<String>,
    rename_keys: &mut BTreeMap<String, String>,
    optional_names: &mut BTreeSet<String>,
) {
    for (dep_key, spec) in table {
        if let Some(spec_table) = spec.as_table() {
            if let Some(path) = spec_table.get("path").and_then(toml::Value::as_str) {
                path_dirs.push(join_relative(member_dir, path));
                continue;
            }
            // A renamed external dep carries `package = "<real-crate>"`; deny on the real name but
            // also record the rename key so liveness can probe src for the rename ident too.
            let real = spec_table
                .get("package")
                .and_then(toml::Value::as_str)
                .unwrap_or(dep_key);
            external.insert(real.to_owned());
            if real != dep_key.as_str() {
                // real_name -> local dep_key (e.g. "kube" -> "k8s")
                rename_keys.insert(real.to_owned(), dep_key.clone());
            }
            // MED-X1: record EVERY `optional = true` dep (real name + dep key), independent of any
            // [features] mention — the implicit feature it exports can be requested by a sibling
            // workspace member, which no local scan or `--no-deps` validation can see.
            if spec_table.get("optional").and_then(toml::Value::as_bool) == Some(true) {
                optional_names.insert(real.to_owned());
                optional_names.insert(dep_key.clone());
            }
        } else {
            // Bare `dep = "x.y"` form — external by the dependency key (the crate name).
            external.insert(dep_key.clone());
        }
    }
}

/// Parse the BUCK `rust_library` target `deps` for external `third-party//:<name>` references.
/// Local `//path:...` deps are workspace-internal (already covered by the Cargo path-dep closure)
/// and are ignored here; `rust_test` deps are intentionally not parsed (test-only). A missing BUCK
/// file is not an error — the gate is repo-portable and some adopters may not use buck2.
fn parse_buck_external_deps(
    root: &Path,
    member_dir: &str,
) -> Result<BTreeSet<String>, CollectError> {
    let buck_path = root.join(member_dir).join("BUCK");
    let text = match fs::read_to_string(&buck_path) {
        Ok(text) => text,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => {
            return Err(CollectError::Io(format!(
                "read {}: {e}",
                buck_path.display()
            )));
        }
    };
    Ok(extract_buck_library_thirdparty_deps(&text))
}

/// Pure extraction via the shared sound `oya-buck-syntax-kernel` parser (ADR-0549): parse the
/// BUCK text, and for each top-level `rust_library(...)` call collect every
/// `third-party//:<name>` token from its STRING VALUES. `rust_test`/`rust_binary` blocks are
/// skipped. Sound by construction against the historical detect-gap classes:
/// - a stray paren in a comment or string cannot end the block early (#691 H5 / #693 LOW-X2);
/// - a backslash-newline continuation inside a string cooks to the JOINED value, so
///   `"third-party//:k\` + newline + `ube"` is detected as `kube` (#693 LOW-2,
///   FRIC-1781230000);
/// - a `third-party//:` mention in a COMMENT is trivia, never a dep (comment-blind class).
///
/// Fail-closed posture for what the subset cannot model: a call containing an opaque
/// (unmodeled) argument shape is ALSO raw-scanned over its exact span, and a BUCK text that
/// does not parse soundly is raw-scanned in full — an over-approximation that can only ADD
/// findings for this born-blocking detector, never hide one (the same posture the pre-kernel
/// EOF fallback took; the REMOVER path is independently guarded and refuses unsound input).
pub fn extract_buck_library_thirdparty_deps(text: &str) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    let Ok(doc) = oya_buck_syntax_kernel::parse(text) else {
        collect_thirdparty_tokens(text, &mut deps);
        return deps;
    };
    // Review F2 (value-indirection evasion): two shapes route a target's deps AROUND any
    // span-local scan — a kwargs splat (`KW = {"deps": [...]}; rust_library(**KW)`, where the
    // deps live in a CLEAN dict assignment far from the opaque call) and a load-aliased rule
    // name (`load(":d.bzl", my_lib = "rust_library")` + `my_lib(deps = [...])`, where the call
    // never bears the rust_library name). NO SILENT MISS: when either trigger is present the
    // WHOLE FILE is raw-scanned (a detector over-scan can only ADD findings; macro-wrapped
    // invocations defined in .bzl bodies remain the ledgered residual class).
    let mut widen_to_whole_file = false;
    doc.visit_calls(&mut |call| {
        if call.has_opaque() {
            // Any opaque-args call (incl. `**KW` splats): the unmodeled content may REFERENCE
            // values assembled anywhere in the file.
            widen_to_whole_file = true;
        }
        if call.func == "load"
            && call
                .args
                .iter()
                .any(|arg| matches!(&arg.value.expr, Expr::Str(s) if s == "rust_library"))
        {
            // A load() binding (aliasing) of rust_library: later calls may carry any name.
            widen_to_whole_file = true;
        }
    });
    if widen_to_whole_file {
        collect_thirdparty_tokens(text, &mut deps);
    }
    // Enumerate EVERY rust_library call in the document — top-level statements AND calls
    // wrapped in assignments or nested in expressions (`X = rust_library(...)`), so a one-token
    // wrapper can never be a dep-hiding device (reviewer BLOCKER on the statement-position-only
    // enumeration).
    doc.visit_calls(&mut |call| {
        if call.func == "rust_library" {
            for value in call_strings(call) {
                collect_thirdparty_tokens(&value, &mut deps);
            }
        }
        // ANY call carrying opaque content gets its span raw-scanned from a `rust_library(`
        // occurrence — a rust_library call discarded into an opaque ARGUMENT of some other
        // call (`helper([rust_library(...)][0])`) must not escape just because the WRAPPING
        // call has a different name. Together with the statement-level arms below this makes
        // the dichotomy total: opaque content always sits inside some visited call's span or
        // a flagged statement span.
        if call.has_opaque() {
            let raw = call.span.slice(text);
            if call.func == "rust_library" {
                collect_thirdparty_tokens(raw, &mut deps);
            } else if let Some(at) = raw.find("rust_library(") {
                collect_thirdparty_tokens(&raw[at..], &mut deps);
            }
        }
    });
    // Spans the subset could not (fully) model carry calls visit_calls cannot reach; if such a
    // span mentions a rust_library call at all, raw-scan it from that point (the same
    // over-approximation the pre-kernel scanner applied to ALL text) — a detector over-scan can
    // only ADD findings. Covered shapes:
    // - Stmt::Opaque (unmodeled statements, incl. trailing-token demotions);
    // - Assign/IndexAssign whose value (or key) contains expression-level Opaque content — the
    //   re-review BLOCKER class: a postfix-index wrapper (`X = [rust_library(...)][0]`), an
    //   unmodeled-primary ternary (`X = -1 if c else rust_library(...)`), or a discarded
    //   comprehension iter (`{k: v for k in [rust_library(...)]}`) all collapse to Opaque
    //   inside a still-modeled statement, invisible to both visit_calls and the opaque-stmt
    //   scan without this arm.
    for stmt in &doc.stmts {
        let opaque_span = match stmt {
            Stmt::Opaque { span } => Some(*span),
            Stmt::Assign { value, span, .. } if value.has_opaque() => Some(*span),
            Stmt::IndexAssign {
                key, value, span, ..
            } if key.has_opaque() || value.has_opaque() => Some(*span),
            _ => None,
        };
        let Some(span) = opaque_span else { continue };
        let raw = span.slice(text);
        if let Some(at) = raw.find("rust_library(") {
            collect_thirdparty_tokens(&raw[at..], &mut deps);
        }
    }
    deps
}

/// Collect every `third-party//:<name>` token in `text` into `deps`.
fn collect_thirdparty_tokens(text: &str, deps: &mut BTreeSet<String>) {
    let marker = "third-party//:";
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(marker) {
        let start = from + rel + marker.len();
        let name: String = text[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if !name.is_empty() {
            deps.insert(name);
        }
        from = start;
    }
}

// ---------------------------------------------------------------------------
// Pure evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub detail: String,
    /// Automation-default doctrine: true iff this violation is mechanically removable by the gate's
    /// `--fix` (a dead transient dep declared in a crate's own manifest, unreferenced in its src).
    pub auto_fixable: bool,
    /// The best next action printed to the contributor, always populated — never a bare FAIL.
    pub next_action: String,
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
            auto_fixable: false,
            next_action: String::new(),
        }
    }

    fn with_action(mut self, auto_fixable: bool, next_action: impl Into<String>) -> Self {
        self.auto_fixable = auto_fixable;
        self.next_action = next_action.into();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_findings(findings: &BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect::<BTreeSet<_>>();
        Self {
            verdict: if violations.is_empty() {
                Verdict::Green
            } else {
                Verdict::Red
            },
            violations,
        }
    }
}

/// One deny rule, parsed from policy DATA.
#[derive(Debug, Clone)]
struct DenyRule {
    dep: String,
    prefix: bool,
}

impl DenyRule {
    fn matches(&self, dep: &str) -> bool {
        if self.prefix {
            dep.starts_with(&self.dep)
        } else {
            dep == self.dep
        }
    }
}

/// Parse deny rules from policy DATA. Returns Err with a human-readable message on any malformed
/// entry — missing `dep` key, missing/unknown `match` value — so the evaluator can emit
/// `KP-POLICY-MALFORMED` and fail CLOSED instead of silently dropping rules (HIGH-3).
fn parse_deny_rules(policy: &Value) -> Result<Vec<DenyRule>, String> {
    let Some(rules) = policy.get("deny").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, rule) in rules.iter().enumerate() {
        let dep = rule
            .get("dep")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("policy deny[{i}]: missing or non-string `dep` key"))?
            .to_owned();
        let match_val = rule.get("match").and_then(Value::as_str).ok_or_else(|| {
            format!("policy deny[{i}] (dep={dep:?}): missing or non-string `match` key")
        })?;
        let prefix = match match_val {
            "exact" => false,
            "prefix" => true,
            other => {
                return Err(format!(
                    "policy deny[{i}] (dep={dep:?}): unknown `match` value {other:?}; must be \"exact\" or \"prefix\""
                ));
            }
        };
        out.push(DenyRule { dep, prefix });
    }
    Ok(out)
}

/// One exception, parsed from policy DATA: a per-(crate, dep) reasoned allowlist entry.
#[derive(Debug, Clone)]
struct Exception {
    crate_name: String,
    dep: String,
}

/// Parse exceptions from policy DATA. Returns Err on any malformed entry (HIGH-3).
fn parse_exceptions(policy: &Value) -> Result<Vec<Exception>, String> {
    let Some(values) = policy.get("exceptions").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, value) in values.iter().enumerate() {
        let crate_name = value
            .get("crate")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("policy exceptions[{i}]: missing or non-string `crate` key"))?
            .to_owned();
        let dep = value
            .get("dep")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "policy exceptions[{i}] (crate={crate_name:?}): missing or non-string `dep` key"
                )
            })?
            .to_owned();
        out.push(Exception { crate_name, dep });
    }
    Ok(out)
}

fn first_deny_hit<'a>(rules: &'a [DenyRule], dep: &str) -> Option<&'a DenyRule> {
    rules.iter().find(|rule| rule.matches(dep))
}

/// Pure evaluator. `policy` is DATA (`kernel-purity-policy.json`); `observed` is the collected
/// graph shaped by [`collect_kernel_deps`].
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "KP-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // HIGH-3: fail CLOSED on malformed policy entries rather than silently dropping rules.
    let deny_rules = match parse_deny_rules(policy) {
        Ok(rules) => rules,
        Err(message) => {
            findings.insert(Finding::new(
                "KP-POLICY-MALFORMED",
                POLICY_KEY,
                format!("deny rules malformed — {message}; the policy must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };
    let exceptions = match parse_exceptions(policy) {
        Ok(ex) => ex,
        Err(message) => {
            findings.insert(Finding::new(
                "KP-POLICY-MALFORMED",
                POLICY_KEY,
                format!("exceptions malformed — {message}; the policy must be corrected before the gate can evaluate"),
            ));
            return findings;
        }
    };
    let min_expected = policy
        .get("min_expected_kernel_crates")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    let kernel_count = observed
        .get("kernel_crates_found")
        .and_then(Value::as_u64)
        .or_else(|| {
            observed
                .get("crates")
                .and_then(Value::as_array)
                .map(|crates| crates.len() as u64)
        })
        .unwrap_or(0);
    if kernel_count < min_expected {
        findings.insert(Finding::new(
            "KP-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {kernel_count} kernel/core crates, below the policy floor of {min_expected}; the kernel glob, CWD, or collection is likely broken (fail-closed against a silent false-green)"
            ),
        ));
    }

    // Track which exceptions were used so unused ones become KP-STALE-EXCEPTION.
    let mut used_exceptions: BTreeSet<(String, String)> = BTreeSet::new();

    let crates = observed
        .get("crates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for kernel in &crates {
        let Some(kernel_name) = kernel.get("kernel").and_then(Value::as_str) else {
            continue;
        };
        let closure = kernel
            .get("closure")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for node in &closure {
            let Some(node_name) = node.get("name").and_then(Value::as_str) else {
                continue;
            };
            let via = node
                .get("via")
                .and_then(Value::as_array)
                .map(|chain| {
                    chain
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" -> ")
                })
                .unwrap_or_else(|| node_name.to_owned());

            evaluate_dep_list(
                kernel_name,
                node,
                node_name,
                &via,
                node.get("cargo_deps"),
                "KP-TRANSIENT-DEP-CARGO",
                "Cargo.toml",
                &deny_rules,
                &exceptions,
                &mut used_exceptions,
                &mut findings,
            );
            evaluate_dep_list(
                kernel_name,
                node,
                node_name,
                &via,
                node.get("buck_deps"),
                "KP-TRANSIENT-DEP-BUCK",
                "BUCK",
                &deny_rules,
                &exceptions,
                &mut used_exceptions,
                &mut findings,
            );

            // Fail-closed on a path dep the collector could not resolve to a workspace member: the
            // unscanned subtree must not be a silent false-green.
            if let Some(unresolved) = node.get("unresolved_path_deps").and_then(Value::as_array) {
                for dir in unresolved.iter().filter_map(Value::as_str) {
                    let key = format!("{kernel_name}:{node_name}:{dir}");
                    findings.insert(
                        Finding::new(
                            "KP-UNRESOLVED-PATH-DEP",
                            &key,
                            format!(
                                "kernel `{kernel_name}` (node `{node_name}`) path-depends on `{dir}`, which is not a resolved workspace member; the gate cannot scan its deps (ADR-0547)"
                            ),
                        )
                        .with_action(
                            false,
                            format!(
                                "DESIGN ACTION (not auto-applied): make `{dir}` a workspace member so it is scanned, or remove the out-of-workspace path dependency from the kernel `{node_name}`"
                            ),
                        ),
                    );
                }
            }
        }
    }

    for exception in &exceptions {
        let key = (exception.crate_name.clone(), exception.dep.clone());
        if !used_exceptions.contains(&key) {
            findings.insert(Finding::new(
                "KP-STALE-EXCEPTION",
                &format!("{}:{}", exception.crate_name, exception.dep),
                format!(
                    "exception for crate `{}` dep `{}` matched no live finding; remove it (exceptions are shrink-only)",
                    exception.crate_name, exception.dep
                ),
            ));
        }
    }

    findings
}

#[allow(clippy::too_many_arguments)]
fn evaluate_dep_list(
    kernel_name: &str,
    node: &Value,
    node_name: &str,
    via: &str,
    deps: Option<&Value>,
    code: &str,
    source_label: &str,
    deny_rules: &[DenyRule],
    exceptions: &[Exception],
    used_exceptions: &mut BTreeSet<(String, String)>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(deps) = deps.and_then(Value::as_array) else {
        return;
    };
    let member_path = node
        .get("member_path")
        .and_then(Value::as_str)
        .unwrap_or("");
    for dep in deps {
        let Some(dep) = dep.as_str() else {
            continue;
        };
        if first_deny_hit(deny_rules, dep).is_none() {
            continue;
        }
        // An exception is keyed on the crate that actually CARRIES the dep (the closure node), not
        // the kernel, so a shared adapter's carve-out is declared once.
        if let Some(exception) = exceptions
            .iter()
            .find(|exception| exception.crate_name == node_name && exception.dep == dep)
        {
            used_exceptions.insert((exception.crate_name.clone(), exception.dep.clone()));
            continue;
        }

        // Automation-default classification (founder directive 2026-06-11). A dep is auto-fixable
        // ONLY when ALL of the following hold:
        //  1. NOT referenced in that node's src (or build.rs) — dead dep.
        //  2. NOT a build-dependency — build.rs liveness is hard to attribute per-dep (BLOCKER-1).
        //  3. NOT a renamed dep — the rename key is used in src; removing by real name would also
        //     leave orphaned `rename.workspace = true` lines (HIGH-2).
        //  4. NOT a feature-backed optional dep — removing it would leave a dangling `dep:X`
        //     feature entry that cargo rejects (CRITICAL-1).
        //  5. NOT `optional = true` AT ALL — even with zero [features] mentions in the owning
        //     manifest, an optional dep exports an implicit cargo feature a SIBLING member can
        //     request via `features = ["x"]` on its path dep; neither the own-manifest scan nor
        //     `cargo metadata --no-deps` resolves cross-member features (MED-X1).
        let used_in_src = node
            .get("cargo_dep_used_in_src")
            .and_then(Value::as_object)
            .and_then(|map| map.get(dep))
            .and_then(Value::as_bool)
            .unwrap_or(true); // unknown ⇒ treat as used (never auto-remove a possibly-live dep)
        let is_build_dep = node
            .get("build_dep_names")
            .and_then(Value::as_array)
            .map(|names| names.iter().any(|n| n.as_str() == Some(dep)))
            .unwrap_or(false);
        // HIGH-2: any renamed dep is conservatively never auto-fixed.
        let is_renamed = node
            .get("cargo_dep_rename_keys")
            .and_then(Value::as_object)
            .map(|map| map.contains_key(dep))
            .unwrap_or(false);
        // CRITICAL-1: a dep that is optional=true and wired through [features] as dep:X must not
        // be removed without also rewriting the feature entry — that is a design act.
        let is_feature_backed = node
            .get("feature_backed_optional_deps")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().any(|v| v.as_str() == Some(dep)))
            .unwrap_or(false);
        // MED-X1 sound bound: ANY `optional = true` dep is never auto-fixable. The implicit
        // feature it exports can be requested by a sibling workspace member, invisible to both
        // guard layers (own-manifest [features] scan; `cargo metadata --no-deps`).
        let is_optional = node
            .get("optional_dep_names")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().any(|v| v.as_str() == Some(dep)))
            .unwrap_or(false);
        // ADR-0549: BUCK --fix rides the shared oya-buck-syntax-kernel sound parser + fixer
        // harness, so a mechanically dead dep is auto-fixable in BOTH manifests (the ADR-0547
        // D6 round-3 refusal-only descope is closed — FRIC-1781200001). The five Cargo sound
        // bounds gate BOTH lanes: a dep edge is removed only when it is wholly dead.
        let is_buck = code == "KP-TRANSIENT-DEP-BUCK";
        let mechanically_dead =
            !used_in_src && !is_build_dep && !is_renamed && !is_feature_backed && !is_optional;
        let auto_fixable = mechanically_dead;

        let key = format!("{kernel_name}:{node_name}:{dep}");
        // LOW-F: remediation text distinguishes the actual reason a dep is not auto-fixable so the
        // contributor sees the right next step, not a generic "used in src" for every blocked case.
        let design_reason = if is_feature_backed {
            format!(
                "`{dep}` is referenced in [features] of {member_path} — removing the dep line alone would leave a dangling feature entry (cargo error: feature includes dep:{dep} but {dep} is not listed as a dependency); remove the dep AND rewrite the [features] entry together, or move both into a sibling adapter"
            )
        } else if is_optional {
            format!(
                "`{dep}` is `optional = true` in {member_path} — an optional dep exports an implicit cargo feature named `{dep}` even when this manifest's [features] never mentions it, and a sibling workspace member can request that implicit feature via `features = [\"{dep}\"]` on its path dependency (invisible to both this gate's own-manifest [features] scan and the `cargo metadata --no-deps` revalidation, which does no cross-member feature resolution); remove the dep AND any sibling `features = [\"{dep}\"]` requests together, or move the optional wiring into a sibling adapter"
            )
        } else if is_renamed {
            format!(
                "`{dep}` is a renamed dep in {member_path} (a `package = \"{dep}\"` key alias); the rename key is what src uses — remove both the dep declaration and the rename alias together, or move them into a sibling adapter"
            )
        } else if is_build_dep {
            format!(
                "`{dep}` is a build-dependency in {member_path} — build-dep liveness is hard to attribute per-dep so auto-fix is withheld; remove it manually from [build-dependencies] if build.rs does not use it, or move build.rs logic into a sibling adapter"
            )
        } else {
            // used_in_src == true (or unknown → treated as used)
            format!(
                "`{dep}` is used in {member_path}/src — move the code that uses `{dep}` into a sibling `*-adapter` crate (e.g. {}-adapter), have the adapter depend on this kernel, and keep the kernel `{dep}`-free",
                kernel_adapter_hint(kernel_name)
            )
        };
        let (detail, next_action) = if node_name == kernel_name {
            let detail = format!(
                "kernel `{kernel_name}` depends on transient `{dep}` in {source_label}; the {dep} adapter is discarded at owned-stack cutover (ADR-0510) — the kernel must stay cutover-stable (ADR-0547)"
            );
            let action = if auto_fixable {
                let manifest = if is_buck {
                    "BUCK (rust_library deps)"
                } else {
                    "Cargo.toml"
                };
                format!(
                    "AUTO-FIXABLE: `{dep}` is declared in {member_path} but unreferenced in its src — run the gate with --fix to remove the dead dependency edge from {manifest} (BUCK edits ride the oya-buck-syntax-kernel fixer harness, ADR-0549), or delete the `{dep}` line manually"
                )
            } else {
                format!("DESIGN ACTION (not auto-applied): {design_reason}")
            };
            (detail, action)
        } else {
            let detail = format!(
                "kernel `{kernel_name}` reaches transient `{dep}` in {source_label} via {via} (node `{node_name}`); a kernel must not absorb a transient-carrying crate into its path-dep closure (ADR-0547)"
            );
            let action = if auto_fixable {
                let manifest = if is_buck {
                    "BUCK (rust_library deps)"
                } else {
                    "Cargo.toml"
                };
                format!(
                    "AUTO-FIXABLE: `{dep}` is declared in {member_path} (closure node `{node_name}`) but unreferenced in its src — run --fix to remove the dead dependency edge from {manifest}, or delete the `{dep}` line manually"
                )
            } else {
                format!(
                    "DESIGN ACTION (not auto-applied): `{node_name}` carries transient `{dep}`; the kernel `{kernel_name}` must not path-depend on a transient-carrying crate — depend on `{node_name}`'s pure kernel/port instead, or relocate `{node_name}`'s {dep} wiring into an adapter. Dep detail: {design_reason}"
                )
            };
            (detail, action)
        };
        findings.insert(Finding::new(code, &key, detail).with_action(auto_fixable, next_action));
    }
}

/// Infer a sibling adapter crate name hint from a kernel crate name (`oya-foo-kernel` ->
/// `oya-foo`, the stem an adapter would share). Pure string heuristic for the remediation hint
/// only — it never drives a code change.
fn kernel_adapter_hint(kernel_name: &str) -> String {
    kernel_name
        .strip_suffix("-kernel")
        .or_else(|| kernel_name.strip_suffix("-core"))
        .unwrap_or(kernel_name)
        .to_owned()
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

// ---------------------------------------------------------------------------
// Automation: derivable auto-fix for dead transient deps (founder directive 2026-06-11)
// ---------------------------------------------------------------------------

/// One mechanically-applicable fix: remove a dead transient dependency declared in `member_path`'s
/// Cargo.toml AND its `third-party//:<dep>` rust_library edge in the sibling BUCK file (the dep is
/// unreferenced in that crate's src, so removal moves no code). BUCK edits ride the shared
/// oya-buck-syntax-kernel sound parser + fixer harness (ADR-0549; closes FRIC-1781200001).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fix {
    pub member_path: String,
    pub dep: String,
}

/// Plan the auto-fixable subset of findings: each `auto_fixable` transient-dep finding maps to a
/// dead-dep removal at its closure node's manifest. Pure: derived from the collected graph + the
/// findings, no filesystem. De-duplicated (the same dead dep can surface under multiple kernels and
/// under both the CARGO and BUCK codes — one Fix per (member_path, dep)).
pub fn plan_fixes(policy: &Value, observed: &Value) -> Vec<Fix> {
    // Build a (kernel:node:dep) -> member_path index from the collected closure so a Fix names the
    // owning crate directory, not the kernel that reached it.
    let mut node_member: BTreeMap<(String, String), String> = BTreeMap::new();
    if let Some(crates) = observed.get("crates").and_then(Value::as_array) {
        for kernel in crates {
            let kernel_name = kernel.get("kernel").and_then(Value::as_str).unwrap_or("");
            if let Some(closure) = kernel.get("closure").and_then(Value::as_array) {
                for node in closure {
                    let node_name = node.get("name").and_then(Value::as_str).unwrap_or("");
                    let member = node
                        .get("member_path")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_owned();
                    node_member.insert((kernel_name.to_owned(), node_name.to_owned()), member);
                }
            }
        }
    }

    let mut fixes: BTreeSet<Fix> = BTreeSet::new();
    for finding in evaluate_keyed(policy, observed) {
        if !finding.auto_fixable {
            continue;
        }
        // Key shape is `<kernel>:<node>:<dep>`.
        let parts: Vec<&str> = finding.key.splitn(3, ':').collect();
        if parts.len() != 3 {
            continue;
        }
        let member = node_member
            .get(&(parts[0].to_owned(), parts[1].to_owned()))
            .cloned()
            .unwrap_or_default();
        if member.is_empty() {
            continue;
        }
        fixes.insert(Fix {
            member_path: member,
            dep: parts[2].to_owned(),
        });
    }
    fixes.into_iter().collect()
}

/// Apply the planned fixes to disk: remove the dep's declaration line from `<member>/Cargo.toml`
/// and its dead `third-party//:<dep>` rust_library edge from `<member>/BUCK` (the BUCK lane rides
/// the oya-buck-syntax-kernel sound parser + write-through fixer harness per ADR-0549, closing the
/// ADR-0547 D6 round-3 refusal-only descope).
///
/// CRITICAL-A layer 2: after ALL Cargo.toml edits are written, run `cargo metadata` (the sole
/// sanctioned cargo invocation per the teammate preamble) to semantically revalidate. If it fails,
/// ALL pre-images are restored and those findings are reclassified as design-actions in the returned
/// error so the gate reports them accurately rather than claiming a corrupt manifest is green.
pub fn apply_fixes(root: &Path, fixes: &[Fix]) -> Result<Vec<String>, CollectError> {
    apply_fixes_with_validator(root, fixes, cargo_metadata_validator)
}

/// The default semantic validator: run `cargo metadata --no-deps` at `root` (the sole sanctioned
/// cargo invocation — the gate's own validation layer). Returns `Err(stderr)` when cargo rejects
/// the edited manifests. If the `cargo` binary itself cannot be spawned (e.g. a hermetic buck2
/// sandbox without cargo on PATH), the validator degrades to `Ok(())`: the layer-1 syntactic
/// bounds have already passed and the blocking buck2 `rust_test` gate is the enforcement backstop
/// (documented in ADR-0547 D6).
///
/// "Cannot be spawned" is broader than `Command::output` returning `Err`. On PATH, `cargo` is
/// usually rustup's PROXY, not a cargo. A proxy that cannot resolve a toolchain still spawns
/// fine and then exits NON-ZERO with its own diagnostic — indistinguishable, at this match arm,
/// from cargo rejecting the manifest. Attributing that to the manifest is a false RED that also
/// rolls back correct edits, so availability is probed separately by [`cargo_is_usable`].
fn cargo_metadata_validator(root: &Path) -> Result<(), String> {
    if let Err(unavailable) = cargo_is_usable(root) {
        // Degraded mode (review F5: surface it, never degrade silently): the layer-1
        // syntactic bounds have already passed and the blocking buck2 rust_test gate is
        // the enforcement backstop (ADR-0547 D6).
        eprintln!(
            "kernel-purity --fix: WARNING — `cargo metadata` revalidation skipped ({unavailable}); \
             layer-2 semantic validation degraded, the blocking buck2 gate remains the backstop"
        );
        return Ok(());
    }
    match std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
        Err(spawn_error) => Err(format!(
            "cargo metadata could not be spawned: {spawn_error}"
        )),
    }
}

/// Is a working `cargo` reachable *from `root`*, independent of any manifest there?
///
/// `cargo --version` is the discriminator because it exercises exactly the resolution that
/// precedes `cargo metadata` — PATH lookup, then rustup's cwd-anchored toolchain resolution —
/// and touches no manifest. `root` matters: rustup resolves the toolchain from the WORKING
/// DIRECTORY, so a probe run anywhere else answers a different question than the one asked.
///
/// Measured (buck2 rust_test action, macOS, simulated CI runner env):
///   * no resolvable toolchain -> `--version` exits 1, `metadata` exits 1 (same rustup text)
///   * resolvable toolchain + broken manifest -> `--version` exits 0, `metadata` exits 101
///
/// So a real manifest error is never degraded away: the probe passes and the caller's
/// `metadata` failure is reported as-is.
fn cargo_is_usable(root: &Path) -> Result<(), String> {
    match std::process::Command::new("cargo")
        .arg("--version")
        .current_dir(root)
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(format!(
            "`cargo --version` failed with {} — no usable cargo toolchain resolves here: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )),
        Err(spawn_error) => Err(format!("cargo could not be spawned: {spawn_error}")),
    }
}

/// [`apply_fixes`] with an injectable semantic validator so the rollback path is deterministically
/// testable without depending on a `cargo` binary in the test environment. The validator is run
/// once after ALL Cargo.toml edits; `Err(reason)` triggers a full pre-image rollback.
pub fn apply_fixes_with_validator<F>(
    root: &Path,
    fixes: &[Fix],
    validator: F,
) -> Result<Vec<String>, CollectError>
where
    F: Fn(&Path) -> Result<(), String>,
{
    if fixes.is_empty() {
        return Ok(Vec::new());
    }

    // --- Phase 1: collect pre-images and perform the Cargo.toml + BUCK edits ---
    // LOW-X3: the shared kernel PreImageRegistry keys by path with FIRST pre-image wins — a
    // manifest edited twice (two dead deps in the same file) must roll back to its ORIGINAL
    // content, not the intermediate one-edit state an insertion-order restore would leave.
    let mut registry = PreImageRegistry::new();
    let mut applied = Vec::new();
    for fix in fixes {
        let cargo_path = root.join(&fix.member_path).join("Cargo.toml");
        // Save pre-image before any write.
        match fs::read_to_string(&cargo_path) {
            Ok(pre) => {
                if remove_cargo_dep_line(&cargo_path, &fix.dep)? {
                    registry.record(&cargo_path.to_string_lossy(), &pre);
                    applied.push(format!(
                        "{}/Cargo.toml: removed `{}`",
                        fix.member_path, fix.dep
                    ));
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(CollectError::Io(format!(
                    "read {}: {e}",
                    cargo_path.display()
                )));
            }
        }
        // BUCK lane (ADR-0549): remove the dead `third-party//:<dep>` rust_library edge via the
        // sound parser + write-through harness. The remover refuses (no write) on any shape it
        // cannot prove sound; a refusal leaves the finding red for the next report.
        let buck_path = root.join(&fix.member_path).join("BUCK");
        match fs::read_to_string(&buck_path) {
            Ok(pre) => {
                if remove_buck_dep_line(&buck_path, &fix.dep)? {
                    registry.record(&buck_path.to_string_lossy(), &pre);
                    applied.push(format!("{}/BUCK: removed `{}`", fix.member_path, fix.dep));
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(CollectError::Io(format!(
                    "read {}: {e}",
                    buck_path.display()
                )));
            }
        }
    }

    if applied.is_empty() {
        return Ok(applied);
    }

    // --- Phase 2: semantic revalidation with rollback (CRITICAL-A layer 2) ---
    // Validate the edited manifests once (default: `cargo metadata`). This catches any
    // feature-entry dangling references or other dependency-graph errors that the layer-1
    // syntactic bounds could not prevent.
    if let Err(cargo_error) = validator(root) {
        // Rollback ALL edits to pre-images (deterministic path order; first image per path).
        for (path, pre) in registry.images() {
            let _ = fs::write(path, pre); // best-effort; failure leaves a partial state the gate will re-report
        }
        return Err(CollectError::Io(format!(
            "cargo metadata failed after Cargo.toml/BUCK edits — ALL changes rolled back; treat \
             these findings as DESIGN ACTIONS (manual removal with coordinated [features]/workspace \
             cleanup), not mechanical line removals. cargo error: {cargo_error}"
        )));
    }

    Ok(applied)
}

fn remove_cargo_dep_line(path: &Path, dep: &str) -> Result<bool, CollectError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(CollectError::Io(format!("read {}: {e}", path.display()))),
    };
    let mut kept = Vec::new();
    let mut removed = false;
    // Track the current `[table]` header so a dep line is removed ONLY inside a real dependency
    // table — never `[features]` (would dangle `dep:<x>`), `[dev-dependencies]` (live test dep), or
    // any other section (review BLOCKER-2). `[dependencies.<x>]` section form is intentionally not
    // matched (it is a header, not a single dep line; such a dep is not auto-fixed).
    let mut in_removable_table = false;
    for line in text.lines() {
        if let Some(header) = section_header(line) {
            in_removable_table = is_removable_dep_table(&header);
            kept.push(line);
            continue;
        }
        if in_removable_table && is_dep_decl_line(line, dep) {
            removed = true;
            continue;
        }
        kept.push(line);
    }
    if removed {
        let mut out = kept.join("\n");
        if text.ends_with('\n') {
            out.push('\n');
        }
        fs::write(path, out)
            .map_err(|e| CollectError::Io(format!("write {}: {e}", path.display())))?;
    }
    Ok(removed)
}

/// The trimmed table name of a `[section]` line (`[dependencies]` -> `dependencies`), or None if the
/// line is not a section header.
fn section_header(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    Some(inner.trim().to_owned())
}

/// True iff a TOML section is a normal/build dependency table whose single-line deps may be removed.
/// `dependencies`, `build-dependencies`, and any `target.*.dependencies`/`*.build-dependencies`.
/// Explicitly excludes `dev-dependencies`, `features`, `dependencies.<x>` (sub-section), etc.
fn is_removable_dep_table(name: &str) -> bool {
    if name == "dependencies" || name == "build-dependencies" {
        return true;
    }
    if let Some(rest) = name.strip_prefix("target.") {
        return rest.ends_with(".dependencies") || rest.ends_with(".build-dependencies");
    }
    false
}

/// True iff `line` is the dependency declaration for `dep` (the dep key at the start of the line,
/// followed by `=`). Anchored so `kube = ...` is removed but `kube-runtime = ...` is not when
/// `dep == "kube"`. The dotted `kube.workspace = true` form is NOT matched (it is not auto-fixable;
/// the residual stays red, which is the safe direction).
fn is_dep_decl_line(line: &str, dep: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix(dep) else {
        return false;
    };
    let rest = rest.trim_start();
    rest.starts_with('=')
}

/// Remove the `third-party//:<dep>` edge from every `rust_library` deps list in the BUCK file,
/// via the shared sound parser + write-through harness (ADR-0549; closes FRIC-1781200001 and
/// re-enables the BUCK `--fix` lane that ADR-0547 D6 round-3 had descoped to refusal-only).
///
/// Sound bounds — the remover REFUSES (returns Ok(false), file byte-identical) when:
/// - the BUCK text does not parse soundly (unterminated block, unbalanced delimiters — H6);
/// - the dep edge is not a plain string element of a `deps = [...]` list literal (a var
///   reference, `select(...)`, or any unmodeled shape);
/// - the post-edit harness validation fails (reparse, dep gone from every rust_library,
///   NO collateral dep removed, top-level statement count unchanged).
///
/// `rust_test`/`rust_binary` deps are never touched (test edges are out of detect scope).
fn remove_buck_dep_line(path: &Path, dep: &str) -> Result<bool, CollectError> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(CollectError::Io(format!("read {}: {e}", path.display()))),
    };
    let Some(candidate) = remove_buck_dep_edges_text(&text, dep) else {
        return Ok(false);
    };
    // Write-through harness: reparse + semantic hook; refusal keeps the file byte-identical.
    let mut registry = PreImageRegistry::new();
    let path_key = path.to_string_lossy().to_string();
    let label = format!("third-party//:{dep}");
    // Every OTHER rust_library third-party dep present before the edit must survive it.
    let pre_others: BTreeSet<String> = {
        let mut all = extract_buck_library_thirdparty_deps(&text);
        all.remove(dep);
        all
    };
    let guarded = guarded_rewrite(&path_key, &text, &candidate, &mut registry, |_, out| {
        let after = extract_buck_library_thirdparty_deps(out);
        if after.contains(dep) {
            return Err(format!("`{label}` still present after removal"));
        }
        if !pre_others.is_subset(&after) {
            return Err("collateral dep removal detected".to_owned());
        }
        Ok(())
    });
    match guarded {
        Ok(validated) => {
            fs::write(path, validated)
                .map_err(|e| CollectError::Io(format!("write {}: {e}", path.display())))?;
            Ok(true)
        }
        Err(_refusal) => Ok(false),
    }
}

/// Pure rewrite: remove every plain-string `third-party//:<dep>` element from `rust_library`
/// deps list literals. Returns `None` (refuse) when the dep edge is absent or any carrying
/// shape is not a modeled list element. Re-parses between removals so spans stay exact.
fn remove_buck_dep_edges_text(text: &str, dep: &str) -> Option<String> {
    let label = format!("third-party//:{dep}");
    let mut current = text.to_owned();
    let mut removed_any = false;
    loop {
        let doc = oya_buck_syntax_kernel::parse(&current).ok()?;
        let mut found: Option<String> = None;
        for stmt in &doc.stmts {
            let Stmt::Call(call) = stmt else { continue };
            if call.func != "rust_library" {
                continue;
            }
            let Some(deps_arg) = call.kwarg("deps") else {
                continue;
            };
            match &deps_arg.value.expr {
                Expr::List(list) => {
                    let index = list.elements.iter().position(
                        |element| matches!(&element.value.expr, Expr::Str(s) if s == &label),
                    );
                    if let Some(index) = index {
                        found = Some(remove_list_element(&current, list, index).ok()?);
                        break;
                    }
                    // The dep may hide in an unmodeled element shape: refuse if the label
                    // appears in the list's raw span (token-boundary exact, so a SIBLING dep
                    // that merely shares the prefix — `kube` vs `kube-runtime` — does not
                    // false-refuse) without being a plain element.
                    if contains_dep_token(deps_arg.value.span.slice(&current), &label) {
                        return None;
                    }
                }
                // deps is not a list literal (a var, select(), concat): refuse if it carries
                // the label anywhere in its raw span (token-boundary exact).
                _ => {
                    if contains_dep_token(deps_arg.value.span.slice(&current), &label) {
                        return None;
                    }
                }
            }
        }
        match found {
            Some(next) => {
                current = next;
                removed_any = true;
            }
            None => break,
        }
    }
    if removed_any { Some(current) } else { None }
}

/// True iff `label` occurs in `text` as an EXACT dep token: the next character (if any) is not
/// part of a dep name (`[A-Za-z0-9_-]`). `third-party//:kube` must not match inside
/// `third-party//:kube-runtime` — substring refusal would wrongly block a sound removal of
/// `kube` whenever a longer-named sibling is present.
fn contains_dep_token(text: &str, label: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(label) {
        let end = from + rel + label.len();
        let boundary = text[end..]
            .chars()
            .next()
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        if boundary {
            return true;
        }
        from = end;
    }
    false
}

/// Human-readable render of the findings, automation-default ordered: the auto-fixable subset first
/// (with the one-command fix), then the design-action subset (each with its best next action). Never
/// a bare FAIL — every finding prints its `next_action`.
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "kernel-purity gate passed: all kernel/core crates are transient-dep-free"
            .to_owned();
    }
    let auto: Vec<&Finding> = findings.iter().filter(|f| f.auto_fixable).collect();
    let manual: Vec<&Finding> = findings.iter().filter(|f| !f.auto_fixable).collect();
    let mut out = String::from("kernel-purity gate failed:\n");
    if !auto.is_empty() {
        out.push_str(&format!(
            "\n  AUTO-FIXABLE ({}): run this gate's binary with --fix to apply, then commit the diff:\n",
            auto.len()
        ));
        for finding in &auto {
            out.push_str(&format!(
                "    - {} {}\n        {}\n",
                finding.code, finding.key, finding.next_action
            ));
        }
    }
    if !manual.is_empty() {
        out.push_str(&format!(
            "\n  DESIGN ACTIONS ({}, not auto-applied — moving live code is a design decision):\n",
            manual.len()
        ));
        for finding in &manual {
            out.push_str(&format!(
                "    - {} {}\n        {}\n        {}\n",
                finding.code, finding.key, finding.detail, finding.next_action
            ));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Policy / glob helpers
// ---------------------------------------------------------------------------

fn kernel_globs(policy: &Value) -> Vec<String> {
    policy
        .get("kernel_crate_globs")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Match a crate name against a simple suffix/prefix glob (`*-kernel`, `*-core`, `oya-*`). Only a
/// single leading or trailing `*` is supported — that is all the kernel-name conventions need.
pub fn name_matches_glob(name: &str, glob: &str) -> bool {
    match (glob.strip_prefix('*'), glob.strip_suffix('*')) {
        (Some(suffix), _) if glob.starts_with('*') => name.ends_with(suffix),
        (_, Some(prefix)) if glob.ends_with('*') => name.starts_with(prefix),
        _ => name == glob,
    }
}

fn name_matches_any_glob(name: &str, globs: &[String]) -> bool {
    globs.iter().any(|glob| name_matches_glob(name, glob))
}

fn normalize_dir(dir: &str) -> String {
    let trimmed = dir.trim_end_matches('/');
    let mut parts: Vec<&str> = Vec::new();
    for segment in trimmed.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// Resolve a `path = "..."` relative to the depending crate's member directory, normalized.
fn join_relative(member_dir: &str, rel: &str) -> String {
    if rel.starts_with('/') {
        return normalize_dir(rel);
    }
    let combined = format!("{}/{}", member_dir.trim_end_matches('/'), rel);
    normalize_dir(&combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The availability probe must be MANIFEST-INDEPENDENT, and degrading must be reserved for
    /// an unusable toolchain — never for a manifest cargo actually rejected.
    ///
    /// Asserted as an implication rather than a fixed outcome, because the answer legitimately
    /// differs by environment: a buck2 `rust_test` action receives an 8-variable env whitelist
    /// with no `RUSTUP_HOME`/`CARGO_HOME`, so on a CI image whose rustup state lives outside
    /// `$HOME` no toolchain resolves and BOTH calls must degrade. Pinning either branch
    /// absolutely would make this test a host detector.
    #[test]
    fn availability_probe_ignores_the_manifest_and_gates_the_degrade() {
        let root = std::env::temp_dir().join(format!(
            "kernel-purity-probe-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&root).expect("create probe root");
        // Syntactically invalid TOML: `cargo metadata` must reject this whenever it can run.
        fs::write(root.join("Cargo.toml"), "[package]\nname = \n").expect("write bad manifest");

        let usable = cargo_is_usable(&root);
        let validated = cargo_metadata_validator(&root);

        if usable.is_ok() {
            assert!(
                validated.is_err(),
                "a usable cargo must still REJECT a broken manifest — degrading here would \
                 silently roll back correct fixes and report a green gate: {validated:?}"
            );
        } else {
            assert_eq!(
                validated,
                Ok(()),
                "an unusable cargo must degrade to the ADR-0547 D6 backstop, not be reported \
                 as a manifest defect: probe said {usable:?}"
            );
        }

        fs::remove_dir_all(&root).expect("remove probe root");
    }

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "kernel_crate_globs": ["*-kernel", "*-core"],
            "min_expected_kernel_crates": 1,
            "deny": [
                {"dep": "kube", "match": "exact"},
                {"dep": "kube-", "match": "prefix"},
                {"dep": "k8s-openapi", "match": "exact"},
                {"dep": "sqlx", "match": "exact"},
                {"dep": "aws-sdk-", "match": "prefix"},
                {"dep": "etcd-", "match": "prefix"}
            ],
            "exceptions": []
        })
    }

    fn kernel(name: &str, cargo: &[&str], buck: &[&str]) -> Value {
        json!({
            "kernel": name,
            "member_path": format!("crates/{name}"),
            "closure": [{
                "name": name,
                "member_path": format!("crates/{name}"),
                "via": [name],
                "cargo_deps": cargo,
                "buck_deps": buck,
            }]
        })
    }

    fn observed(kernels: Vec<Value>) -> Value {
        json!({ "kernel_crates_found": kernels.len(), "crates": kernels })
    }

    /// A kernel whose cargo deps carry an explicit per-dep used-in-src map (for the automation
    /// classification tests). `used` lists the deps referenced in src; all others are dead.
    fn kernel_with_usage(name: &str, cargo: &[&str], used: &[&str]) -> Value {
        let used_map: serde_json::Map<String, Value> = cargo
            .iter()
            .map(|dep| ((*dep).to_owned(), Value::from(used.contains(dep))))
            .collect();
        json!({
            "kernel": name,
            "member_path": format!("crates/{name}"),
            "closure": [{
                "name": name,
                "member_path": format!("crates/{name}"),
                "via": [name],
                "cargo_deps": cargo,
                "buck_deps": [],
                "cargo_dep_used_in_src": used_map,
            }]
        })
    }

    #[test]
    fn dead_transient_dep_is_auto_fixable() {
        // kube declared but NOT used in src -> auto-fixable.
        let obs = observed(vec![kernel_with_usage(
            "oya-bad-kernel",
            &["serde", "kube"],
            &["serde"],
        )]);
        let findings = evaluate_keyed(&policy(), &obs);
        let kube = findings
            .iter()
            .find(|f| f.code == "KP-TRANSIENT-DEP-CARGO" && f.key.ends_with(":kube"))
            .expect("kube finding");
        assert!(kube.auto_fixable, "dead dep must be auto-fixable: {kube:?}");
        assert!(kube.next_action.contains("AUTO-FIXABLE"));
        let fixes = plan_fixes(&policy(), &obs);
        assert_eq!(fixes.len(), 1);
        assert_eq!(fixes[0].dep, "kube");
        assert_eq!(fixes[0].member_path, "crates/oya-bad-kernel");
    }

    #[test]
    fn used_transient_dep_is_a_design_action_not_auto_fixable() {
        // kube IS used in src -> NOT auto-fixable; the action names moving code to an adapter.
        let obs = observed(vec![kernel_with_usage(
            "oya-bad-kernel",
            &["kube"],
            &["kube"],
        )]);
        let findings = evaluate_keyed(&policy(), &obs);
        let kube = findings
            .iter()
            .find(|f| f.key.ends_with(":kube"))
            .expect("kube finding");
        assert!(
            !kube.auto_fixable,
            "used dep must not be auto-fixable: {kube:?}"
        );
        assert!(kube.next_action.contains("DESIGN ACTION"));
        assert!(
            kube.next_action.contains("oya-bad-adapter"),
            "the design action should hint the sibling adapter name: {}",
            kube.next_action
        );
        // plan_fixes must NOT include a design-action finding.
        assert!(plan_fixes(&policy(), &obs).is_empty());
    }

    #[test]
    fn unknown_usage_defaults_to_not_auto_fixable() {
        // The plain kernel() helper omits cargo_dep_used_in_src -> default used=true -> NOT
        // auto-fixable (never auto-remove a possibly-live dep).
        let obs = observed(vec![kernel("oya-bad-kernel", &["kube"], &[])]);
        let kube = evaluate_keyed(&policy(), &obs)
            .into_iter()
            .find(|f| f.key.ends_with(":kube"))
            .expect("kube finding");
        assert!(!kube.auto_fixable);
        assert!(plan_fixes(&policy(), &obs).is_empty());
    }

    #[test]
    fn render_findings_separates_auto_and_design_and_is_never_bare() {
        let obs = observed(vec![
            kernel_with_usage("oya-dead-kernel", &["kube"], &[]),
            kernel_with_usage("oya-live-kernel", &["sqlx"], &["sqlx"]),
        ]);
        let rendered = render_findings(&evaluate_keyed(&policy(), &obs));
        assert!(rendered.contains("AUTO-FIXABLE"));
        assert!(rendered.contains("DESIGN ACTIONS"));
        assert!(rendered.contains("--fix"));
        // green case
        let green = render_findings(&BTreeSet::new());
        assert!(green.contains("passed"));
    }

    #[test]
    fn dep_decl_line_matcher_anchors_dep_name() {
        assert!(is_dep_decl_line("kube = \"0.99\"", "kube"));
        assert!(is_dep_decl_line("  kube = { workspace = true }", "kube"));
        assert!(!is_dep_decl_line("kube-runtime = \"0.99\"", "kube"));
        assert!(!is_dep_decl_line("# kube = \"0.99\"", "kube"));
    }

    #[test]
    fn build_dependency_is_never_auto_fixable() {
        // BLOCKER-1: a transient dep declared in a build-deps table must NOT be auto-fixable even if
        // unreferenced in src — build.rs is its legitimate usage site and per-dep liveness is hard.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-bad-kernel",
                "member_path": "crates/oya-bad-kernel",
                "closure": [{
                    "name": "oya-bad-kernel",
                    "member_path": "crates/oya-bad-kernel",
                    "via": ["oya-bad-kernel"],
                    "cargo_deps": ["k8s-openapi"],
                    "buck_deps": [],
                    "cargo_dep_used_in_src": {"k8s-openapi": false},
                    "build_dep_names": ["k8s-openapi"]
                }]
            }]
        });
        let findings = evaluate_keyed(&policy(), &obs);
        let f = findings
            .iter()
            .find(|f| f.key.ends_with(":k8s-openapi"))
            .expect("finding");
        assert!(!f.auto_fixable, "build-dep must not be auto-fixable: {f:?}");
        assert!(plan_fixes(&policy(), &obs).is_empty());
    }

    #[test]
    fn removable_dep_table_classification() {
        assert!(is_removable_dep_table("dependencies"));
        assert!(is_removable_dep_table("build-dependencies"));
        assert!(is_removable_dep_table("target.'cfg(unix)'.dependencies"));
        assert!(is_removable_dep_table(
            "target.'cfg(unix)'.build-dependencies"
        ));
        assert!(!is_removable_dep_table("dev-dependencies"));
        assert!(!is_removable_dep_table("features"));
        assert!(!is_removable_dep_table("dependencies.kube")); // sub-section header
        assert!(!is_removable_dep_table("package"));
    }

    #[test]
    fn section_header_parses_table_names() {
        assert_eq!(
            section_header("[dependencies]").as_deref(),
            Some("dependencies")
        );
        assert_eq!(
            section_header("  [dev-dependencies]  ").as_deref(),
            Some("dev-dependencies")
        );
        assert_eq!(section_header("kube = \"1\""), None);
    }

    #[test]
    fn unresolved_path_dep_fails_closed() {
        // A kernel closure node that path-depends on a crate outside the workspace (collector could
        // not resolve it to a member) must fail closed — the unscanned subtree is not a free pass.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-foo-kernel",
                "member_path": "crates/oya-foo-kernel",
                "closure": [{
                    "name": "oya-foo-kernel",
                    "member_path": "crates/oya-foo-kernel",
                    "via": ["oya-foo-kernel"],
                    "cargo_deps": ["serde"],
                    "buck_deps": [],
                    "cargo_dep_used_in_src": {"serde": true},
                    "unresolved_path_deps": ["vendor/out-of-workspace-crate"]
                }]
            }]
        });
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings.iter().any(|f| {
                f.code == "KP-UNRESOLVED-PATH-DEP"
                    && f.key == "oya-foo-kernel:oya-foo-kernel:vendor/out-of-workspace-crate"
            }),
            "unresolved path dep must be RED: {findings:#?}"
        );
        assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Red);
    }

    #[test]
    fn serde_only_kernel_is_green() {
        let obs = observed(vec![kernel("oya-foo-kernel", &["serde"], &["serde"])]);
        let report = evaluate(&policy(), &obs);
        assert_eq!(report.verdict, Verdict::Green, "{report:?}");
        assert!(evaluate_keyed(&policy(), &obs).is_empty());
    }

    #[test]
    fn primitive_dep_kernel_is_green_no_false_positive() {
        // aws-lc-rs / libc / zeroize / tokio are cutover-stable primitives, NOT denied.
        let obs = observed(vec![kernel(
            "oya-cloud-kms-enclave-kernel",
            &["aws-lc-rs", "libc", "zeroize", "tokio"],
            &["aws-lc-rs", "libc", "zeroize", "tokio"],
        )]);
        assert!(
            evaluate_keyed(&policy(), &obs).is_empty(),
            "primitives must not false-positive"
        );
    }

    #[test]
    fn kube_dep_in_cargo_fails_closed() {
        let obs = observed(vec![kernel(
            "oya-bad-kernel",
            &["serde", "kube"],
            &["serde"],
        )]);
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(findings.iter().any(|f| {
            f.code == "KP-TRANSIENT-DEP-CARGO" && f.key == "oya-bad-kernel:oya-bad-kernel:kube"
        }));
        assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Red);
    }

    #[test]
    fn kube_runtime_matches_hyphen_prefix() {
        let obs = observed(vec![kernel("oya-bad-kernel", &["kube-runtime"], &[])]);
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "KP-TRANSIENT-DEP-CARGO" && f.key.ends_with(":kube-runtime"))
        );
    }

    #[test]
    fn kuberos_is_not_denied_by_kube_prefix() {
        // The kube/kuberos trap: `kube` exact + `kube-` hyphen-prefix must NOT match `kuberos`.
        let obs = observed(vec![kernel(
            "oya-cloud-kernel-frame-kernel",
            &["kuberos"],
            &[],
        )]);
        assert!(
            evaluate_keyed(&policy(), &obs).is_empty(),
            "kuberos must not be matched by the kube rules"
        );
    }

    #[test]
    fn aws_lc_rs_is_not_denied_by_aws_sdk_prefix() {
        let obs = observed(vec![kernel("oya-crypto-kernel", &["aws-lc-rs"], &[])]);
        assert!(
            evaluate_keyed(&policy(), &obs).is_empty(),
            "aws-lc-rs must not be matched by the aws-sdk- prefix"
        );
    }

    #[test]
    fn aws_sdk_prefix_hit_fails_closed() {
        let obs = observed(vec![kernel("oya-bad-kernel", &["aws-sdk-s3"], &[])]);
        assert!(
            evaluate_keyed(&policy(), &obs)
                .iter()
                .any(|f| f.key.ends_with(":aws-sdk-s3"))
        );
    }

    #[test]
    fn sqlx_only_in_buck_fails_closed_with_buck_code() {
        let obs = observed(vec![kernel(
            "oya-bad-kernel",
            &["serde"],
            &["serde", "sqlx"],
        )]);
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "KP-TRANSIENT-DEP-BUCK" && f.key.ends_with(":sqlx") })
        );
        assert!(
            !findings.iter().any(|f| f.code == "KP-TRANSIENT-DEP-CARGO"),
            "sqlx is only in BUCK here, so no CARGO finding"
        );
    }

    #[test]
    fn closure_leak_through_local_adapter_fails_closed() {
        // The escape the critic falsified direct-only against: a kernel that path-depends on a
        // local adapter which carries sqlx must be RED, keyed via the closure chain.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-foo-kernel",
                "member_path": "crates/oya-foo-kernel",
                "closure": [
                    {
                        "name": "oya-foo-kernel",
                        "member_path": "crates/oya-foo-kernel",
                        "via": ["oya-foo-kernel"],
                        "cargo_deps": ["serde"],
                        "buck_deps": ["serde"]
                    },
                    {
                        "name": "oya-data-sql-adapter-sqlx",
                        "member_path": "crates/oya-data-sql-adapter-sqlx",
                        "via": ["oya-foo-kernel", "oya-data-sql-adapter-sqlx"],
                        "cargo_deps": ["sqlx"],
                        "buck_deps": ["sqlx"]
                    }
                ]
            }]
        });
        let findings = evaluate_keyed(&policy(), &obs);
        assert!(
            findings.iter().any(|f| {
                f.code == "KP-TRANSIENT-DEP-CARGO"
                    && f.key == "oya-foo-kernel:oya-data-sql-adapter-sqlx:sqlx"
                    && f.detail
                        .contains("oya-foo-kernel -> oya-data-sql-adapter-sqlx")
            }),
            "closure leak must be RED keyed via the chain: {findings:#?}"
        );
        assert_eq!(evaluate(&policy(), &obs).verdict, Verdict::Red);
    }

    #[test]
    fn exception_suppresses_exactly_its_finding() {
        let mut p = policy();
        p["exceptions"] = json!([
            {"crate": "oya-bad-kernel", "dep": "kube", "reason": "transitional", "adr": "ADR-0510"}
        ]);
        let obs = observed(vec![kernel("oya-bad-kernel", &["kube"], &[])]);
        assert!(
            evaluate_keyed(&p, &obs).is_empty(),
            "the exact (crate,dep) exception must suppress the finding"
        );
    }

    #[test]
    fn stale_exception_fails_closed() {
        let mut p = policy();
        p["exceptions"] = json!([
            {"crate": "oya-bad-kernel", "dep": "kube", "reason": "x", "adr": "ADR-0510"}
        ]);
        // Kernel is clean — the exception matches nothing.
        let obs = observed(vec![kernel("oya-bad-kernel", &["serde"], &[])]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings
                .iter()
                .any(|f| { f.code == "KP-STALE-EXCEPTION" && f.key == "oya-bad-kernel:kube" })
        );
    }

    #[test]
    fn empty_scan_below_floor_fails_closed() {
        let mut p = policy();
        p["min_expected_kernel_crates"] = json!(100);
        let obs = observed(vec![kernel("oya-foo-kernel", &["serde"], &[])]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(findings.iter().any(|f| f.code == "KP-EMPTY-SCAN"));
    }

    #[test]
    fn gate_id_mismatch_fails_closed() {
        let mut p = policy();
        p["gate_id"] = json!("cloud-ci-wrong");
        let obs = observed(vec![kernel("oya-foo-kernel", &["serde"], &[])]);
        assert!(
            evaluate_keyed(&p, &obs)
                .iter()
                .any(|f| f.code == "KP-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let obs = observed(vec![
            kernel("oya-a-kernel", &["kube"], &[]),
            kernel("oya-b-core", &["sqlx"], &["sqlx"]),
        ]);
        let projected: BTreeSet<String> = evaluate_keyed(&policy(), &obs)
            .into_iter()
            .map(|finding| finding.code)
            .collect();
        assert_eq!(evaluate(&policy(), &obs).violations, projected);
    }

    #[test]
    fn core_glob_arm_is_matched() {
        let obs = observed(vec![kernel("oya-foo-core", &["kube"], &[])]);
        assert!(
            evaluate_keyed(&policy(), &obs)
                .iter()
                .any(|f| f.code == "KP-TRANSIENT-DEP-CARGO"),
            "the *-core glob arm must be evaluated"
        );
    }

    #[test]
    fn name_glob_matches_suffix_and_exact() {
        assert!(name_matches_glob("oya-foo-kernel", "*-kernel"));
        assert!(name_matches_glob("oya-foo-core", "*-core"));
        assert!(!name_matches_glob("oya-foo-adapter", "*-kernel"));
        assert!(name_matches_glob("oya-x", "oya-*"));
        assert!(name_matches_glob("exact", "exact"));
    }

    #[test]
    fn buck_extractor_reads_only_rust_library_thirdparty_deps() {
        let buck = r#"
rust_library(
    name = "oya-x",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        "third-party//:serde",
        "third-party//:sqlx",
        "//libs/oya-foo:oya-foo",
    ],
)

rust_test(
    name = "oya-x-unittest",
    deps = [
        "third-party//:kube",
    ],
)
"#;
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(deps.contains("serde"));
        assert!(deps.contains("sqlx"));
        assert!(
            !deps.contains("kube"),
            "rust_test deps must NOT be parsed; got {deps:?}"
        );
    }

    #[test]
    fn violation_codes_const_covers_every_emitted_code() {
        let declared: BTreeSet<&str> = VIOLATION_CODES.into_iter().collect();
        let mut p = policy();
        p["gate_id"] = json!("cloud-ci-wrong");
        p["min_expected_kernel_crates"] = json!(100);
        p["exceptions"] = json!([{"crate": "nobody", "dep": "kube", "reason": "x", "adr": "y"}]);
        // A kernel that trips transient-cargo, transient-buck, and unresolved-path-dep at once,
        // plus the policy-level gate-id-mismatch, empty-scan, and stale-exception codes.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-bad-kernel",
                "member_path": "crates/oya-bad-kernel",
                "closure": [{
                    "name": "oya-bad-kernel",
                    "member_path": "crates/oya-bad-kernel",
                    "via": ["oya-bad-kernel"],
                    "cargo_deps": ["kube"],
                    "buck_deps": ["sqlx"],
                    "cargo_dep_used_in_src": {"kube": true},
                    "unresolved_path_deps": ["vendor/elsewhere"]
                }]
            }]
        });
        let findings = evaluate_keyed(&p, &obs);
        let emitted: BTreeSet<&str> = findings.iter().map(|f| f.code.as_str()).collect();
        for code in &emitted {
            assert!(declared.contains(code), "undeclared code {code}");
        }
        assert!(
            emitted.len() >= 6,
            "expected broad coverage, got {emitted:?}"
        );

        // KP-POLICY-MALFORMED is emitted on a bad deny rule.
        let mut bad_policy = policy();
        bad_policy["deny"] = json!([{"dep": "kube", "match": "typo"}]);
        let mf = evaluate_keyed(&bad_policy, &obs);
        assert!(
            mf.iter().any(|f| f.code == "KP-POLICY-MALFORMED"),
            "malformed policy must emit KP-POLICY-MALFORMED: {mf:?}"
        );
    }

    // --------------------------------------------------------------------------
    // CRITICAL-1: optional + feature-backed dep is not auto-fixable
    // --------------------------------------------------------------------------

    #[test]
    fn optional_feature_backed_dep_is_not_auto_fixable() {
        // A dep that is `optional = true` and wired via `dep:X` in [features] must NOT be
        // auto-fixable — removing it would leave a dangling `dep:kube` feature entry that cargo
        // rejects. CRITICAL-1 regression.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-bad-kernel",
                "member_path": "crates/oya-bad-kernel",
                "closure": [{
                    "name": "oya-bad-kernel",
                    "member_path": "crates/oya-bad-kernel",
                    "via": ["oya-bad-kernel"],
                    "cargo_deps": ["kube"],
                    "buck_deps": [],
                    "cargo_dep_used_in_src": {"kube": false},
                    "build_dep_names": [],
                    "cargo_dep_rename_keys": {},
                    "feature_backed_optional_deps": ["kube"]
                }]
            }]
        });
        let findings = evaluate_keyed(&policy(), &obs);
        let f = findings
            .iter()
            .find(|f| f.key.ends_with(":kube"))
            .expect("kube finding");
        assert!(
            !f.auto_fixable,
            "feature-backed optional dep must NOT be auto-fixable: {f:?}"
        );
        assert!(
            f.next_action.contains("DESIGN ACTION"),
            "must be a design action: {f:?}"
        );
        assert!(
            plan_fixes(&policy(), &obs).is_empty(),
            "plan_fixes must produce nothing for feature-backed dep"
        );
    }

    // --------------------------------------------------------------------------
    // MED-X1: ANY optional dep is never auto-fixable (implicit-feature export)
    // --------------------------------------------------------------------------

    #[test]
    fn optional_dep_without_own_features_mention_is_not_auto_fixable() {
        // MED-X1 (reviewer-reproduced vector): an `optional = true` dep exports an IMPLICIT cargo
        // feature named after itself even when its OWN manifest's [features] never mentions it.
        // A sibling workspace member can request that feature (`features = ["kube"]`) on its path
        // dep; the own-manifest [features] scan (layer 1) cannot see the sibling, and
        // `cargo metadata --no-deps` (layer 2) does not do cross-member feature resolution. The
        // only sound bound: optional deps are NEVER auto-fixable.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-bad-kernel",
                "member_path": "crates/oya-bad-kernel",
                "closure": [{
                    "name": "oya-bad-kernel",
                    "member_path": "crates/oya-bad-kernel",
                    "via": ["oya-bad-kernel"],
                    "cargo_deps": ["kube"],
                    "buck_deps": [],
                    "cargo_dep_used_in_src": {"kube": false},
                    "build_dep_names": [],
                    "cargo_dep_rename_keys": {},
                    "feature_backed_optional_deps": [],
                    "optional_dep_names": ["kube"]
                }]
            }]
        });
        let findings = evaluate_keyed(&policy(), &obs);
        let f = findings
            .iter()
            .find(|f| f.key.ends_with(":kube"))
            .expect("kube finding");
        assert!(
            !f.auto_fixable,
            "an optional dep must NEVER be auto-fixable even with zero own-manifest [features] mentions: {f:?}"
        );
        assert!(
            f.next_action.contains("DESIGN ACTION"),
            "must be a design action: {f:?}"
        );
        assert!(
            f.next_action.contains("implicit"),
            "remediation must explain the implicit-feature export: {f:?}"
        );
        assert!(
            plan_fixes(&policy(), &obs).is_empty(),
            "plan_fixes must schedule nothing for an optional dep"
        );
    }

    // --------------------------------------------------------------------------
    // LOW-X2: backslash escapes inside Starlark strings
    // --------------------------------------------------------------------------

    #[test]
    fn backslash_escaped_quote_in_string_does_not_hide_following_dep() {
        // LOW-X2: `labels = ["weird\")label"]` — the `\"` is an ESCAPED quote INSIDE the string.
        // An escape-blind scanner ends string state at the `\"`, leaks the following `)` as live
        // text, terminates the block span early, and hides the dep below from the detect lane.
        // The shared kernel lexes escapes exactly, so the dep stays visible.
        let buck = concat!(
            "rust_library(\n",
            "    name = \"x\",\n",
            "    labels = [\"weird\\\")label\"],\n",
            "    deps = [\n",
            "        \"third-party//:kube\",\n",
            "    ],\n",
            ")\n",
        );
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("kube"),
            "the dep after a backslash-escaped quote must still be detected: {deps:?}"
        );
    }

    // --------------------------------------------------------------------------
    // FRIC-1781230000 (#693 LOW-2): backslash-newline continuation must not hide a dep
    // --------------------------------------------------------------------------

    #[test]
    fn backslash_newline_continuation_does_not_hide_dep() {
        // RED fixture for the pre-kernel gap: a Starlark string may continue across a
        // backslash-newline, so `"third-party//:k\` + newline + `ube"` is the SINGLE cooked
        // value `third-party//:kube`. The line-bounded stripper of the pre-kernel scanner reset
        // string state at the newline and saw only the truncated token `k` — the dep `kube` was
        // invisible to the detect lane. The shared kernel cooks the continuation, closing the gap.
        let buck =
            "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:k\\\nube\"],\n)\n";
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("kube"),
            "a continuation-split dep must be detected as its JOINED value: {deps:?}"
        );
        assert!(
            !deps.contains("k"),
            "the truncated pre-continuation token must not be reported: {deps:?}"
        );
    }

    #[test]
    fn wrapped_rust_library_calls_cannot_hide_deps() {
        // Reviewer BLOCKER closure: buck2-valid wrappers that take the target call out of
        // statement position must not hide its deps from the detect lane.
        // (a) assignment-wrapped: X = rust_library(...)
        let assigned =
            "X = rust_library(\n    name = \"x\",\n    deps = [\"third-party//:kube\"],\n)\n";
        assert!(
            extract_buck_library_thirdparty_deps(assigned).contains("kube"),
            "assignment-wrapped target must be detected"
        );
        // (b) expression-statement wrapped: [rust_library(...)] — an opaque statement whose
        // span mentions rust_library( is raw-scanned (over-approximation).
        let listed =
            "[rust_library(\n    name = \"x\",\n    deps = [\"third-party//:kube\"],\n)]\n";
        assert!(
            extract_buck_library_thirdparty_deps(listed).contains("kube"),
            "expression-statement-wrapped target must be detected"
        );
        // (c) trailing-ternary tail: X = 1 if c else rust_library(...) — the statement demotes
        // to opaque in the kernel and the raw scan sees the call.
        let ternary = "X = 1 if c else rust_library(\n    name = \"x\",\n    deps = [\"third-party//:kube\"],\n)\n";
        assert!(
            extract_buck_library_thirdparty_deps(ternary).contains("kube"),
            "ternary-tail target must be detected"
        );
    }

    #[test]
    fn expression_level_opaque_wrappers_cannot_hide_deps() {
        // Re-review BLOCKER closure: expression-level Opaque content inside a still-modeled
        // Assign must be raw-scanned — visit_calls cannot reach a call the widen arm discarded.
        // (a) postfix-index wrapper: the parsed list+call is discarded into one Expr::Opaque.
        let indexed = "X = [rust_library(name = \"x\", deps = [\"third-party//:kube\"])][0]\n";
        assert!(
            extract_buck_library_thirdparty_deps(indexed).contains("kube"),
            "postfix-index wrapper must be detected"
        );
        // (b) unmodeled-primary ternary: `-1` consumes the whole line as expression-level Opaque,
        // so the trailing-token demotion never fires.
        let neg_ternary =
            "X = -1 if c else rust_library(name = \"x\", deps = [\"third-party//:kube\"])\n";
        assert!(
            extract_buck_library_thirdparty_deps(neg_ternary).contains("kube"),
            "unmodeled-primary ternary must be detected"
        );
        // (c) discarded comprehension iter: the non-ident iter is dropped with no node
        // (comp.iter == None => has_opaque).
        let comp_iter = "M = {k: \"v\" for k in [rust_library(name = \"x\", deps = [\"third-party//:kube\"])]}\n";
        assert!(
            extract_buck_library_thirdparty_deps(comp_iter).contains("kube"),
            "discarded comprehension iter must be detected"
        );
        // (d) opaque argument of a DIFFERENT call at statement position: the wrapping call's
        // name must not exempt its opaque content from the raw scan.
        let wrapped_arg =
            "helper([rust_library(name = \"x\", deps = [\"third-party//:kube\"])][0])\n";
        assert!(
            extract_buck_library_thirdparty_deps(wrapped_arg).contains("kube"),
            "opaque argument of a non-rust_library call must be detected"
        );
    }

    #[test]
    fn escape_spelled_dep_is_detected_under_its_cooked_name() {
        // Review F1 (HIGH, RED fixture): buck2 evaluates `"third-party//:k\x75be"` to
        // `third-party//:kube` (proven via buck2 uquery). The detect lane must key the dep as
        // `kube` — an escape spelling must not hide a denylisted transient dep.
        let hex = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:k\\x75be\"],\n)\n";
        let deps = extract_buck_library_thirdparty_deps(hex);
        assert!(
            deps.contains("kube"),
            "\\x75 spelling must cook to kube: {deps:?}"
        );
        assert!(
            !deps.contains("k"),
            "the truncated raw token must not be reported: {deps:?}"
        );
        let octal =
            "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:k\\165be\"],\n)\n";
        assert!(
            extract_buck_library_thirdparty_deps(octal).contains("kube"),
            "octal spelling must cook to kube"
        );
        let uni =
            "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:k\\u0075be\"],\n)\n";
        assert!(
            extract_buck_library_thirdparty_deps(uni).contains("kube"),
            "\\u spelling must cook to kube"
        );
    }

    #[test]
    fn unimplemented_escape_fails_closed_to_raw_scan() {
        // An escape class the lexer refuses (hard LexError) drops the file to the full-text
        // raw scan — the detector still over-approximates rather than passing silently.
        // (buck2 itself rejects such a file, so this is a defensive posture, not a live shape.)
        let bad = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:serde\\q\"],\n)\n";
        let deps = extract_buck_library_thirdparty_deps(bad);
        assert!(
            deps.contains("serde"),
            "raw-scan fallback must still surface the dep: {deps:?}"
        );
    }

    #[test]
    fn escape_spelled_dep_removal_is_sound_or_refused_never_corrupt() {
        // Review F1 fixer direction: with correct cooking the parsed element equals the label,
        // so removal targets exactly the escape-spelled element's ORIGINAL span; the harness
        // round-trip validates no collateral. Never a mis-cooked edit.
        let buck = "rust_library(\n    name = \"x\",\n    deps = [\n        \"third-party//:k\\x75be\",\n        \"third-party//:serde\",\n    ],\n)\n";
        match remove_buck_dep_edges_text(buck, "kube") {
            Some(out) => {
                assert!(
                    !out.contains("k\\x75be"),
                    "escape-spelled element removed: {out}"
                );
                assert!(out.contains("third-party//:serde"), "no collateral: {out}");
                let after = extract_buck_library_thirdparty_deps(&out);
                assert!(
                    !after.contains("kube"),
                    "kube edge gone after reparse: {after:?}"
                );
                assert!(after.contains("serde"), "serde survives reparse: {after:?}");
            }
            None => {
                // Refusal is also sound (file untouched) — but silent corruption never is.
            }
        }
    }

    #[test]
    fn kwargs_splat_widens_to_whole_file_scan() {
        // Review F2: deps routed through a **KW splat live in a CLEAN dict assignment the
        // call-span scan never covers. The opaque-args trigger must widen to the whole file.
        let buck = concat!(
            "KW = {\"name\": \"x\", \"deps\": [\"third-party//:kube\"]}\n",
            "rust_library(**KW)\n",
        );
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("kube"),
            "splat-routed dep must surface: {deps:?}"
        );
    }

    #[test]
    fn load_aliased_rule_name_widens_to_whole_file_scan() {
        // Review F2: a load() alias takes the rust_library name off the call site entirely.
        let buck = concat!(
            "load(\":defs.bzl\", my_lib = \"rust_library\")\n",
            "my_lib(\n    name = \"x\",\n    deps = [\"third-party//:kube\"],\n)\n",
        );
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("kube"),
            "load-aliased target's dep must surface: {deps:?}"
        );
    }

    #[test]
    fn comment_mention_of_a_dep_is_not_a_dep() {
        // Comment-blind class closure: a `third-party//:` mention in a COMMENT inside the block
        // must not be extracted (the pre-kernel scanner raw-scanned comments too).
        let buck = "rust_library(\n    name = \"x\",\n    # TODO: maybe add third-party//:kube one day\n    deps = [\"third-party//:serde\"],\n)\n";
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(deps.contains("serde"), "{deps:?}");
        assert!(
            !deps.contains("kube"),
            "a comment mention must not be a dep: {deps:?}"
        );
    }

    // --------------------------------------------------------------------------
    // HIGH-2: renamed dep liveness uses both real name and rename key
    // --------------------------------------------------------------------------

    #[test]
    fn renamed_dep_is_not_auto_fixable() {
        // `foo = { package = "kube" }` — src uses `foo::` not `kube::`. The dep is renamed, so it
        // must never be auto-fixed regardless of the used_in_src flag. HIGH-2 regression.
        let obs = json!({
            "kernel_crates_found": 1,
            "crates": [{
                "kernel": "oya-bad-kernel",
                "member_path": "crates/oya-bad-kernel",
                "closure": [{
                    "name": "oya-bad-kernel",
                    "member_path": "crates/oya-bad-kernel",
                    "via": ["oya-bad-kernel"],
                    "cargo_deps": ["kube"],
                    "buck_deps": [],
                    "cargo_dep_used_in_src": {"kube": false},
                    "build_dep_names": [],
                    "cargo_dep_rename_keys": {"kube": "foo"},
                    "feature_backed_optional_deps": []
                }]
            }]
        });
        let findings = evaluate_keyed(&policy(), &obs);
        let f = findings
            .iter()
            .find(|f| f.key.ends_with(":kube"))
            .expect("kube finding");
        assert!(
            !f.auto_fixable,
            "renamed dep must NOT be auto-fixable: {f:?}"
        );
        assert!(plan_fixes(&policy(), &obs).is_empty());
    }

    // --------------------------------------------------------------------------
    // HIGH-3: malformed policy fails closed
    // --------------------------------------------------------------------------

    #[test]
    fn malformed_deny_match_value_fails_closed() {
        // A typo'd `match` value (neither "exact" nor "prefix") must produce KP-POLICY-MALFORMED
        // and prevent evaluation — not silently drop the rule. HIGH-3 regression.
        let mut p = policy();
        p["deny"] = json!([{"dep": "kube", "match": "prefiks"}]); // typo
        let obs = observed(vec![kernel("oya-bad-kernel", &["kube"], &[])]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings.iter().any(|f| f.code == "KP-POLICY-MALFORMED"),
            "typo'd match value must emit KP-POLICY-MALFORMED: {findings:?}"
        );
        // Evaluation short-circuits — no KP-TRANSIENT-DEP-CARGO is emitted (the deny list is corrupt).
        assert!(
            !findings.iter().any(|f| f.code == "KP-TRANSIENT-DEP-CARGO"),
            "evaluation must short-circuit on malformed policy: {findings:?}"
        );
    }

    #[test]
    fn malformed_deny_missing_dep_key_fails_closed() {
        let mut p = policy();
        p["deny"] = json!([{"dep_name": "kube", "match": "exact"}]); // wrong field name
        let obs = observed(vec![kernel("oya-bad-kernel", &["kube"], &[])]);
        let findings = evaluate_keyed(&p, &obs);
        assert!(
            findings.iter().any(|f| f.code == "KP-POLICY-MALFORMED"),
            "missing dep key must emit KP-POLICY-MALFORMED: {findings:?}"
        );
    }

    // --------------------------------------------------------------------------
    // CRITICAL-A / H1–H4: sound feature-reference refusal (all syntaxes, all tables)
    // --------------------------------------------------------------------------

    /// H1: `k8s = ["kube/client"]` — dep referenced in features via sub-feature path.
    /// collect_features_referenced_deps must catch this and refuse auto-fix.
    #[test]
    fn h1_feature_subfeat_path_refusal() {
        // Manifest: kube declared normally (not optional), but [features] k8s = ["kube/client"]
        // The token "kube" is extracted from "kube/client" → feature-backed → refusal.
        let toml_src = r#"
[package]
name = "fake-kernel"
version = "0.1.0"

[dependencies]
kube = "0.88"

[features]
k8s = ["kube/client"]
"#;
        let doc: toml::Value = toml_src.parse().unwrap();
        let mut feature_backed = BTreeSet::new();
        collect_features_referenced_deps(&doc, &mut feature_backed);
        assert!(
            feature_backed.contains("kube"),
            "H1: kube via 'kube/client' in [features] must be feature-backed: {feature_backed:?}"
        );
    }

    /// H2: `["kube?/client"]` — optional-dep activation syntax with `?`.
    #[test]
    fn h2_feature_optional_activation_refusal() {
        let toml_src = r#"
[package]
name = "fake-kernel"
version = "0.1.0"

[dependencies]
kube = { version = "0.88", optional = true }

[features]
k8s = ["kube?/client"]
"#;
        let doc: toml::Value = toml_src.parse().unwrap();
        let mut feature_backed = BTreeSet::new();
        collect_features_referenced_deps(&doc, &mut feature_backed);
        assert!(
            feature_backed.contains("kube"),
            "H2: kube via 'kube?/client' in [features] must be feature-backed: {feature_backed:?}"
        );
    }

    /// H3: `full = ["kube"]` — bare dep name in features (no dep: prefix, no sub-feature).
    #[test]
    fn h3_feature_bare_dep_name_refusal() {
        let toml_src = r#"
[package]
name = "fake-kernel"
version = "0.1.0"

[dependencies]
kube = { version = "0.88", optional = true }

[features]
full = ["kube"]
"#;
        let doc: toml::Value = toml_src.parse().unwrap();
        let mut feature_backed = BTreeSet::new();
        collect_features_referenced_deps(&doc, &mut feature_backed);
        assert!(
            feature_backed.contains("kube"),
            "H3: kube via bare 'kube' in [features] must be feature-backed: {feature_backed:?}"
        );
    }

    /// H4: dep declared only under `[target.'cfg(unix)'.dependencies]` and referenced in
    /// [features] — collect_features_referenced_deps must scan all target.*.* tables.
    #[test]
    fn h4_target_cfg_dep_feature_refusal() {
        let toml_src = r#"
[package]
name = "fake-kernel"
version = "0.1.0"

[target.'cfg(unix)'.dependencies]
kube = { version = "0.88", optional = true }

[features]
k8s = ["dep:kube"]
"#;
        let doc: toml::Value = toml_src.parse().unwrap();
        let mut feature_backed = BTreeSet::new();
        collect_features_referenced_deps(&doc, &mut feature_backed);
        assert!(
            feature_backed.contains("kube"),
            "H4: kube in target.cfg dep table referenced via dep:kube in [features] must be feature-backed: {feature_backed:?}"
        );
    }

    // --------------------------------------------------------------------------
    // HIGH-B / H5: stray `)` inside a BUCK comment must not false-terminate the block
    // --------------------------------------------------------------------------

    /// H5: a BUCK file whose comment line contains a stray `)` must still detect kube below it.
    /// The shared kernel treats comments as trivia, so the paren cannot end the block early.
    #[test]
    fn h5_stray_paren_in_comment_still_detects_dep() {
        let buck = concat!(
            "rust_library(\n",
            "    name = \"fake-kernel\",\n",
            "    # 1) serde 2) kube — note: stray ) in comment\n",
            "    deps = [\n",
            "        \"third-party//:kube\",\n",
            "    ],\n",
            ")\n",
        );
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("kube"),
            "H5: kube dep must be extracted even when comment has a stray ')': {deps:?}"
        );
    }

    // --------------------------------------------------------------------------
    // MED-C / H6: find_block_end None = skip removal (manifest byte-identical)
    // --------------------------------------------------------------------------

    /// H6: an unterminated rust_library block (no matching close). The DETECTOR must stay
    /// fail-closed (the dep is still reported via the raw over-approximating scan) while the
    /// REMOVER refuses (Ok(false)) and leaves the file byte-identical (MED-C posture, now
    /// enforced by the kernel parse error + harness refusal instead of a None sentinel).
    #[test]
    fn h6_unterminated_block_detect_fail_closed_removal_refused() {
        let buck = concat!(
            "rust_library(\n",
            "    # opening paren in comment: (this breaks naive counters\n",
            "    name = \"fake-kernel\",\n",
            "    deps = [\"third-party//:kube\"],\n",
            "    # no matching close — block is intentionally unterminated\n",
        );
        // Detector: fail-closed — the unparseable text is raw-scanned, kube still surfaces.
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("kube"),
            "H6: detector must over-approximate, never hide: {deps:?}"
        );
        // Remover: refusal — Ok(false), file byte-identical.
        let tmp_path = std::env::temp_dir().join("h6_test_buck_file.BUCK");
        std::fs::write(&tmp_path, buck).unwrap();
        let original = std::fs::read(&tmp_path).unwrap();
        let result = remove_buck_dep_line(&tmp_path, "kube");
        assert!(
            result.is_ok(),
            "H6: remove_buck_dep_line must not error: {result:?}"
        );
        assert!(
            !result.unwrap(),
            "H6: remove_buck_dep_line must refuse unsound input"
        );
        let after = std::fs::read(&tmp_path).unwrap();
        assert_eq!(
            original, after,
            "H6: file must be byte-identical after refusal"
        );
        let _ = std::fs::remove_file(&tmp_path);
    }

    // --------------------------------------------------------------------------
    // ADR-0549: the sound BUCK remover (closes FRIC-1781200001)
    // --------------------------------------------------------------------------

    #[test]
    fn buck_remover_removes_dead_edge_soundly_even_with_comments() {
        // The shape ADR-0547 D6 refused: a comment-bearing deps list. The kernel-backed remover
        // deletes exactly the kube element, preserves every sibling dep + the rust_test block,
        // and the result reparses green.
        let buck = concat!(
            "rust_library(\n",
            "    name = \"fake-kernel\",\n",
            "    # transient deps below — 1) kube must go\n",
            "    deps = [\n",
            "        \"third-party//:kube\",\n",
            "        \"third-party//:serde\",\n",
            "    ],\n",
            ")\n",
            "\n",
            "rust_test(\n",
            "    name = \"fake-kernel-unittest\",\n",
            "    deps = [\"third-party//:kube\"],\n",
            ")\n",
        );
        let tmp_path = std::env::temp_dir().join("oya-kp-buck-remover-sound.BUCK");
        std::fs::write(&tmp_path, buck).unwrap();
        let result = remove_buck_dep_line(&tmp_path, "kube");
        assert_eq!(result, Ok(true), "sound removal must apply");
        let after = std::fs::read_to_string(&tmp_path).unwrap();
        let lib_deps = extract_buck_library_thirdparty_deps(&after);
        assert!(!lib_deps.contains("kube"), "kube edge removed: {after}");
        assert!(lib_deps.contains("serde"), "serde survives: {after}");
        assert!(
            after.contains("rust_test") && after.matches("third-party//:kube").count() == 1,
            "the rust_test kube edge is out of scope and must survive: {after}"
        );
        let _ = std::fs::remove_file(&tmp_path);
    }

    #[test]
    fn buck_remover_refuses_unmodeled_deps_shapes() {
        // deps via a variable / select(): the label is present but not a plain list element —
        // the remover must refuse rather than guess.
        let via_var = "DEPS = [\"third-party//:kube\"]\nrust_library(\n    name = \"x\",\n    deps = DEPS,\n)\n";
        assert_eq!(
            remove_buck_dep_edges_text(via_var, "kube"),
            None,
            "var-carried dep must refuse"
        );
        let via_select = "rust_library(\n    name = \"x\",\n    deps = select({\"cfg\": [\"third-party//:kube\"]}),\n)\n";
        assert_eq!(
            remove_buck_dep_edges_text(via_select, "kube"),
            None,
            "select-carried dep must refuse"
        );
        let absent = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:serde\"],\n)\n";
        assert_eq!(
            remove_buck_dep_edges_text(absent, "kube"),
            None,
            "absent dep is a no-op refusal"
        );
    }

    #[test]
    fn buck_remover_is_token_exact_against_prefix_sibling_deps() {
        // `kube` must be removable even when `kube-runtime` sits in the same list: the residual
        // check is token-boundary exact, so the longer sibling neither matches nor false-refuses.
        let buck = "rust_library(\n    name = \"x\",\n    deps = [\n        \"third-party//:kube\",\n        \"third-party//:kube-runtime\",\n    ],\n)\n";
        let out = remove_buck_dep_edges_text(buck, "kube")
            .expect("kube removal must not be blocked by kube-runtime");
        assert!(
            !contains_dep_token(&out, "third-party//:kube"),
            "kube gone: {out}"
        );
        assert!(
            out.contains("third-party//:kube-runtime"),
            "kube-runtime survives: {out}"
        );
        // And removing a dep that is ONLY present as a longer sibling refuses (absent token).
        let only_runtime =
            "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:kube-runtime\"],\n)\n";
        assert_eq!(
            remove_buck_dep_edges_text(only_runtime, "kube"),
            None,
            "prefix sibling alone is not the dep"
        );
    }

    // --------------------------------------------------------------------------
    // MED-4: indented closing parens do not swallow subsequent blocks
    // --------------------------------------------------------------------------

    #[test]
    fn indented_closing_paren_does_not_swallow_following_block() {
        // A rust_library block whose `)` is indented (not at column 0) must end exactly there.
        // Pinned through the PRODUCTION extractor — an indented-close rust_library followed by a
        // rust_test must yield only the library dep.
        let buck = "rust_library(\n    name = \"x\",\n    deps = [\n        \"third-party//:serde\",\n    ],\n    )\n";
        let deps = extract_buck_library_thirdparty_deps(buck);
        assert!(
            deps.contains("serde"),
            "indented-close block parses: {deps:?}"
        );
        let buck2 = "rust_library(\n    name = \"x\",\n    deps = [\"third-party//:serde\"],\n    )\nrust_test(\n    name = \"y\",\n    deps = [\"third-party//:kube\"],\n)\n";
        let deps2 = extract_buck_library_thirdparty_deps(buck2);
        assert!(deps2.contains("serde"), "library dep extracted: {deps2:?}");
        assert!(
            !deps2.contains("kube"),
            "rust_test dep must NOT be swallowed by the indented-close library span: {deps2:?}"
        );
    }
}
