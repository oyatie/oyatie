//! # oya-crate-registrar-app (ADR-0568, G011 slice 2)
//!
//! The I/O / adapter half of `register_crate`: the thin WRITERS that APPLY the kernel's typed
//! [`Edit`](oya_crate_registrar_kernel::Edit)s to the born-accounting SSOTs on disk. The pure
//! [`oya-crate-registrar-kernel`](oya_crate_registrar_kernel) computes an ordered
//! [`RegistrationPlan`](oya_crate_registrar_kernel::RegistrationPlan); this crate turns three of
//! those edits — the three NEW SSOT shapes the kernel introduces — into byte-stable file content.
//! (The other edits reuse the producer's existing `--fix-owners`/`--fix-reachability`/`--next-adr`
//! bridges and the workspace-member glob; those are wired in slice 3, not here.)
//!
//! ## The three writers
//! - [`capability_mapping`] — [`Edit::CapabilityMapping`](oya_crate_registrar_kernel::Edit::CapabilityMapping):
//!   upsert `<crate_dir>` into the matching `globs` list of `specs/capability-registry.json`'s
//!   `membership_lint_coverage.absorbs_current_crate_globs` (closed-set validated — an unknown
//!   capability/meta-dir slug is refused), re-serialized with the repo's canonical-JSON form so the
//!   registry stays byte-stable.
//! - [`adr_governed_paths`] — [`Edit::AdrGovernedPathAppend`](oya_crate_registrar_kernel::Edit::AdrGovernedPathAppend):
//!   upsert the VERBATIM tracked paths into the owning ADR's `## Governed surfaces` fenced block
//!   (sorted, deduped, literal — the #66 format the producer's `resolve_justifications` credits).
//!   Creates the block if absent.
//! - [`catalog_yaml`] — [`Edit::CatalogYaml`](oya_crate_registrar_kernel::Edit::CatalogYaml):
//!   render `registry/catalog/<leaf>.yaml` (schema-driven; the human-supplied `slo`/`plane` are
//!   required — never silently defaulted, per ADR-0548 D2).
//!
//! ## Pure-compute-then-tiny-apply + idempotent
//! Every writer is split into a PURE `compute_*` function that takes the current file content (or
//! `None` when absent) plus the edit and returns the new content as a `String` — testable with no
//! filesystem — and a thin `apply_*` that reads the file, calls `compute_*`, and writes ONLY when
//! the bytes change. Applying a writer to already-correct state changes nothing (byte-identical):
//! the compute is a deterministic upsert, so `compute(compute(x)) == compute(x)`.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};

// ───────────────────────────── canonical JSON (byte-stable) ─────────────────────────────

