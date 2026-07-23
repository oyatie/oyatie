//! # cloud-ci-no-graphql-without-adr (ADR-0565)
//!
//! The zero-GraphQL reintroduction gate. ADR-0565 (founder 2026-06-21, door: one-way) removes ALL
//! GraphQL from the owned API surface: the canonical surface set is REST + gRPC + AsyncAPI + realtime
//! (SSE / WebSocket / gRPC-streaming), and GraphQL returns ONLY by a future ADR that explicitly
//! reverses ADR-0565. The drop PR (#775 / ADR-0565) deletes every GraphQL artifact; THIS gate is the
//! enforcement half (enforcement-layering doctrine: the drop is the construction, the gate is the
//! backstop that prevents recurrence). It fails CLOSED if a change reintroduces, repo-wide, WITHOUT
//! referencing an accepted authorizing ADR id, EITHER:
//! - a GraphQL execution/parse library in any `Cargo.toml` (`async-graphql`, `juniper`,
//!   `graphql-parser`, `cynic`, `apollo-*`, … — the forbidden set is DATA), OR
//! - any `.graphql` / `.gql` / `*.sdl` GraphQL schema file (added or edited).
//!
//! ## Candidate-tree evaluation, NOT a frozen merge-base
//! The gate scans the CANDIDATE tree directly — the live workspace `Cargo.toml` manifests resolved
//! via `oya-workspace-members-kernel` (glob-aware; NO `cargo metadata`/`buck2` shell-out) plus a
//! read-only walk of the candidate tree for GraphQL schema files. It does NOT diff a frozen
//! merge-base baseline. This is deliberate: a frozen-baseline predicate evaluated at PR-tier against
//! the merge-base but at push-tier against the integrated tip is the documented PR/push
//! baseline-asymmetry false-green (gate-baseline-pr-push-asymmetry memo) — a GraphQL artifact added
//! on dev between branch-point and merge would pass PR-tier and only fail on the integrated tip.
//! Evaluating the candidate tree means the verdict is the SAME at PR-tier and push-tier: any GraphQL
//! artifact present in the tree (without an authorizing ADR ref) is RED, full stop.
//!
//! ## Frozen baseline = EMPTY
//! The drop PR leaves the tree GraphQL-free, so this gate ships born-blocking with an EMPTY baseline:
//! there is no shrink-only legacy debt. Any NEW GraphQL artifact fails closed on arrival.
//!
//! ## ADR escape-hatch (allowlist-gated + ADR-validated; review-gated, NOT mechanically attacker-proof)
//! GraphQL is admissible ONLY via a future ADR that explicitly reverses ADR-0565 (ADR-0565
//! "Decision"). The gate honors that, but it does NOT trust any `ADR-NNNN` token a candidate file
//! prints. A citation launders a forbidden artifact iff BOTH hold:
//! 1. The cited id is in the policy `authorizing_adrs` ALLOWLIST. That list is EMPTY today — nothing
//!    authorizes GraphQL — so an arbitrary, fabricated, or typo id (`ADR-9999`, `ADR-1234`,
//!    `ADR-05650`) CANNOT launder. A real reversal first adds its id to the policy (a reviewed DATA
//!    change), and only then can a file cite it.
//! 2. Defense in depth: the cited id RESOLVES to a real `docs/decisions/ADR-NNNN-*.md` whose
//!    frontmatter `status` is `Accepted` and whose frontmatter `supersedes`/`amends`/`reverses` list
//!    (parsed as a YAML list-or-scalar, not a body-anywhere match) names the forbidding ADR id. A
//!    Proposed/Draft ADR, a missing file, an ADR that names the forbidding id only in `related:` or
//!    prose, or an ADR that uses words like "not superseded" does NOT launder — even if it is
//!    (mistakenly) allowlisted.
//!
//! **Security framing**: this escape-hatch is REVIEW-GATED and OWNERS-protected — not mechanically
//! attacker-proof against an insider who controls both `authorizing_adrs` in the policy file AND the
//! matching ADR in `docs/decisions`. The security posture is: (a) the policy file's `authorizing_adrs`
//! requires an OWNERS-approved code review (architecture-council/founder sign-off), and (b) the ADR
//! requires a separate code review in `docs/decisions`. An attacker with review authority over BOTH
//! files could bypass the gate — the control is process, not cryptographic. For the zero-GraphQL
//! doctrine this is sufficient: a legitimate reversal of ADR-0565 genuinely requires both an Accepted
//! ADR and an allowlist edit; the gate validates both are present and coherent.
//!
//! The cited id is also always a DIFFERENT ADR than the forbidding ADR (`policy.forbidding_adr`), so a
//! file can never launder itself by merely mentioning ADR-0565 (the rule it would be violating). The
//! set of ids that satisfy BOTH conditions is computed ONCE by the collector (the only I/O) and
//! carried in `observed.valid_authorizing_adrs`; the pure evaluator then admits a forbidden artifact
//! only if it cites one of those validated ids. This is construction-over-flag: a real reversal is a
//! reviewed Accepted ADR plus a separately reviewed policy allowlist edit, and the gate validates both
//! — never a bare suppression comment.
//!
//! ## Born pack-shaped
//! The crate is a NEUTRAL engine. All repo-specifics — the forbidden crate set (exact + prefix
//! rules), the GraphQL schema extensions, the forbidding-ADR id, the workspace-member floor — are
//! DATA in `no-graphql-without-adr-policy.json`. Nothing oyatie-specific is hardcoded in Rust; a
//! different repo adopts the gate by repointing the policy.
//!
//! ## Kernel contract
//! - [`collect_graphql_artifacts`] `(root, policy) -> observed` is the ONLY I/O: read-only `fs`
//!   reads of the candidate tree (member `Cargo.toml` manifests + a `.graphql`/`.gql`/`.sdl` walk).
//!   No shell, no network, no VCS. Writes no temp files.
//! - [`evaluate_keyed`] `(policy, observed) -> BTreeSet<Finding>` is PURE and unit-testable without a
//!   filesystem; it applies the forbidden set + the ADR escape-hatch to the observed artifacts.
//! - [`evaluate`] is the bare-code projection of [`evaluate_keyed`], the single source of the verdict.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `NGQL-FORBIDDEN-LIB`     — a GraphQL execution/parse library is declared in a `Cargo.toml`
//!   dependency table (any manifest in the tree, including a `[workspace.dependencies]` rename)
//!   without an allowlisted+validated authorizing ADR reference.
//! - `NGQL-SCHEMA-FILE`       — a `.graphql`/`.gql`/`.graphqls`/`.gqls`/`.sdl` GraphQL schema file is
//!   present without an allowlisted+validated authorizing ADR reference.
//! - `NGQL-BUILD-GRAPH-SCHEMA-GLOB` — a `BUCK` build graph still admits GraphQL schema files via a
//!   `**/*.graphql`/`.gql`/`.sdl` glob, which would make regression invisible to owners.
//! - `NGQL-LOCK-FORBIDDEN`    — a forbidden GraphQL crate is present in the resolved `Cargo.lock`
//!   graph (catches a transitive reintroduction no manifest names directly), without a tree-wide
//!   allowlisted+validated authorizing ADR reference.
//! - `NGQL-EMPTY-SCAN`        — the workspace member census is below the policy floor (catches a
//!   broken CWD / member-glob that would otherwise be a silent false-green).
//! - `NGQL-POLICY-GATE-ID-MISMATCH` — the policy `gate_id` is not [`GATE_ID`] (fail-closed).
//! - `NGQL-POLICY-MALFORMED`  — the policy `forbidden_crates` / `schema_extensions` is missing or
//!   malformed (fail-closed: the gate would have nothing to enforce).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use oya_workspace_members_kernel::resolve_member_dirs;
use serde_json::{Value, json};

/// The gate id, matching the buck2 target + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-no-graphql-without-adr";

/// The blocking + structural violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 7] = [
    "NGQL-FORBIDDEN-LIB",
    "NGQL-SCHEMA-FILE",
    "NGQL-BUILD-GRAPH-SCHEMA-GLOB",
    "NGQL-LOCK-FORBIDDEN",
    "NGQL-EMPTY-SCAN",
    "NGQL-POLICY-GATE-ID-MISMATCH",
    "NGQL-POLICY-MALFORMED",
];

/// The sentinel key for codes that are policy-level rather than per-artifact.
const POLICY_KEY: &str = "<policy>";

// ---------------------------------------------------------------------------
// Collection (the only I/O; read-only, hermetic — no shell / network / VCS)
// ---------------------------------------------------------------------------

/// Errors collecting the observed GraphQL-artifact view. Returned instead of panicking so the caller
/// (CI / a controller) decides how to surface them — a malformed manifest or unreadable tree is a
/// fail-closed error, never a silently skipped artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    /// Workspace-member resolution failed (the candidate-tree member universe is unknown).
    ResolveMembers(String),
    /// A read-only filesystem operation failed (the candidate tree could not be scanned).
    Io(String),
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::ResolveMembers(message) => {
                write!(
                    f,
                    "no-graphql-without-adr resolve workspace members: {message}"
                )
            }
            CollectError::Io(message) => write!(f, "no-graphql-without-adr io: {message}"),
        }
    }
}

impl std::error::Error for CollectError {}