/// Serialize `value` to the repo's canonical on-disk JSON form: keys recursively sorted, 2-space
/// pretty, single trailing newline. This is byte-identical to the producer's
/// `accounting-registry::to_canonical_json` (accounting-registry-app `lib.rs:892`) — the same
/// canonical form keeps `specs/capability-registry.json` byte-stable across cargo and buck2 (the
/// explicit key-sort makes the bytes independent of serde_json's `preserve_order` feature, which
/// reindeer unions ON under buck2). It is reimplemented here rather than importing the heavyweight
/// producer (a 12-gate dependency tree) because the algorithm is small and the coupling would be
/// inappropriate for a thin writer crate.
///
/// # Errors
/// Returns [`WriterError::Serialize`] if the value cannot be serialized (e.g. a non-string map key,
/// which `serde_json::Value` never produces from parsed JSON).
pub fn to_canonical_json(value: &Value) -> Result<String, WriterError> {
    let mut text = serde_json::to_string_pretty(&canonicalize_value(value))
        .map_err(|e| WriterError::Serialize(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

/// Recursively reorder every object's keys into sorted (Unicode-scalar / UTF-8-byte) order. Mirrors
/// the producer's `canonicalize_value`; arrays keep element order, scalars are unchanged.
fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted: std::collections::BTreeMap<String, Value> =
                std::collections::BTreeMap::new();
            for (key, val) in map {
                sorted.insert(key.clone(), canonicalize_value(val));
            }
            let mut out = Map::new();
            for (key, val) in sorted {
                out.insert(key, val);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        other => other.clone(),
    }
}

// ───────────────────────────── writer errors ─────────────────────────────

/// Why a writer refused. Every writer fails CLOSED: an invalid edit or unparseable current file
/// yields a typed error, never a partial/silent write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriterError {
    /// The current `specs/capability-registry.json` could not be parsed as JSON.
    RegistryParse(String),
    /// The registry JSON did not have the expected
    /// `membership_lint_coverage.absorbs_current_crate_globs` array shape.
    RegistryShape(String),
    /// The capability/meta-dir slug has no existing group in the closed registry (fail-closed: a
    /// writer never invents a new capability group).
    UnknownCapability(String),
    /// A governed path contained brace-glob syntax (`{`/`}`) — the #66 trap. The kernel already
    /// rejects these; the writer re-checks so it can never be invoked with an unmatched token.
    BraceGlobInGovernedPath(String),
    /// A catalog render was requested with an empty `slo` or `plane` (never silently defaulted).
    MissingCatalogField(String),
    /// A JSON value could not be serialized to canonical form.
    Serialize(String),
    /// A filesystem read/write failed.
    Io(String),
}

impl std::fmt::Display for WriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriterError::RegistryParse(m) => write!(f, "capability-registry parse error: {m}"),
            WriterError::RegistryShape(m) => write!(f, "capability-registry shape error: {m}"),
            WriterError::UnknownCapability(c) => {
                write!(f, "unknown capability/meta-dir slug (closed set): {c}")
            }
            WriterError::BraceGlobInGovernedPath(p) => {
                write!(f, "brace-glob in governed path (the #66 trap): {p}")
            }
            WriterError::MissingCatalogField(field) => {
                write!(f, "catalog field is required and must not be empty: {field}")
            }
            WriterError::Serialize(m) => write!(f, "canonical-json serialize error: {m}"),
            WriterError::Io(m) => write!(f, "writer io: {m}"),
        }
    }
}

impl std::error::Error for WriterError {}

/// True iff `path` contains brace-glob syntax (`{` or `}`) — the #66 trap.
fn has_brace_glob(path: &str) -> bool {
    path.contains('{') || path.contains('}')
}

/// The crate leaf (last path component) of a repo-relative dir.
fn crate_leaf(crate_dir: &str) -> &str {
    let trimmed = crate_dir.trim_end_matches('/');
    trimmed.rsplit('/').next().unwrap_or(trimmed)
}

// ───────────────────────────── 1. CapabilityMappingWriter ─────────────────────────────

/// The `membership_lint_coverage.absorbs_current_crate_globs` group key a crate dir maps to. A
/// group is keyed by EITHER `meta_dir` (e.g. `build/`, `governance/`) OR `capability` (e.g. `data`,
/// `messaging`); the kernel's `capability` field carries whichever slug the human chose, so the
/// writer matches against both key shapes.
pub mod capability_mapping {
    use super::{Path, Value, WriterError, fs, to_canonical_json};

    /// The repo-relative path of the closed capability registry.
    pub const REGISTRY_PATH: &str = "specs/capability-registry.json";