/// The forbidden GraphQL crate rules declared in policy DATA. Each rule denies a crate name either
/// EXACTLY or as a PREFIX (so `apollo-*` covers `apollo-router`, `apollo-compiler`, …). Returns the
/// rules in canonical (sorted-by-name) order; `None` if the `forbidden_crates` list is absent.
fn forbidden_crate_rules(policy: &Value) -> Option<Vec<CrateRule>> {
    let list = policy.get("forbidden_crates").and_then(Value::as_array)?;
    let mut out = Vec::new();
    for entry in list {
        let Some(name) = entry.get("crate").and_then(Value::as_str) else {
            continue;
        };
        let prefix = entry.get("match").and_then(Value::as_str) == Some("prefix");
        out.push(CrateRule {
            name: name.to_owned(),
            prefix,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Some(out)
}

/// One forbidden-crate rule: a crate name matched EXACTLY or as a PREFIX.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CrateRule {
    name: String,
    prefix: bool,
}

impl CrateRule {
    fn matches(&self, dep: &str) -> bool {
        if self.prefix {
            dep.starts_with(&self.name)
        } else {
            dep == self.name
        }
    }
}

/// The GraphQL schema-file extensions declared in policy DATA (lowercased, no leading dot), in
/// canonical order; `None` if the list is absent.
fn schema_extensions(policy: &Value) -> Option<Vec<String>> {
    let list = policy.get("schema_extensions").and_then(Value::as_array)?;
    let mut out: BTreeSet<String> = BTreeSet::new();
    for entry in list {
        if let Some(ext) = entry.as_str() {
            out.insert(ext.trim_start_matches('.').to_ascii_lowercase());
        }
    }
    Some(out.into_iter().collect())
}

/// Collect the candidate-tree GraphQL-artifact view the policy asks about.
///
/// Read-only scans of the candidate tree — NO shell, NO network, NO VCS:
/// 0. The set of VALIDATED authorizing ADR ids: each id in the policy `authorizing_adrs` allowlist
///    that also resolves to a real `docs/decisions/ADR-NNNN-*.md` whose frontmatter status is
///    `Accepted` and that supersedes/reverses the forbidding ADR. A citation only launders an
///    artifact if it names one of these ids (computed once here, the only I/O).
/// 1. EVERY `Cargo.toml` in the candidate tree (NOT only resolved workspace members — a non-member
///    dir or excluded nested workspace can still declare a forbidden dep): its declared
///    dependency-table crate names — resolving `{ workspace = true }` inheritance back to the root
///    `[workspace.dependencies]` (with `package = "<real>"` renames) — plus which authorizing ADR ids
///    that manifest cites.
/// 2. A recursive walk for `.graphql`/`.gql`/`.graphqls`/`.gqls`/`.sdl` schema files (skipping the
///    policy-declared `excluded_dirs`, e.g. `third-party/`), each tagged with which authorizing ADR
///    ids it cites (in its own contents or in a sibling `<file>.adr` marker).
/// 3. The resolved `Cargo.lock` graph (if present): every package name, so a forbidden GraphQL crate
///    pulled in transitively (named by no manifest directly) still fails closed.
///
/// Emits:
/// `{ "workspace_members_found": <usize>,
///    "valid_authorizing_adrs": [<id>..],
///    "manifests": [ { "member_path", "deps":[<name>..], "cited_adrs":[<id>..] } ],
///    "schema_files": [ { "path", "cited_adrs":[<id>..] } ],
///    "lock": { "present": <bool>, "packages": [<name>..], "cited_adrs":[<id>..] } }`.
pub fn collect_graphql_artifacts(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let forbidding_adr = forbidding_adr(policy);
    let exts = schema_extensions(policy).unwrap_or_default();
    let excluded = excluded_dirs(policy);

    // --- (0) validated authorizing-ADR allowlist (allowlist ∩ docs/decisions validation) ---
    let valid_authorizing_adrs =
        validated_authorizing_adrs(root, policy, forbidding_adr.as_deref())?;

    // --- (1) EVERY Cargo.toml in the candidate tree (members AND non-members) ---
    let member_dirs = resolve_member_dirs(root)
        .map_err(|error| CollectError::ResolveMembers(error.to_string()))?;
    let members_found = member_dirs.len();
    let root_workspace_deps = read_root_workspace_deps(root)?;
    let mut manifest_paths: Vec<String> = Vec::new();
    collect_manifest_paths(root, root, &excluded, &mut manifest_paths)?;
    manifest_paths.sort();
    let mut manifests = Vec::new();
    for rel in &manifest_paths {
        let cargo_path = root.join(rel);
        let text = match fs::read_to_string(&cargo_path) {
            Ok(text) => text,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(CollectError::Io(format!(
                    "read {}: {e}",
                    cargo_path.display()
                )));
            }
        };
        let deps = parse_manifest_dep_names_with_workspace(&text, &root_workspace_deps);
        let cited = cited_authorizing_adrs(&text, forbidding_adr.as_deref());
        // Report the manifest's parent dir (matching the prior member-relative key shape:
        // `<dir>/Cargo.toml`), so the finding key is `<dir>/Cargo.toml:<dep>`.
        let member_path = rel.strip_suffix("/Cargo.toml").unwrap_or(rel);
        manifests.push(json!({
            "member_path": member_path,
            "deps": deps.into_iter().collect::<Vec<_>>(),
            "cited_adrs": cited.into_iter().collect::<Vec<_>>(),
        }));
    }

    // --- (2) GraphQL schema files anywhere in the candidate tree ---
    let mut schema_files: Vec<Value> = Vec::new();
    collect_schema_files(
        root,
        root,
        &exts,
        &excluded,
        forbidding_adr.as_deref(),
        &mut schema_files,
    )?;
    schema_files.sort_by(|a, b| {
        a.get("path")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(b.get("path").and_then(Value::as_str).unwrap_or_default())
    });

    // --- (3) BUCK build graph globs that would keep admitting GraphQL schema files ---
    let mut build_graph_schema_globs: Vec<Value> = Vec::new();
    collect_build_graph_schema_globs(
        root,
        root,
        &exts,
        &excluded,
        forbidding_adr.as_deref(),
        &mut build_graph_schema_globs,
    )?;
    build_graph_schema_globs.sort_by(|a, b| {
        a.get("path")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(b.get("path").and_then(Value::as_str).unwrap_or_default())
    });

    // --- (4) Cargo.lock resolved graph (transitive reintroduction catch) ---
    let lock = collect_lock_packages(root, forbidding_adr.as_deref())?;

    Ok(json!({
        "workspace_members_found": members_found,
        "valid_authorizing_adrs": valid_authorizing_adrs.into_iter().collect::<Vec<_>>(),
        "manifests": manifests,
        "schema_files": schema_files,
        "build_graph_schema_globs": build_graph_schema_globs,
        "lock": lock,
    }))
}

/// The root manifest `[workspace.dependencies]` table: dependency-key -> REAL crate name (honoring
/// `package = "<real>"` renames). Empty if absent. A member's `{ workspace = true }` inheritance
/// resolves THROUGH this table, so a forbidden lib renamed at the workspace seam (e.g.
/// `gqlrt = { package = "async-graphql" }`) is denied on its real name even though the member only
/// writes `gqlrt = { workspace = true }`.
fn read_root_workspace_deps(root: &Path) -> Result<WorkspaceDeps, CollectError> {
    let cargo_path = root.join("Cargo.toml");
    let text = match fs::read_to_string(&cargo_path) {
        Ok(text) => text,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(WorkspaceDeps::default());
        }
        Err(e) => {
            return Err(CollectError::Io(format!(
                "read {}: {e}",
                cargo_path.display()
            )));
        }
    };
    Ok(parse_root_workspace_deps(&text))
}

/// Recursively collect repo-relative `Cargo.toml` paths under `dir`, skipping the VCS dir, the
/// `target`/`buck-out` build dirs, and any policy-`excluded` prefix. Read-only; missing dirs skipped.
fn collect_manifest_paths(
    root: &Path,
    dir: &Path,
    excluded: &[String],
    out: &mut Vec<String>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if is_skipped_dir(&rel_str, excluded) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        // Follow symlinks to directories (file_type().is_dir() is false for symlink-to-dir, but
        // symlinked workspace members ARE counted in the census floor, so their Cargo.toml deps
        // must be scanned too). Use fs::metadata (which follows symlinks) to detect symlink-to-dir.
        // Cycle protection: only follow if the resolved canonical path descends from `root` or
        // is within the same filesystem — we bound the recursion depth by checking the entry is a
        // symlink and using fs::metadata() for the dir check; the recursive call will skip
        // already-visited paths via `is_skipped_dir` (VCS/build exclusion).
        let is_dir = if file_type.is_dir() {
            true
        } else if file_type.is_symlink() {
            fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false)
        } else {
            false
        };
        if is_dir {
            collect_manifest_paths(root, &path, excluded, out)?;
        } else if rel_str == "Cargo.toml" || rel_str.ends_with("/Cargo.toml") {
            out.push(rel_str);
        }
    }
    Ok(())
}

/// Whether a repo-relative path is in an always-skipped dir (VCS / build output) or under a
/// policy-`excluded` prefix. Shared by every tree walk so the LIB leg, schema leg, and manifest leg
/// all honor the same scope.
fn is_skipped_dir(rel_str: &str, excluded: &[String]) -> bool {
    const ALWAYS_SKIP: [&str; 3] = [".git", "target", "buck-out"];
    if ALWAYS_SKIP
        .iter()
        .any(|d| rel_str == *d || rel_str.starts_with(&format!("{d}/")))
    {
        return true;
    }
    excluded
        .iter()
        .any(|ex| rel_str == *ex || rel_str.starts_with(&format!("{ex}/")))
}

/// The forbidding-ADR id (`policy.forbidding_adr`, e.g. `ADR-0565`) — the decision this gate enforces.
/// A forbidden artifact cannot launder itself by citing THIS id; the escape-hatch requires citing a
/// DIFFERENT (reversing/authorizing) ADR.
fn forbidding_adr(policy: &Value) -> Option<String> {
    policy
        .get("forbidding_adr")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

/// The directory prefixes the scan skips (policy DATA, e.g. `third-party/`, `.git/`). A vendored
/// third-party crate's `.graphql`/`.gql`/`.sdl` test fixtures are not part of the OWNED API surface
/// ADR-0565 governs, so they are excluded by DATA, not by a Rust hardcode.
fn excluded_dirs(policy: &Value) -> Vec<String> {
    policy
        .get("excluded_dirs")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(|d| d.trim_end_matches('/').to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// The `ADR-NNNN` ids `text` cites OTHER than the forbidding ADR (the rule the artifact would be
/// violating — naming it can never self-launder). These are merely the CITED ids; whether any of them
/// actually launders is decided later against the VALIDATED authorizing-ADR set (allowlist ∩ real
/// Accepted-and-reversing ADR file). Citing an id is necessary but NOT sufficient for the escape-hatch.
fn cited_authorizing_adrs(text: &str, forbidding_adr: Option<&str>) -> BTreeSet<String> {
    adr_citations(text)
        .into_iter()
        .filter(|cited| Some(cited.as_str()) != forbidding_adr)
        .collect()
}

/// The set of authorizing ADR ids that may launder a forbidden artifact: the policy `authorizing_adrs`
/// allowlist INTERSECTED with the ids that pass defense-in-depth validation against the real
/// `docs/decisions/` tree. Both conditions are required:
/// - ALLOWLIST: only ids explicitly enumerated in `policy.authorizing_adrs` are eligible. That list is
///   EMPTY today (nothing authorizes GraphQL), so a fabricated/typo id can never launder.
/// - VALIDATION: the id must resolve to a real `docs/decisions/ADR-<id>-*.md` whose frontmatter
///   `status` is `Accepted` (case-insensitive) and that supersedes/reverses the forbidding ADR (its
///   `supersedes`/`amends`/`reverses` frontmatter, or its body, names the forbidding id with a
///   reverse/supersede verb). The forbidding id itself is never eligible.
///
/// Returns the validated ids. A non-existent `docs/decisions/` dir yields an empty set (no id can be
/// validated) — fail-closed, since with an empty allowlist there is nothing to validate anyway.
fn validated_authorizing_adrs(
    root: &Path,
    policy: &Value,
    forbidding_adr: Option<&str>,
) -> Result<BTreeSet<String>, CollectError> {
    let allowlist: Vec<String> = policy
        .get("authorizing_adrs")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    if allowlist.is_empty() {
        return Ok(BTreeSet::new());
    }
    let decisions_dir = policy
        .get("decisions_dir")
        .and_then(Value::as_str)
        .unwrap_or("docs/decisions");
    let dir = root.join(decisions_dir);
    let mut validated = BTreeSet::new();
    for id in allowlist {
        if Some(id.as_str()) == forbidding_adr {
            // The forbidding ADR can never authorize its own reversal.
            continue;
        }
        let Some(forbidding) = forbidding_adr else {
            continue;
        };
        if adr_file_reverses_forbidding(&dir, &id, forbidding)? {
            validated.insert(id);
        }
    }
    Ok(validated)
}

/// Whether the ADR with id `id` resolves, in `decisions_dir`, to a real `ADR-<id>-*.md` file whose
/// frontmatter status is `Accepted` and that supersedes/reverses `forbidding`. Read-only; a missing
/// file or dir is `false` (fail-closed). The file is `ADR-NNNN-*.md` (id then a `-`-separated slug).
fn adr_file_reverses_forbidding(
    decisions_dir: &Path,
    id: &str,
    forbidding: &str,
) -> Result<bool, CollectError> {
    let entries = match fs::read_dir(decisions_dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => {
            return Err(CollectError::Io(format!(
                "read dir {}: {e}",
                decisions_dir.display()
            )));
        }
    };
    let prefix = format!("{id}-");
    for entry in entries {
        let entry = entry.map_err(|e| {
            CollectError::Io(format!("read entry in {}: {e}", decisions_dir.display()))
        })?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.starts_with(&prefix) && name.ends_with(".md")) {
            continue;
        }
        let body = match fs::read_to_string(entry.path()) {
            Ok(body) => body,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(CollectError::Io(format!(
                    "read {}: {e}",
                    entry.path().display()
                )));
            }
        };
        return Ok(adr_is_accepted_and_reverses(&body, forbidding));
    }
    Ok(false)
}

/// Pure predicate: does an ADR document `body` carry frontmatter `status: Accepted` (case-insensitive)
/// AND list `forbidding` in its frontmatter `supersedes:`, `amends:`, or `reverses:` field (parsed as
/// a YAML list-or-scalar)? This is intentionally stricter than generic decision liveness: an in-place
/// `Amended` ADR does not newly authorize GraphQL. The reversal is required STRUCTURALLY in the frontmatter — a body-anywhere
/// mention of the forbidding id (in `related:`, in prose, or in a phrase like "has not been
/// superseded") does NOT satisfy the requirement. This prevents an attacker from citing the forbidding
/// ADR in an unrelated field and claiming the reversal was present. Exposed for unit tests.
pub fn adr_is_accepted_and_reverses(body: &str, forbidding: &str) -> bool {
    if !adr_status_is_accepted(body) {
        return false;
    }
    adr_frontmatter_reverses(body, forbidding)
}

/// Whether the first `---`…`---` frontmatter block contains a `supersedes:`, `amends:`, or
/// `reverses:` field whose value (list-or-scalar) includes `forbidding`. Accepts both:
/// - Inline scalar:  `supersedes: ADR-0565`
/// - YAML list item: `supersedes:\n  - ADR-0565`
///
/// Values are compared case-sensitively (canonical `ADR-NNNN` form). A match in `related:` or
/// any other frontmatter field does NOT satisfy the requirement.
fn adr_frontmatter_reverses(body: &str, forbidding: &str) -> bool {
    const REVERSAL_FIELDS: [&str; 3] = ["supersedes:", "amends:", "reverses:"];
    let mut lines = body.lines();
    // Skip the opening `---` (already validated by adr_status_is_accepted).
    let first = lines.next().unwrap_or("").trim();
    if first != "---" {
        return false;
    }
    // Track whether we are currently inside a reversal field's multi-line list.
    let mut in_reversal_list = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            // End of frontmatter.
            return false;
        }
        // A new top-level key (not indented, ends with `:` or `: value`) resets list context.
        let is_top_level_key =
            !line.starts_with(' ') && !line.starts_with('\t') && trimmed.contains(':');
        if is_top_level_key {
            in_reversal_list = REVERSAL_FIELDS.iter().any(|f| trimmed.starts_with(f));
            // Check for inline scalar: `supersedes: ADR-0565`
            if in_reversal_list {
                let field_end = trimmed.find(':').unwrap_or(0) + 1;
                let scalar = trimmed[field_end..].trim();
                // Strip optional surrounding quotes.
                let scalar = scalar
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .or_else(|| scalar.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                    .unwrap_or(scalar)
                    .trim();
                if scalar == forbidding {
                    return true;
                }
            }
            continue;
        }
        // Inside a reversal list — check for `- ADR-NNNN` list items.
        if in_reversal_list {
            let item = trimmed.strip_prefix("- ").unwrap_or("").trim();
            // Strip optional quotes.
            let item = item
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| item.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                .unwrap_or(item)
                .trim();
            if item == forbidding {
                return true;
            }
        }
    }
    false
}

/// Whether the ADR frontmatter declares `status: Accepted` (case-insensitive on the value). Only the
/// FIRST `---`…`---` YAML frontmatter block is scanned — a `status:` line inside a fenced code block
/// or prose section does NOT count as the frontmatter status field. Accepts both bare and quoted
/// values (`status: Accepted` and `status: "Accepted"`). Exactly one top-level `status` key is
/// required. A pipe-delimited multi-value status (`Proposed | Accepted | ...`) never counts; only a
/// resolved single Accepted value can authorize, so a not-yet-ratified `Proposed` does not launder.
fn adr_status_is_accepted(body: &str) -> bool {
    // Locate the first `---`…`---` YAML frontmatter block. The opening `---` must be the very first
    // line (standard Jekyll/Hugo convention). If no closing `---` is found, the entire header up to
    // the first blank-or-body line is NOT treated as frontmatter — fail closed.
    let mut lines = body.lines();
    let first = lines.next().unwrap_or("").trim();
    if first != "---" {
        return false;
    }
    let mut accepted = None;
    let mut status_count = 0usize;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            return status_count == 1 && accepted == Some(true);
        }
        let Some(rest) = line.strip_prefix("status:") else {
            continue;
        };
        status_count += 1;
        if status_count > 1 {
            return false;
        }
        let raw = rest.trim();
        // Strip optional surrounding quotes (`"Accepted"` or `'Accepted'`).
        let value = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(raw)
            .trim()
            .to_ascii_lowercase();
        // A pipe-delimited status enum (`proposed | accepted | superseded`) is a TEMPLATE, not a
        // ratified value — it does not count as Accepted.
        if value.contains('|') {
            accepted = Some(false);
            continue;
        }
        accepted = Some(value == "accepted");
    }
    false
}