    /// Compute the new `specs/capability-registry.json` content for upserting `crate_dir` into the
    /// `globs` list of the group whose `meta_dir` OR `capability` equals `slug`. PURE — no I/O.
    ///
    /// Idempotent upsert: if `crate_dir` is already in the matching group's `globs`, the returned
    /// bytes are byte-identical to the canonical re-serialization of `current` (re-apply = no-op).
    /// Fail-closed: a `slug` with no existing group is [`WriterError::UnknownCapability`] (a writer
    /// never invents a capability group), and a malformed registry is a typed shape/parse error.
    ///
    /// # Errors
    /// [`WriterError::RegistryParse`]/[`RegistryShape`](WriterError::RegistryShape)/
    /// [`UnknownCapability`](WriterError::UnknownCapability)/[`Serialize`](WriterError::Serialize).
    pub fn compute(current: &str, crate_dir: &str, slug: &str) -> Result<String, WriterError> {
        let mut root: Value = serde_json::from_str(current)
            .map_err(|e| WriterError::RegistryParse(e.to_string()))?;

        let groups = root
            .get_mut("membership_lint_coverage")
            .and_then(|m| m.get_mut("absorbs_current_crate_globs"))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                WriterError::RegistryShape(
                    "membership_lint_coverage.absorbs_current_crate_globs is not an array"
                        .to_owned(),
                )
            })?;

        let mut matched = false;
        for group in groups.iter_mut() {
            let is_match = group_slug(group).is_some_and(|s| s == slug);
            if !is_match {
                continue;
            }
            matched = true;
            let globs = group
                .get_mut("globs")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| {
                    WriterError::RegistryShape(format!("group {slug} has no `globs` array"))
                })?;
            // Upsert: insert only if absent, then keep the list sorted + deduped so the output is
            // canonical regardless of insertion order (byte-stable re-apply).
            let already = globs
                .iter()
                .any(|g| g.as_str() == Some(crate_dir));
            if !already {
                globs.push(Value::String(crate_dir.to_owned()));
            }
            sort_dedup_string_array(globs);
            break;
        }

        if !matched {
            return Err(WriterError::UnknownCapability(slug.to_owned()));
        }

        to_canonical_json(&root)
    }

    /// The slug a group is keyed by: its `meta_dir` if present, else its `capability`.
    fn group_slug(group: &Value) -> Option<&str> {
        group
            .get("meta_dir")
            .and_then(Value::as_str)
            .or_else(|| group.get("capability").and_then(Value::as_str))
    }

    /// Sort a JSON string array in place and drop duplicate strings (canonical, byte-stable order).
    fn sort_dedup_string_array(array: &mut Vec<Value>) {
        let mut strings: Vec<String> = array
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
        strings.sort();
        strings.dedup();
        *array = strings.into_iter().map(Value::String).collect();
    }

    /// Apply the capability mapping to `specs/capability-registry.json` under `repo_root`. Reads the
    /// registry, computes the upserted canonical content, and writes ONLY if the bytes changed.
    /// Returns `true` if the file was rewritten, `false` if it was already correct (idempotent).
    ///
    /// # Errors
    /// [`WriterError::Io`] on a read/write failure, or any [`compute`] error.
    pub fn apply(repo_root: &Path, crate_dir: &str, slug: &str) -> Result<bool, WriterError> {
        let abs = repo_root.join(REGISTRY_PATH);
        let current = fs::read_to_string(&abs)
            .map_err(|e| WriterError::Io(format!("read {REGISTRY_PATH}: {e}")))?;
        let next = compute(&current, crate_dir, slug)?;
        if next == current {
            return Ok(false);
        }
        fs::write(&abs, &next).map_err(|e| WriterError::Io(format!("write {REGISTRY_PATH}: {e}")))?;
        Ok(true)
    }
}

// ───────────────────────────── 2. AdrGovernedPathAppendWriter ─────────────────────────────

/// Upsert verbatim tracked paths into an ADR's `## Governed surfaces` fenced block (the #66 fix).
pub mod adr_governed_paths {
    use super::{BTreeSet, Path, WriterError, fs, has_brace_glob};

    /// The heading that opens the governed-surfaces section.
    const HEADING: &str = "## Governed surfaces";
    /// The fence delimiter for the path block.
    const FENCE: &str = "```";