/// Extract every `ADR-NNNN` citation from `text` (4+ digits after `ADR-`). Deterministic, pure;
/// case-sensitive on the `ADR-` prefix (the canonical decision-id form in this repo).
fn adr_citations(text: &str) -> BTreeSet<String> {
    let marker = "ADR-";
    let mut out = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut from = 0usize;
    while let Some(rel) = text[from..].find(marker) {
        let digits_start = from + rel + marker.len();
        let mut end = digits_start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        // Require at least four digits (the canonical `ADR-NNNN` shape) so a stray `ADR-` token is
        // not a false citation.
        if end - digits_start >= 4 {
            out.insert(format!("{marker}{}", &text[digits_start..end]));
        }
        from = digits_start.max(from + rel + 1);
    }
    out
}

/// The root `[workspace.dependencies]` table resolved to dependency-key -> REAL crate name (honoring
/// `package = "<real>"`). A member's `{ workspace = true }` inheritance resolves through this map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceDeps {
    map: std::collections::BTreeMap<String, String>,
}

impl WorkspaceDeps {
    /// The REAL crate name a workspace dependency key resolves to (the `package = "<real>"` rename if
    /// present, else the key itself), or `None` if the key is not in `[workspace.dependencies]`.
    fn real_name(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }
}

/// Parse the root manifest's `[workspace.dependencies]` table. Each entry maps its key to the REAL
/// crate name: `gqlrt = { package = "async-graphql", version = "7" }` maps `gqlrt -> async-graphql`,
/// so a forbidden lib renamed at the workspace seam is still denied on its real name. A bare
/// `serde = "1"` maps `serde -> serde`. Pure helper, exposed for tests.
pub fn parse_root_workspace_deps(text: &str) -> WorkspaceDeps {
    let mut map = std::collections::BTreeMap::new();
    let Ok(doc) = text.parse::<toml::Value>() else {
        return WorkspaceDeps { map };
    };
    if let Some(table) = doc
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|w| w.get("dependencies"))
        .and_then(toml::Value::as_table)
    {
        for (dep_key, spec) in table {
            let real = spec
                .as_table()
                .and_then(|t| t.get("package").and_then(toml::Value::as_str))
                .unwrap_or(dep_key.as_str());
            map.insert(dep_key.clone(), real.to_owned());
        }
    }
    WorkspaceDeps { map }
}

/// Parse the crate names declared in a `Cargo.toml`'s dependency tables WITHOUT workspace resolution.
/// Convenience wrapper over [`parse_manifest_dep_names_with_workspace`] with an empty workspace table;
/// `{ workspace = true }` entries resolve to their local key (no root to inherit from). Pure helper,
/// exposed for tests.
pub fn parse_manifest_dep_names(text: &str) -> BTreeSet<String> {
    parse_manifest_dep_names_with_workspace(text, &WorkspaceDeps::default())
}

/// Parse the crate names declared in a `Cargo.toml`'s dependency tables —
/// `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and the `[target.*.*]` variants.
/// Honors `package = "<real>"` renames (denies on the REAL crate name) AND `{ workspace = true }`
/// inheritance: a `{ workspace = true }` entry resolves through `workspace_deps` back to the root
/// `[workspace.dependencies]` definition (including the root's `package =` rename), so a forbidden lib
/// smuggled via the workspace seam is denied on its real name. dev-dependencies ARE scanned: a GraphQL
/// lib reintroduced as a dev-dep is still a reintroduction of the surface ADR-0565 forbids.
pub fn parse_manifest_dep_names_with_workspace(
    text: &str,
    workspace_deps: &WorkspaceDeps,
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(doc) = text.parse::<toml::Value>() else {
        // FAIL-CLOSED on malformed TOML: fall back to a raw-text over-approximation. Scan for any
        // forbidden crate name appearing as a word in the raw bytes. This avoids the fails-open
        // vulnerability where a forbidden crate in an unparseable Cargo.toml passes GREEN because
        // the empty set has no forbidden-name intersection. The raw scan is intentionally
        // over-approximate (it may flag a crate name in a comment or string literal) — that is the
        // correct bias for a fail-closed gate.
        names.extend(raw_text_dep_names(text, workspace_deps));
        return names;
    };
    collect_dep_table_names(doc.get("dependencies"), workspace_deps, &mut names);
    collect_dep_table_names(doc.get("dev-dependencies"), workspace_deps, &mut names);
    collect_dep_table_names(doc.get("build-dependencies"), workspace_deps, &mut names);
    if let Some(targets) = doc.get("target").and_then(toml::Value::as_table) {
        for target_cfg in targets.values() {
            collect_dep_table_names(target_cfg.get("dependencies"), workspace_deps, &mut names);
            collect_dep_table_names(
                target_cfg.get("dev-dependencies"),
                workspace_deps,
                &mut names,
            );
            collect_dep_table_names(
                target_cfg.get("build-dependencies"),
                workspace_deps,
                &mut names,
            );
        }
    }
    names
}

/// Raw-text fallback used when `parse_manifest_dep_names_with_workspace` receives a TOML file that
/// fails to parse. Scans the raw text for any workspace-dep key that appears as a word boundary
/// match, then resolves it through `workspace_deps` to the real crate name. Also scans for the
/// real crate names directly. This is an OVER-APPROXIMATION (a name appearing in a comment or
/// string value will be flagged) — the correct bias for a fail-closed gate. The caller is
/// responsible for only invoking this on parse failure, not on valid TOML.
fn raw_text_dep_names(text: &str, workspace_deps: &WorkspaceDeps) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    // Check every workspace dep key and its resolved real name.
    for (key, real) in &workspace_deps.map {
        if text_contains_word(text, key) || text_contains_word(text, real) {
            names.insert(real.clone());
        }
    }
    // Also scan for every crate name directly (covers non-workspace cases where the key IS the
    // real name, and where there is no workspace deps table to resolve through).
    for word in raw_crate_name_words(text) {
        names.insert(word);
    }
    names
}

/// Yield every token from `text` that looks like a crate name (alphanumeric + `-` / `_`, length ≥ 2).
/// Used by the raw-text fallback to extract candidate dep names from a malformed TOML manifest.
fn raw_crate_name_words(text: &str) -> impl Iterator<Item = String> + '_ {
    // Split on any character that is NOT alphanumeric, `-`, or `_`.
    text.split(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        .filter(|s| {
            s.len() >= 2
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        .map(str::to_owned)
}

/// Whether `text` contains `word` as a standalone token (surrounded by non-word chars or at
/// string boundaries). Used by `raw_text_dep_names` to detect a specific crate name.
fn text_contains_word(text: &str, word: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let abs = start + pos;
        let before_ok = abs == 0 || {
            let b = text.as_bytes()[abs - 1];
            !b.is_ascii_alphanumeric() && b != b'-' && b != b'_'
        };
        let after_ok = abs + word.len() >= text.len() || {
            let b = text.as_bytes()[abs + word.len()];
            !b.is_ascii_alphanumeric() && b != b'-' && b != b'_'
        };
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
    }
    false
}

/// Collect crate names from one dependency table into `names`. Resolution order per entry:
/// 1. `{ workspace = true }` -> the REAL name the root `[workspace.dependencies]` key resolves to
///    (the root's `package =` rename applies); if the key is not in the root table, fall back to the
///    local key (a malformed manifest, but still denied on its written name).
/// 2. `{ package = "<real>" }` -> the real crate name (a local rename).
/// 3. otherwise -> the dependency key itself.
fn collect_dep_table_names(
    table: Option<&toml::Value>,
    workspace_deps: &WorkspaceDeps,
    names: &mut BTreeSet<String>,
) {
    let Some(table) = table.and_then(toml::Value::as_table) else {
        return;
    };
    for (dep_key, spec) in table {
        let spec_table = spec.as_table();
        let inherits_workspace = spec_table
            .and_then(|t| t.get("workspace").and_then(toml::Value::as_bool))
            == Some(true);
        let real = if inherits_workspace {
            workspace_deps
                .real_name(dep_key)
                .unwrap_or(dep_key.as_str())
        } else {
            spec_table
                .and_then(|t| t.get("package").and_then(toml::Value::as_str))
                .unwrap_or(dep_key.as_str())
        };
        names.insert(real.to_owned());
    }
}

/// Scan the resolved `Cargo.lock` for package names (the transitive-dep catch). Emits
/// `{ "present": <bool>, "packages": [<name>..], "cited_adrs":[<id>..] }`. A missing lock is
/// `present: false` with no packages (the manifest legs still cover declared deps). The lock's own
/// citations (rare, but a generated header could carry one) feed the lock leg's escape-hatch.
fn collect_lock_packages(root: &Path, forbidding_adr: Option<&str>) -> Result<Value, CollectError> {
    let lock_path = root.join("Cargo.lock");
    let text = match fs::read_to_string(&lock_path) {
        Ok(text) => text,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(json!({ "present": false, "packages": [], "cited_adrs": [] }));
        }
        Err(e) => {
            return Err(CollectError::Io(format!(
                "read {}: {e}",
                lock_path.display()
            )));
        }
    };
    let packages = parse_lock_package_names(&text);
    let cited = cited_authorizing_adrs(&text, forbidding_adr);
    Ok(json!({
        "present": true,
        "packages": packages.into_iter().collect::<Vec<_>>(),
        "cited_adrs": cited.into_iter().collect::<Vec<_>>(),
    }))
}

/// Parse the `name = "<crate>"` of every `[[package]]` entry in a `Cargo.lock`. Pure helper, exposed
/// for tests. FAIL-CLOSED on malformed TOML: falls back to a raw-text scan for `name = "<word>"`
/// patterns so a forbidden crate in an unparseable lock still produces findings rather than a
/// silent false-green pass.
pub fn parse_lock_package_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Ok(doc) = text.parse::<toml::Value>() else {
        // Raw-text fallback: scan for `name = "…"` value patterns. This is an over-approximation
        // but correct bias (fail-closed). We extract quoted strings following `name =`.
        names.extend(raw_lock_package_names(text));
        return names;
    };
    if let Some(packages) = doc.get("package").and_then(toml::Value::as_array) {
        for pkg in packages {
            if let Some(name) = pkg.get("name").and_then(toml::Value::as_str) {
                names.insert(name.to_owned());
            }
        }
    }
    names
}

/// Raw-text fallback for `parse_lock_package_names` when the lock file fails to parse as TOML.
/// Scans for `name = "…"` patterns (the canonical Cargo.lock package name field) and extracts the
/// quoted value. Over-approximates (a name in a comment would match) — correct bias for fail-closed.
fn raw_lock_package_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in text.lines() {
        let trimmed = line.trim();
        // Match `name = "…"` or `name = '…'`
        let Some(rest) = trimmed.strip_prefix("name") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim();
        let value = if let Some(inner) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            inner
        } else if let Some(inner) = rest.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
            inner
        } else {
            continue;
        };
        if !value.is_empty() {
            names.insert(value.to_owned());
        }
    }
    names
}

/// Recursively walk `dir` collecting GraphQL schema files (by `exts`), skipping the VCS/build dirs and
/// `excluded` prefixes (relative to `root`). Each row carries its repo-relative path and the
/// authorizing-ADR ids it (or a sibling `<file>.adr` marker) cites. Read-only; missing dirs skipped.
fn collect_schema_files(
    root: &Path,
    dir: &Path,
    exts: &[String],
    excluded: &[String],
    forbidding_adr: Option<&str>,
    out: &mut Vec<Value>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if is_skipped_dir(&rel_str, excluded) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        // Follow symlinks to directories — same rationale as collect_manifest_paths.
        let is_dir = if file_type.is_dir() {
            true
        } else if file_type.is_symlink() {
            fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false)
        } else {
            false
        };
        if is_dir {
            collect_schema_files(root, &path, exts, excluded, forbidding_adr, out)?;
        } else if has_graphql_ext(&rel_str, exts) {
            let body = fs::read_to_string(&path).unwrap_or_default();
            // The schema file may cite the authorizing ADR in its OWN body (a generated header
            // comment) OR in a sibling `<file>.adr` marker file alongside it.
            let marker = path.with_extension(format!(
                "{}.adr",
                path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ));
            let marker_body = fs::read_to_string(&marker).unwrap_or_default();
            let mut cited = cited_authorizing_adrs(&body, forbidding_adr);
            cited.extend(cited_authorizing_adrs(&marker_body, forbidding_adr));
            out.push(json!({
                "path": rel_str,
                "cited_adrs": cited.into_iter().collect::<Vec<_>>(),
            }));
        }
    }
    Ok(())
}

/// Recursively walk `BUCK` files and collect build-graph source globs that still admit GraphQL
/// schema files. A repo may carry zero actual `.graphql` files while stale `srcs` globs keep
/// normalizing their return; this closes that regression hole without shelling out to Buck2.
fn collect_build_graph_schema_globs(
    root: &Path,
    dir: &Path,
    exts: &[String],
    excluded: &[String],
    forbidding_adr: Option<&str>,
    out: &mut Vec<Value>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    let forbidden_globs: Vec<String> = exts.iter().map(|ext| format!("**/*.{ext}")).collect();
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("read entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if is_skipped_dir(&rel_str, excluded) {
            continue;
        }
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        let is_dir = if file_type.is_dir() {
            true
        } else if file_type.is_symlink() {
            fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false)
        } else {
            false
        };
        if is_dir {
            collect_build_graph_schema_globs(root, &path, exts, excluded, forbidding_adr, out)?;
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) != Some("BUCK") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| CollectError::Io(format!("read {}: {e}", path.display())))?;
        let cited = cited_authorizing_adrs(&text, forbidding_adr);
        for (line_idx, line) in text.lines().enumerate() {
            if line.trim_start().starts_with('#') {
                continue;
            }
            if let Some(glob) = forbidden_globs
                .iter()
                .find(|glob| line.contains(glob.as_str()))
            {
                out.push(json!({
                    "path": rel_str,
                    "line": line_idx + 1,
                    "glob": glob,
                    "cited_adrs": cited.iter().cloned().collect::<Vec<_>>(),
                }));
            }
        }
    }
    Ok(())
}

/// Whether a path's extension is one of the GraphQL schema extensions (case-insensitive).
fn has_graphql_ext(path: &str, exts: &[String]) -> bool {
    let lower = path.to_ascii_lowercase();
    exts.iter().any(|ext| lower.ends_with(&format!(".{ext}")))
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
}

impl Finding {
    fn new(code: &str, key: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
            detail: detail.into(),
        }
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

/// Pure evaluator. `policy` is DATA (`no-graphql-without-adr-policy.json`); `observed` is the
/// candidate-tree GraphQL-artifact view shaped by [`collect_graphql_artifacts`].
///
/// RED iff the candidate tree carries a forbidden GraphQL library (in ANY `Cargo.toml`), a forbidden
/// crate in the resolved `Cargo.lock` graph, or a GraphQL schema file, in EVERY case WITHOUT the
/// artifact citing an ALLOWLISTED + VALIDATED authorizing ADR (`observed.valid_authorizing_adrs`). The
/// frozen baseline is EMPTY (the tree is GraphQL-free post-drop), so any such artifact fails closed.
pub fn evaluate_keyed(policy: &Value, observed: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if policy.get("gate_id").and_then(Value::as_str) != Some(GATE_ID) {
        findings.insert(Finding::new(
            "NGQL-POLICY-GATE-ID-MISMATCH",
            POLICY_KEY,
            format!("policy gate_id must be {GATE_ID}"),
        ));
    }

    // Fail CLOSED on a missing/empty forbidden set or schema-extension set rather than silently
    // passing with nothing to enforce.
    let Some(crate_rules) = forbidden_crate_rules(policy) else {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `forbidden_crates` must be a non-null array of {crate, match?} entries; correct the policy before the gate can evaluate",
        ));
        return findings;
    };
    if crate_rules.is_empty() {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `forbidden_crates` resolved to zero crate names; the gate would have nothing to enforce — correct the policy",
        ));
        return findings;
    }
    let Some(exts) = schema_extensions(policy) else {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `schema_extensions` must be a non-null array of extension strings (e.g. \"graphql\", \"gql\", \"sdl\"); correct the policy",
        ));
        return findings;
    };
    if exts.is_empty() {
        findings.insert(Finding::new(
            "NGQL-POLICY-MALFORMED",
            POLICY_KEY,
            "policy `schema_extensions` resolved to zero extensions; the gate would not catch any schema file — correct the policy",
        ));
        return findings;
    }

    let min_expected = policy
        .get("min_expected_workspace_members")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let members = observed
        .get("workspace_members_found")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if members < min_expected {
        findings.insert(Finding::new(
            "NGQL-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "workspace member census {members} is below the policy floor of {min_expected}; the CWD or the member glob is likely broken (fail-closed against a silent false-green where the scan saw an empty tree)"
            ),
        ));
    }

    let forbidding = policy
        .get("forbidding_adr")
        .and_then(Value::as_str)
        .unwrap_or("ADR-0565");

    // The VALIDATED authorizing-ADR ids (allowlist ∩ real Accepted-and-reversing ADR file), computed
    // by the collector. An artifact launders ONLY by citing one of these; a bare/fabricated/typo id
    // is NOT in this set (and the set is empty today — nothing authorizes GraphQL).
    let valid_authorizing: BTreeSet<&str> = observed
        .get("valid_authorizing_adrs")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let cites_validated = |cited_field: &Value| -> bool {
        cited_field
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .any(|id| valid_authorizing.contains(id))
            })
            .unwrap_or(false)
    };

    // --- forbidden GraphQL libraries in ANY Cargo.toml in the candidate tree ---
    let manifests = observed
        .get("manifests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for manifest in &manifests {
        let member_path = manifest
            .get("member_path")
            .and_then(Value::as_str)
            .unwrap_or("<unknown-member>");
        let launders = manifest
            .get("cited_adrs")
            .map(&cites_validated)
            .unwrap_or(false);
        let deps = manifest
            .get("deps")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        for dep in deps {
            let Some(rule) = crate_rules.iter().find(|rule| rule.matches(dep)) else {
                continue;
            };
            if launders {
                // Escape-hatch: this manifest cites an allowlisted, validated reversing ADR — allowed.
                continue;
            }
            let match_note = if rule.prefix {
                format!(" (matched the forbidden prefix `{}`)", rule.name)
            } else {
                String::new()
            };
            findings.insert(Finding::new(
                "NGQL-FORBIDDEN-LIB",
                &format!("{member_path}/Cargo.toml:{dep}"),
                format!(
                    "`{member_path}/Cargo.toml` declares the GraphQL library `{dep}`{match_note}, which {forbidding} forbids in the owned stack (the canonical API surface is REST + gRPC + AsyncAPI + realtime). Remove the dependency. GraphQL is admissible ONLY via a future ADR that explicitly reverses {forbidding}; such an ADR must be Accepted in docs/decisions AND enumerated in the gate policy `authorizing_adrs` allowlist, after which citing its id in this Cargo.toml authorizes the dependency."
                ),
            ));
        }
    }

    // --- GraphQL schema files anywhere in the candidate tree ---
    let schema_files = observed
        .get("schema_files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for file in &schema_files {
        let path = file
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("<unknown-path>");
        let launders = file
            .get("cited_adrs")
            .map(&cites_validated)
            .unwrap_or(false);
        if launders {
            continue;
        }
        findings.insert(Finding::new(
            "NGQL-SCHEMA-FILE",
            path,
            format!(
                "`{path}` is a GraphQL schema file, which {forbidding} forbids in the owned stack (the canonical API surface is REST + gRPC + AsyncAPI + realtime). Remove the file. GraphQL is admissible ONLY via a future ADR that explicitly reverses {forbidding}; such an ADR must be Accepted in docs/decisions AND enumerated in the gate policy `authorizing_adrs` allowlist, after which citing its id in the schema file (or a sibling `{path}.adr` marker) authorizes it."
            ),
        ));
    }

    // --- BUCK build graph globs that still admit GraphQL schema files ---
    let build_graph_schema_globs = observed
        .get("build_graph_schema_globs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for entry in &build_graph_schema_globs {
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("<unknown-buck>");
        let line = entry.get("line").and_then(Value::as_u64).unwrap_or(0);
        let glob = entry
            .get("glob")
            .and_then(Value::as_str)
            .unwrap_or("**/*.graphql");
        let launders = entry
            .get("cited_adrs")
            .map(&cites_validated)
            .unwrap_or(false);
        if launders {
            continue;
        }
        findings.insert(Finding::new(
            "NGQL-BUILD-GRAPH-SCHEMA-GLOB",
            &format!("{path}:{line}:{glob}"),
            format!(
                "`{path}:{line}` keeps the GraphQL schema glob `{glob}` in the Buck2 build graph, which {forbidding} forbids for the owned stack. Remove the glob so generated or hand-authored GraphQL schema files cannot become normal build inputs without a future ADR that explicitly reverses {forbidding}."
            ),
        ));
    }

    // --- forbidden GraphQL crates in the resolved Cargo.lock graph (transitive catch) ---
    if let Some(lock) = observed.get("lock") {
        let present = lock
            .get("present")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if present {
            let lock_launders = lock
                .get("cited_adrs")
                .map(&cites_validated)
                .unwrap_or(false);
            let packages = lock
                .get("packages")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().filter_map(Value::as_str).collect::<Vec<_>>())
                .unwrap_or_default();
            for pkg in packages {
                let Some(rule) = crate_rules.iter().find(|rule| rule.matches(pkg)) else {
                    continue;
                };
                if lock_launders {
                    continue;
                }
                let match_note = if rule.prefix {
                    format!(" (matched the forbidden prefix `{}`)", rule.name)
                } else {
                    String::new()
                };
                findings.insert(Finding::new(
                    "NGQL-LOCK-FORBIDDEN",
                    &format!("Cargo.lock:{pkg}"),
                    format!(
                        "the resolved `Cargo.lock` graph contains the GraphQL crate `{pkg}`{match_note}, which {forbidding} forbids in the owned stack — even though no manifest names it directly, it is pulled in transitively. Remove the dependency edge that introduces it (run a fresh resolve after dropping it). GraphQL is admissible ONLY via a future ADR that explicitly reverses {forbidding}, Accepted in docs/decisions AND enumerated in the gate policy `authorizing_adrs` allowlist."
                    ),
                ));
            }
        }
    }

    findings
}