    /// The repo-relative path of an ADR id's markdown file is resolved by the caller; the conventional
    /// shape is `docs/decisions/<id>-<slug>.md`. [`apply`] takes the full path so the writer stays
    /// repo-neutral about the slug.
    ///
    /// Compute the new ADR markdown content with `paths` upserted into the `## Governed surfaces`
    /// fenced block. PURE — no I/O. The block lists one VERBATIM tracked path per line, sorted +
    /// deduped (the canonical, byte-stable enumeration the producer's `resolve_justifications`
    /// tokenizes and credits). If the heading + fence block is absent it is created, appended after
    /// the existing body (with a single blank-line separator). If present, the existing literal paths
    /// in the block are merged with `paths`, re-sorted, and the block is rewritten in place.
    ///
    /// Idempotent: re-running with paths already present yields byte-identical content.
    ///
    /// # Errors
    /// [`WriterError::BraceGlobInGovernedPath`] if any path carries brace-glob syntax (defensive —
    /// the kernel already rejects these).
    pub fn compute(current: &str, paths: &[String]) -> Result<String, WriterError> {
        for p in paths {
            if has_brace_glob(p) {
                return Err(WriterError::BraceGlobInGovernedPath(p.clone()));
            }
        }

        // The existing literal paths already in the block (if any), plus the new ones.
        let mut all: BTreeSet<String> = existing_block_paths(current).into_iter().collect();
        for p in paths {
            all.insert(p.clone());
        }
        let block_body: String = all
            .iter()
            .map(|p| format!("{p}\n"))
            .collect::<String>();
        let new_block = format!("{HEADING}\n\n{FENCE}\n{block_body}{FENCE}\n");

        match locate_block(current) {
            Some((start, end)) => {
                // Replace the existing heading..fence-close span with the rebuilt block. `end` is the
                // byte index just past the closing fence line's trailing newline (or EOF).
                let mut out = String::with_capacity(current.len() + block_body.len());
                out.push_str(&current[..start]);
                out.push_str(&new_block);
                out.push_str(&current[end..]);
                Ok(out)
            }
            None => {
                // Append the block at EOF, separated from the existing body by exactly one blank line.
                let mut out = String::with_capacity(current.len() + new_block.len() + 2);
                out.push_str(current);
                if !current.is_empty() && !current.ends_with('\n') {
                    out.push('\n');
                }
                if !current.is_empty() {
                    out.push('\n');
                }
                out.push_str(&new_block);
                Ok(out)
            }
        }
    }

    /// The verbatim paths already enumerated inside the `## Governed surfaces` fenced block, if the
    /// block exists. Each non-empty trimmed line inside the fence is one path.
    fn existing_block_paths(current: &str) -> Vec<String> {
        let Some((start, end)) = locate_block(current) else {
            return Vec::new();
        };
        let block = &current[start..end];
        // Skip the heading line + the opening fence; collect lines until the closing fence.
        let mut lines = block.lines();
        let mut paths = Vec::new();
        let mut in_fence = false;
        for line in lines.by_ref() {
            let trimmed = line.trim();
            if trimmed == FENCE {
                if in_fence {
                    break; // closing fence
                }
                in_fence = true;
                continue;
            }
            if in_fence && !trimmed.is_empty() {
                paths.push(trimmed.to_owned());
            }
        }
        paths
    }

    /// Locate the `## Governed surfaces` block: returns `(start, end)` byte offsets spanning from the
    /// heading line through the line after the closing fence (or EOF). `None` if there is no heading
    /// followed by an opening fence.
    fn locate_block(current: &str) -> Option<(usize, usize)> {
        let heading_at = find_heading(current)?;
        // The opening fence must be the first fence line at/after the heading.
        let after_heading = &current[heading_at..];
        let rel_open = after_heading.find(FENCE)?;
        let open_at = heading_at + rel_open;
        // The closing fence is the next fence after the opening fence's own line.
        let open_line_end = current[open_at..]
            .find('\n')
            .map(|n| open_at + n + 1)
            .unwrap_or(current.len());
        let rel_close = current[open_line_end..].find(FENCE)?;
        let close_at = open_line_end + rel_close;
        // End span = just past the closing fence line's trailing newline (or EOF).
        let end = current[close_at..]
            .find('\n')
            .map(|n| close_at + n + 1)
            .unwrap_or(current.len());
        Some((heading_at, end))
    }

    /// The byte offset of the `## Governed surfaces` heading line, matched at the start of a line.
    fn find_heading(current: &str) -> Option<usize> {
        if current.starts_with(HEADING) {
            return Some(0);
        }
        let needle = format!("\n{HEADING}");
        current.find(&needle).map(|i| i + 1)
    }

    /// Apply the governed-path append to the ADR markdown at `adr_path` under `repo_root`. Reads the
    /// file, upserts the verbatim paths into the block, and writes ONLY if the bytes changed. Returns
    /// `true` if the file was rewritten, `false` if it was already correct (idempotent).
    ///
    /// # Errors
    /// [`WriterError::Io`] on a read/write failure, or a [`compute`] error.
    pub fn apply(
        repo_root: &Path,
        adr_rel_path: &str,
        paths: &[String],
    ) -> Result<bool, WriterError> {
        let abs = repo_root.join(adr_rel_path);
        let current = fs::read_to_string(&abs)
            .map_err(|e| WriterError::Io(format!("read {adr_rel_path}: {e}")))?;
        let next = compute(&current, paths)?;
        if next == current {
            return Ok(false);
        }
        fs::write(&abs, &next)
            .map_err(|e| WriterError::Io(format!("write {adr_rel_path}: {e}")))?;
        Ok(true)
    }
}

// ───────────────────────────── 3. CatalogYamlWriter ─────────────────────────────

/// Render a crate's `registry/catalog/<leaf>.yaml` record from the human-supplied plane + SLO.
pub mod catalog_yaml {
    use super::{Path, WriterError, crate_leaf, fs};

    /// Compute the `registry/catalog/<leaf>.yaml` content for `crate_dir` with the given `plane` and
    /// `slo`. PURE — no I/O. The record is schema-driven (the practical catalog-record shape the
    /// `cloud-ci-slo-coverage` / `cloud-ci-catalog-liveness` gates parse: a top-level `slo:` scalar
    /// plus the human-decision `plane:` field). Both `plane` and `slo` are REQUIRED — an empty value
    /// is a [`WriterError::MissingCatalogField`] (never silently defaulted, per ADR-0548 D2).
    ///
    /// Deterministic: the same inputs always render byte-identical content (idempotent re-apply).
    ///
    /// # Errors
    /// [`WriterError::MissingCatalogField`] if `plane` or `slo` is empty/blank.
    pub fn compute(crate_dir: &str, plane: &str, slo: &str) -> Result<String, WriterError> {
        if plane.trim().is_empty() {
            return Err(WriterError::MissingCatalogField("plane".to_owned()));
        }
        if slo.trim().is_empty() {
            return Err(WriterError::MissingCatalogField("slo".to_owned()));
        }
        let capability = crate_leaf(crate_dir);
        // The minimal valid catalog record the gates consume: a top-level `slo:` scalar (the
        // slo-coverage / catalog-liveness contract) plus the human-supplied `plane:` and the
        // capability slug derived from the crate leaf. Rendered as canonical YAML (one
        // `key: value` per line, trailing newline) so re-rendering is byte-stable.
        Ok(format!(
            "capability: {capability}\nplane: {plane}\nslo: {slo}\n"
        ))
    }

    /// The repo-relative catalog path for a crate dir: `registry/catalog/<leaf>.yaml`.
    #[must_use]
    pub fn catalog_path(crate_dir: &str) -> String {
        format!("registry/catalog/{}.yaml", crate_leaf(crate_dir))
    }

    /// Apply the catalog render under `repo_root`. Computes the record content and writes ONLY if the
    /// bytes changed (or the file is absent). Returns `true` if the file was written, `false` if it
    /// was already correct (idempotent).
    ///
    /// # Errors
    /// [`WriterError::Io`] on a write failure, or a [`compute`] error.
    pub fn apply(
        repo_root: &Path,
        crate_dir: &str,
        plane: &str,
        slo: &str,
    ) -> Result<bool, WriterError> {
        let rel = catalog_path(crate_dir);
        let abs = repo_root.join(&rel);
        let next = compute(crate_dir, plane, slo)?;
        let current = fs::read_to_string(&abs).ok();
        if current.as_deref() == Some(next.as_str()) {
            return Ok(false);
        }
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| WriterError::Io(format!("create dir for {rel}: {e}")))?;
        }
        fs::write(&abs, &next).map_err(|e| WriterError::Io(format!("write {rel}: {e}")))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