/// Bare-code projection of [`evaluate_keyed`]; the single source of truth for the verdict.
pub fn evaluate(policy: &Value, observed: &Value) -> Report {
    Report::from_findings(&evaluate_keyed(policy, observed))
}

/// Human-readable render of the findings. Never a bare FAIL — every finding prints its detail.
pub fn render_findings(findings: &BTreeSet<Finding>) -> String {
    if findings.is_empty() {
        return "no-graphql-without-adr gate passed: the candidate tree carries no GraphQL library, schema file, nor build-graph schema glob (the owned stack is GraphQL-free — ADR-0565)".to_owned();
    }
    let mut out = String::from("no-graphql-without-adr gate failed (ADR-0565):\n");
    for finding in findings {
        out.push_str(&format!(
            "    - {} {}\n        {}\n",
            finding.code, finding.key, finding.detail
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> Value {
        json!({
            "gate_id": GATE_ID,
            "forbidding_adr": "ADR-0565",
            "min_expected_workspace_members": 1,
            "forbidden_crates": [
                {"crate": "async-graphql", "match": "prefix"},
                {"crate": "async_graphql", "match": "prefix"},
                {"crate": "juniper", "match": "exact"},
                {"crate": "graphql-parser", "match": "exact"},
                {"crate": "graphql_parser", "match": "exact"},
                {"crate": "cynic", "match": "exact"},
                {"crate": "apollo-", "match": "prefix"}
            ],
            "schema_extensions": ["graphql", "graphqls", "gql", "gqls", "sdl"],
            "authorizing_adrs": [],
            "excluded_dirs": ["third-party/"]
        })
    }

    /// As [`policy`] but with `authorizing_adrs` carrying `ADR-0700` (the test-only authorizing id).
    /// The pure evaluator trusts `observed.valid_authorizing_adrs` for the escape-hatch; this policy
    /// is what a repo would ship after an ADR-0700 reversal was Accepted and allowlisted.
    fn policy_allowing(ids: &[&str]) -> Value {
        let mut p = policy();
        p["authorizing_adrs"] = json!(ids);
        p
    }

    /// The full observed shape. `valid` is the set the COLLECTOR validated (allowlist ∩ real
    /// Accepted-reversing ADR); the pure evaluator launders an artifact only if it cites one of these.
    fn observed_full(
        members: u64,
        valid: &[&str],
        manifests: Value,
        schema_files: Value,
        lock: Value,
    ) -> Value {
        json!({
            "workspace_members_found": members,
            "valid_authorizing_adrs": valid,
            "manifests": manifests,
            "schema_files": schema_files,
            "lock": lock,
        })
    }

    /// Convenience: no validated authorizing ids, no lock packages.
    fn observed(members: u64, manifests: Value, schema_files: Value) -> Value {
        observed_full(
            members,
            &[],
            manifests,
            schema_files,
            json!({"present": false, "packages": [], "cited_adrs": []}),
        )
    }

    #[test]
    fn green_on_a_clean_tree() {
        // The post-drop tree: no GraphQL lib, no schema file. The gate PASSES.
        let report = evaluate(
            &policy(),
            &observed(
                500,
                json!([{"member_path": "cloud/iam/pdp", "deps": ["serde", "tokio"], "cited_adrs": []}]),
                json!([]),
            ),
        );
        assert_eq!(report.verdict, Verdict::Green, "clean tree ⇒ green");
        assert!(report.violations.is_empty());
        assert!(
            render_findings(&evaluate_keyed(
                &policy(),
                &observed(500, json!([]), json!([]))
            ))
            .contains("passed")
        );
    }

    #[test]
    fn red_when_a_cargo_toml_adds_async_graphql() {
        let observed = observed(
            500,
            json!([{"member_path": "oya/studio/graphql", "deps": ["async-graphql", "serde"], "cited_adrs": []}]),
            json!([]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        let f = findings
            .iter()
            .find(|f| f.code == "NGQL-FORBIDDEN-LIB")
            .unwrap_or_else(|| panic!("async-graphql must be RED: {findings:?}"));
        assert_eq!(f.key, "oya/studio/graphql/Cargo.toml:async-graphql");
        assert!(
            f.detail.contains("ADR-0565"),
            "remediation must name the forbidding ADR: {f:?}"
        );
        assert!(
            f.detail.contains("Remove the dependency"),
            "remediation must say how to fix: {f:?}"
        );
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn red_on_a_prefixed_apollo_crate() {
        let observed = observed(
            500,
            json!([{"member_path": "cloud/gw", "deps": ["apollo-router"], "cited_adrs": []}]),
            json!([]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings
                .iter()
                .any(|f| f.code == "NGQL-FORBIDDEN-LIB" && f.key.ends_with("apollo-router")),
            "a prefixed apollo-* crate must be RED: {findings:?}"
        );
    }

    #[test]
    fn red_when_a_graphql_schema_file_is_present() {
        let observed = observed(
            500,
            json!([]),
            json!([{"path": "oya/analytics/contracts/graphql-v1.sdl", "cited_adrs": []}]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        let f = findings
            .iter()
            .find(|f| f.code == "NGQL-SCHEMA-FILE")
            .unwrap_or_else(|| panic!("a .sdl schema file must be RED: {findings:?}"));
        assert_eq!(f.key, "oya/analytics/contracts/graphql-v1.sdl");
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn red_when_buck_keeps_a_graphql_schema_glob() {
        let mut observed = observed(500, json!([]), json!([]));
        observed["build_graph_schema_globs"] = json!([{
            "path": "cloud/gw/BUCK",
            "line": 3,
            "glob": "**/*.graphql",
            "cited_adrs": []
        }]);
        let findings = evaluate_keyed(&policy(), &observed);
        let f = findings
            .iter()
            .find(|f| f.code == "NGQL-BUILD-GRAPH-SCHEMA-GLOB")
            .unwrap_or_else(|| panic!("a GraphQL schema glob in BUCK must be RED: {findings:?}"));
        assert_eq!(f.key, "cloud/gw/BUCK:3:**/*.graphql");
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn green_when_a_forbidden_lib_cites_a_validated_authorizing_adr() {
        // The escape-hatch: a manifest citing an ALLOWLISTED + VALIDATED reversing ADR is allowed.
        // The collector put ADR-0700 in valid_authorizing_adrs (allowlist ∩ real Accepted reversal).
        let observed = observed_full(
            500,
            &["ADR-0700"],
            json!([{"member_path": "cloud/gw", "deps": ["async-graphql"], "cited_adrs": ["ADR-0700"]}]),
            json!([]),
            json!({"present": false, "packages": [], "cited_adrs": []}),
        );
        let findings = evaluate_keyed(&policy_allowing(&["ADR-0700"]), &observed);
        assert!(
            !findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB"),
            "a validated-ADR-cited GraphQL lib must be allowed: {findings:?}"
        );
        assert_eq!(
            evaluate(&policy_allowing(&["ADR-0700"]), &observed).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn green_when_a_schema_file_cites_a_validated_authorizing_adr() {
        let observed = observed_full(
            500,
            &["ADR-0700"],
            json!([]),
            json!([{"path": "oya/analytics/contracts/graphql-v2.graphql", "cited_adrs": ["ADR-0700"]}]),
            json!({"present": false, "packages": [], "cited_adrs": []}),
        );
        assert_eq!(
            evaluate(&policy_allowing(&["ADR-0700"]), &observed).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn red_when_a_lib_cites_an_unvalidated_adr_backdoor() {
        // CRITICAL backdoor (the adversarial-review finding): citing an arbitrary/fabricated ADR token
        // must NOT launder. The id is NOT in valid_authorizing_adrs (empty allowlist), so the gate is
        // RED even though the manifest "cites" ADR-9999.
        let observed = observed(
            500,
            json!([{"member_path": "cloud/gw", "deps": ["async-graphql"], "cited_adrs": ["ADR-9999"]}]),
            json!([]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.iter().any(|f| f.code == "NGQL-FORBIDDEN-LIB"),
            "citing an unvalidated/fabricated ADR must NOT launder: {findings:?}"
        );
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn red_when_a_schema_cites_an_unvalidated_adr_backdoor() {
        let observed = observed(
            500,
            json!([]),
            json!([{"path": "api.graphql", "cited_adrs": ["ADR-1234"]}]),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        assert!(
            findings.iter().any(|f| f.code == "NGQL-SCHEMA-FILE"),
            "a schema citing an unvalidated ADR must NOT launder: {findings:?}"
        );
    }

    #[test]
    fn mentioning_only_the_forbidding_adr_does_not_self_launder() {
        // A file that cites ONLY the forbidding ADR (ADR-0565 — the rule it would be violating) does
        // NOT escape: cited_authorizing_adrs filters the forbidding id out.
        assert!(cited_authorizing_adrs("see ADR-0565", Some("ADR-0565")).is_empty());
        // Citing a DIFFERENT (reversing) id is collected (but still must be VALIDATED to launder).
        assert!(
            cited_authorizing_adrs(
                "reintroduced per ADR-0700 reversing ADR-0565",
                Some("ADR-0565")
            )
            .contains("ADR-0700")
        );
    }

    #[test]
    fn red_when_lock_carries_a_forbidden_crate_transitively() {
        // A forbidden crate present ONLY in the resolved Cargo.lock graph (no manifest names it
        // directly) must be RED — the transitive-reintroduction catch.
        let observed = observed_full(
            500,
            &[],
            json!([{"member_path": "cloud/gw", "deps": ["serde"], "cited_adrs": []}]),
            json!([]),
            json!({"present": true, "packages": ["serde", "tokio", "async-graphql"], "cited_adrs": []}),
        );
        let findings = evaluate_keyed(&policy(), &observed);
        let f = findings
            .iter()
            .find(|f| f.code == "NGQL-LOCK-FORBIDDEN")
            .unwrap_or_else(|| panic!("a forbidden lock crate must be RED: {findings:?}"));
        assert_eq!(f.key, "Cargo.lock:async-graphql");
        assert_eq!(evaluate(&policy(), &observed).verdict, Verdict::Red);
    }

    #[test]
    fn lock_leg_honors_a_validated_authorizing_adr() {
        // When the lock itself cites a validated authorizing ADR (rare; e.g. a generated lock header),
        // the lock leg laundering applies — proving the escape-hatch is consistent across legs.
        let observed = observed_full(
            500,
            &["ADR-0700"],
            json!([]),
            json!([]),
            json!({"present": true, "packages": ["async-graphql"], "cited_adrs": ["ADR-0700"]}),
        );
        assert_eq!(
            evaluate(&policy_allowing(&["ADR-0700"]), &observed).verdict,
            Verdict::Green
        );
    }

    #[test]
    fn empty_scan_fails_closed() {
        let findings = evaluate_keyed(&policy(), &observed(0, json!([]), json!([])));
        assert!(
            findings.iter().any(|f| f.code == "NGQL-EMPTY-SCAN"),
            "a below-floor member census must trip NGQL-EMPTY-SCAN: {findings:?}"
        );
    }

    #[test]
    fn policy_gate_id_mismatch_fails_closed() {
        let mut p = policy();
        p["gate_id"] = Value::from("wrong-id");
        let findings = evaluate_keyed(&p, &observed(500, json!([]), json!([])));
        assert!(
            findings
                .iter()
                .any(|f| f.code == "NGQL-POLICY-GATE-ID-MISMATCH")
        );
    }

    #[test]
    fn malformed_policy_with_no_forbidden_list_fails_closed() {
        let p = json!({ "gate_id": GATE_ID, "schema_extensions": ["graphql"] });
        let findings = evaluate_keyed(&p, &observed(500, json!([]), json!([])));
        assert!(
            findings.iter().any(|f| f.code == "NGQL-POLICY-MALFORMED"),
            "a missing forbidden_crates list must fail closed: {findings:?}"
        );
    }

    #[test]
    fn malformed_policy_with_no_schema_extensions_fails_closed() {
        let p = json!({
            "gate_id": GATE_ID,
            "forbidden_crates": [{"crate": "juniper", "match": "exact"}]
        });
        let findings = evaluate_keyed(&p, &observed(500, json!([]), json!([])));
        assert!(findings.iter().any(|f| f.code == "NGQL-POLICY-MALFORMED"));
    }

    #[test]
    fn parse_manifest_dep_names_covers_all_tables_and_renames() {
        let manifest = r#"
[package]
name = "x"
[dependencies]
serde = "1"
gql = { package = "async-graphql", version = "7" }
[dev-dependencies]
juniper = "0.16"
[build-dependencies]
graphql-parser = "0.4"
[target.'cfg(unix)'.dependencies]
cynic = "3"
"#;
        let names = parse_manifest_dep_names(manifest);
        // The rename is denied on the REAL crate name.
        assert!(
            names.contains("async-graphql"),
            "rename must resolve to real name: {names:?}"
        );
        assert!(names.contains("juniper"));
        assert!(names.contains("graphql-parser"));
        assert!(names.contains("cynic"));
        assert!(names.contains("serde"));
        // The local rename key is NOT what we deny on.
        assert!(!names.contains("gql"));
    }

    #[test]
    fn workspace_dependency_rename_resolves_to_real_name() {
        // CRITICAL backdoor: a forbidden lib smuggled via the root [workspace.dependencies] rename and
        // a member `{ workspace = true }` inheritance must resolve to its REAL name.
        let root = r#"
[workspace]
members = ["crates/*"]
[workspace.dependencies]
gqlrt = { package = "async-graphql", version = "7" }
serde = "1"
"#;
        let ws = parse_root_workspace_deps(root);
        let member = r#"
[package]
name = "gw"
[dependencies]
gqlrt = { workspace = true }
serde = { workspace = true }
"#;
        let names = parse_manifest_dep_names_with_workspace(member, &ws);
        assert!(
            names.contains("async-graphql"),
            "{{ workspace = true }} on a renamed workspace dep must resolve to the real name: {names:?}"
        );
        assert!(names.contains("serde"));
        assert!(
            !names.contains("gqlrt"),
            "the workspace rename key must NOT be denied on: {names:?}"
        );
    }

    #[test]
    fn parse_lock_package_names_collects_every_package() {
        let lock = r#"
version = 4
[[package]]
name = "serde"
version = "1.0.150"
[[package]]
name = "async-graphql"
version = "7.0.0"
"#;
        let names = parse_lock_package_names(lock);
        assert!(names.contains("serde"));
        assert!(names.contains("async-graphql"));
    }

    #[test]
    fn adr_status_accepted_requires_a_ratified_value() {
        assert!(adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Accepted\nsupersedes:\n  - ADR-0565\n---\nThis ADR reverses ADR-0565.\n",
            "ADR-0565"
        ));
        // A Proposed ADR does not launder even if it claims to reverse the ban.
        assert!(!adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Proposed\n---\nThis ADR reverses ADR-0565.\n",
            "ADR-0565"
        ));
        // An in-place amendment remains live for planning, but cannot newly authorize GraphQL.
        assert!(!adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Amended\nsupersedes:\n  - ADR-0565\n---\n",
            "ADR-0565"
        ));
        // Conflicting duplicate lifecycle keys cannot launder an authorization.
        assert!(!adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Accepted\nstatus: Amended\nsupersedes:\n  - ADR-0565\n---\n",
            "ADR-0565"
        ));
        // An Accepted ADR that does not mention the forbidding id does not launder.
        assert!(!adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Accepted\n---\nUnrelated decision.\n",
            "ADR-0565"
        ));
        // A pipe-delimited status template is not a ratified value.
        assert!(!adr_is_accepted_and_reverses(
            "---\nstatus: Proposed | Accepted | Superseded\n---\nreverses ADR-0565\n",
            "ADR-0565"
        ));
    }

    #[test]
    fn adr_citations_requires_four_digits() {
        let cited = adr_citations("ADR-0565 and ADR-12 and ADR-0700 and ADR-1234");
        assert!(cited.contains("ADR-0565"));
        assert!(cited.contains("ADR-0700"));
        assert!(cited.contains("ADR-1234"));
        // Fewer than four digits is not a canonical citation.
        assert!(!cited.contains("ADR-12"));
    }

    #[test]
    fn typo_near_forbidding_id_does_not_launder() {
        // `ADR-05650` (a typo of the forbidding ADR-0565) is a DIFFERENT id; it is collected as a
        // citation but is not in the validated set, so it cannot launder.
        let cited = cited_authorizing_adrs("authorized per ADR-05650", Some("ADR-0565"));
        assert!(
            cited.contains("ADR-05650"),
            "the typo is a distinct citation: {cited:?}"
        );
        let observed = observed(
            500,
            json!([{"member_path": "cloud/gw", "deps": ["async-graphql"], "cited_adrs": ["ADR-05650"]}]),
            json!([]),
        );
        assert_eq!(
            evaluate(&policy(), &observed).verdict,
            Verdict::Red,
            "a typo id near the forbidding ADR must NOT launder"
        );
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let obs = observed(
            500,
            json!([{"member_path": "a", "deps": ["juniper"], "cited_adrs": []}]),
            json!([{"path": "b.graphql", "cited_adrs": []}]),
        );
        let projected: BTreeSet<String> = evaluate_keyed(&policy(), &obs)
            .into_iter()
            .map(|f| f.code)
            .collect();
        assert_eq!(evaluate(&policy(), &obs).violations, projected);
    }

    // --- Fix 1: malformed-TOML fail-closed ---

    #[test]
    fn malformed_manifest_with_forbidden_crate_is_red() {
        // Fix 1 RED: a Cargo.toml that fails to parse as TOML but contains "async-graphql" must
        // NOT pass GREEN — the raw-text fallback must flag it.
        let malformed = "THIS IS NOT TOML !!!\nasync-graphql = garbage [[\n";
        let ws = WorkspaceDeps::default();
        let names = parse_manifest_dep_names_with_workspace(malformed, &ws);
        assert!(
            names.contains("async-graphql"),
            "malformed TOML with async-graphql in raw text must produce the name (fail-closed): {names:?}"
        );
    }

    #[test]
    fn malformed_manifest_without_forbidden_crate_is_green() {
        // Fix 1 GREEN: a Cargo.toml that fails to parse but contains NO forbidden crate name must
        // remain GREEN — the raw-text fallback must not over-block clean content.
        let malformed = "THIS IS NOT TOML !!!\nserde = garbage [[\n";
        let ws = WorkspaceDeps::default();
        let names = parse_manifest_dep_names_with_workspace(malformed, &ws);
        assert!(
            !names.contains("async-graphql"),
            "malformed TOML without async-graphql must not produce a false positive: {names:?}"
        );
    }

    #[test]
    fn malformed_lock_with_forbidden_crate_is_red() {
        // Fix 1 RED: a Cargo.lock that fails to parse as TOML but contains a `name = "async-graphql"`
        // line must NOT pass GREEN — the raw lock fallback must flag it.
        let malformed = "version = BROKEN\n\n[[package]]\nname = \"async-graphql\"\nversion = \"7.0.0\"\n[[BROKEN\n";
        let names = parse_lock_package_names(malformed);
        assert!(
            names.contains("async-graphql"),
            "malformed Cargo.lock with async-graphql name line must produce the name (fail-closed): {names:?}"
        );
    }

    #[test]
    fn malformed_lock_without_forbidden_crate_is_green() {
        // Fix 1 GREEN: a Cargo.lock that fails to parse but has no forbidden crate name must remain GREEN.
        let malformed =
            "version = BROKEN\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\n[[BROKEN\n";
        let names = parse_lock_package_names(malformed);
        assert!(
            !names.contains("async-graphql"),
            "malformed Cargo.lock without async-graphql must not produce a false positive: {names:?}"
        );
    }

    // --- Fix 3: adr_status_is_accepted bounded to frontmatter ---

    #[test]
    fn status_in_code_block_does_not_validate() {
        // Fix 3 RED: a `status: Accepted` line INSIDE a fenced code block must NOT satisfy the
        // frontmatter check — only the first `---`…`---` block is authoritative.
        let body = "---\nid: ADR-0700\nstatus: Proposed\n---\n\n```yaml\nstatus: Accepted\n```\n";
        assert!(
            !adr_status_is_accepted(body),
            "status: Accepted inside a code block must NOT validate: body={body:?}"
        );
    }

    #[test]
    fn status_in_prose_does_not_validate() {
        // Fix 3 RED: a `status: Accepted` line in the prose body (after the closing `---`) must
        // NOT satisfy the frontmatter check.
        let body = "---\nid: ADR-0700\nstatus: Proposed\n---\n\nstatus: Accepted\n";
        assert!(
            !adr_status_is_accepted(body),
            "status: Accepted in prose must NOT validate: body={body:?}"
        );
    }

    #[test]
    fn quoted_status_accepted_validates() {
        // Fix 3 GREEN: `status: "Accepted"` (quoted value) must validate.
        let body = "---\nid: ADR-0700\nstatus: \"Accepted\"\n---\n";
        assert!(
            adr_status_is_accepted(body),
            "quoted status: \"Accepted\" must validate: body={body:?}"
        );
    }

    #[test]
    fn single_quoted_status_accepted_validates() {
        let body = "---\nid: ADR-0700\nstatus: 'Accepted'\n---\n";
        assert!(
            adr_status_is_accepted(body),
            "single-quoted status: 'Accepted' must validate: body={body:?}"
        );
    }

    #[test]
    fn status_accepted_case_insensitive_validates() {
        // Case-insensitive: `status: accepted` (lowercase) must validate.
        let body = "---\nid: ADR-0700\nstatus: accepted\n---\n";
        assert!(
            adr_status_is_accepted(body),
            "lowercase 'accepted' must validate"
        );
    }

    // --- Fix 4: adr_is_accepted_and_reverses requires structural supersedes in frontmatter ---

    #[test]
    fn forbidding_id_only_in_related_does_not_validate() {
        // Fix 4 RED: an ADR that lists ADR-0565 only under `related:` (not supersedes/amends/reverses)
        // must NOT validate as a reversal — even if the body also says "has not been superseded".
        let body = "---\nid: ADR-0700\nstatus: Accepted\nrelated:\n  - ADR-0565\n---\n\nADR-0565 has not been superseded by this decision.\n";
        assert!(
            !adr_is_accepted_and_reverses(body, "ADR-0565"),
            "forbidding id only in related: must NOT validate as reversal: body={body:?}"
        );
    }

    #[test]
    fn not_superseded_phrase_in_body_does_not_validate() {
        // Fix 4 RED: a body containing "superseded" only as part of "has not been superseded" must
        // NOT satisfy the reversal requirement (the supersedes: field is not present).
        let body = "---\nid: ADR-0700\nstatus: Accepted\n---\n\nThis ADR has NOT superseded ADR-0565; ADR-0565 remains in effect.\n";
        assert!(
            !adr_is_accepted_and_reverses(body, "ADR-0565"),
            "body-only superseded mention must NOT validate: body={body:?}"
        );
    }

    #[test]
    fn frontmatter_supersedes_list_validates() {
        // Fix 4 GREEN: frontmatter `supersedes:\n  - ADR-0565` validates correctly.
        let body = "---\nid: ADR-0700\nstatus: Accepted\nsupersedes:\n  - ADR-0565\n---\n";
        assert!(
            adr_is_accepted_and_reverses(body, "ADR-0565"),
            "supersedes list must validate"
        );
    }

    #[test]
    fn frontmatter_amends_field_validates() {
        // Fix 4 GREEN: `amends: ADR-0565` (inline scalar) validates.
        let body = "---\nid: ADR-0700\nstatus: Accepted\namends: ADR-0565\n---\n";
        assert!(
            adr_is_accepted_and_reverses(body, "ADR-0565"),
            "amends scalar must validate"
        );
    }

    #[test]
    fn frontmatter_reverses_field_validates() {
        // Fix 4 GREEN: `reverses:\n  - ADR-0565` validates.
        let body = "---\nid: ADR-0700\nstatus: Accepted\nreverses:\n  - ADR-0565\n---\n";
        assert!(
            adr_is_accepted_and_reverses(body, "ADR-0565"),
            "reverses list must validate"
        );
    }

    #[test]
    fn frontmatter_supersedes_inline_scalar_validates() {
        // Fix 4 GREEN: `supersedes: ADR-0565` (inline scalar, no list) validates.
        let body = "---\nid: ADR-0700\nstatus: Accepted\nsupersedes: ADR-0565\n---\n";
        assert!(
            adr_is_accepted_and_reverses(body, "ADR-0565"),
            "supersedes inline scalar must validate"
        );
    }

    #[test]
    fn existing_adr_status_accepted_test_still_passes() {
        // Regression: the existing adr_status_accepted_requires_a_ratified_value test shape
        // with frontmatter-bounded check.
        assert!(adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Accepted\nsupersedes:\n  - ADR-0565\n---\nThis ADR reverses ADR-0565.\n",
            "ADR-0565"
        ));
        // Proposed + supersedes list → still fails (wrong status).
        assert!(!adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Proposed\nsupersedes:\n  - ADR-0565\n---\n",
            "ADR-0565"
        ));
        // Accepted + no reversal field → fails.
        assert!(!adr_is_accepted_and_reverses(
            "---\nid: ADR-0700\nstatus: Accepted\n---\nUnrelated decision.\n",
            "ADR-0565"
        ));
    }
}
