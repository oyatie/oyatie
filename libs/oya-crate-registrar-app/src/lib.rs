//! # oya-crate-registrar-app (ADR-0568, G011 slice 2)
//!
//! The I/O / adapter half of `register_crate`: the thin WRITERS that APPLY the kernel's typed
//! [`Edit`](oya_crate_registrar_kernel::Edit)s to the born-accounting SSOTs on disk. The pure
//! [`oya-crate-registrar-kernel`](oya_crate_registrar_kernel) computes an ordered
//! [`RegistrationPlan`](oya_crate_registrar_kernel::RegistrationPlan); this crate turns the NEW
//! SSOT-shape edits the kernel introduces into byte-stable file content (the remaining edits reuse
//! the producer's existing `--fix-owners`/`--fix-reachability`/`--next-adr` bridges).
//!
//! ## The four writers
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
//! - [`workspace_member_glob`] — [`Edit::WorkspaceMemberGlob`](oya_crate_registrar_kernel::Edit::WorkspaceMemberGlob):
//!   a coverage VERIFIER, not a mutator. Under ADR-0538 the root `[workspace].members` array is
//!   glob-only (the `cloud-ci-workspace-glob-coverage` gate makes a literal-path member a BLOCKING
//!   violation) and the covering glob for each root is a human ADR decision (ADR-0568 D2). The edit
//!   carries only a `dir` (no glob), so this writer can only CONFIRM coverage (no-op) or REFUSE
//!   (fail-closed) — it never synthesizes a glob. Coverage is resolved purely through
//!   `oya-workspace-members-kernel` (the single source of member-glob semantics), so `compute`
//!   returns `current` byte-unchanged when covered and [`WriterError::WorkspaceMemberUncovered`]
//!   otherwise.
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

use serde_json::Value;

// ───────────────────────────── canonical JSON (byte-stable) ─────────────────────────────

/// Serialize `value` to the repo's canonical on-disk JSON form: 2-space pretty, single trailing
/// newline, **authored key order preserved**.
///
/// `sort_keys` is FALSE. That is the settled one-way-door dialect
/// (`ci/facade/canonical-json/canonical-json-policy.json`, ADR-0546): "sort_keys=false because the
/// defect is rewrite nondeterminism, not key-order ambiguity, and sorting would churn 1452 repo
/// files and destroy intentional order on the agent entry surface". The file this writer edits —
/// `specs/capability-registry.json` — is HAND-AUTHORED governance data whose key order is a design
/// act, so a recursive sort here would silently reorder the whole registry on the next
/// `register_crate` and hang that diff on whoever's PR happened to trigger it.
///
/// This is deliberately NOT the producer's `accounting-registry::to_canonical_json`, which sorts:
/// that one serializes GENERATED faces, where no authored order exists to destroy.
///
/// Byte-stability across cargo and buck2 comes from `serde_json`'s `preserve_order` feature, which
/// this crate's `Cargo.toml` declares explicitly and which reindeer already unions ON for the
/// single generated `third-party//:serde_json`. Both build paths therefore see an insertion-ordered
/// map — without the feature, cargo's default `BTreeMap` backing would sort on PARSE and no
/// serializer could recover the authored order.
///
/// # Errors
/// Returns [`WriterError::Serialize`] if the value cannot be serialized (e.g. a non-string map key,
/// which `serde_json::Value` never produces from parsed JSON).
pub fn to_canonical_json(value: &Value) -> Result<String, WriterError> {
    let mut text =
        serde_json::to_string_pretty(value).map_err(|e| WriterError::Serialize(e.to_string()))?;
    text.push('\n');
    Ok(text)
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
    /// A governed path was not in clean normal form — it contained a newline (`\n`/`\r`), the fence
    /// sequence ```` ``` ````, or leading/trailing whitespace. Such a value would either inject
    /// markdown content into the fenced block or be non-idempotent (emit verbatim, parse trimmed),
    /// so the writer fails CLOSED rather than store it.
    MalformedGovernedPath(String),
    /// The root `Cargo.toml` `[workspace]` manifest could not be parsed into its members/exclude
    /// shape (a malformed manifest fails CLOSED — the member-glob writer never partial-writes).
    WorkspaceManifest(String),
    /// A `WorkspaceMemberGlob` edit named a crate dir that NO existing `[workspace].members` glob
    /// covers. The writer fails CLOSED rather than synthesize a glob: a covering glob is a human
    /// ADR decision (ADR-0538 glob-only members + ADR-0568 D2 "automation applies decisions, never
    /// invents them"), and inventing one would also risk sweeping unintended sibling dirs.
    WorkspaceMemberUncovered(String),
    /// A catalog render was requested with an empty `slo` or `plane` (never silently defaulted).
    MissingCatalogField(String),
    /// A catalog field (`slo`/`plane`) carried a value that is not a safe YAML scalar — a newline
    /// (`\n`/`\r`), a tab, a YAML-significant metacharacter, or leading/trailing whitespace. Such a
    /// value would forge YAML keys or change the scalar into a map/sequence, so the writer fails
    /// CLOSED. The message names the offending field.
    InvalidCatalogField(String),
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
            WriterError::MalformedGovernedPath(p) => {
                write!(
                    f,
                    "malformed governed path (newline, fence sequence, or surrounding whitespace): {p}"
                )
            }
            WriterError::WorkspaceManifest(m) => {
                write!(f, "root Cargo.toml workspace-manifest error: {m}")
            }
            WriterError::WorkspaceMemberUncovered(dir) => {
                write!(
                    f,
                    "crate dir is not covered by any [workspace].members glob (no glob may be \
                     synthesized — a covering glob is a human ADR-0538 decision): {dir}"
                )
            }
            WriterError::MissingCatalogField(field) => {
                write!(f, "catalog field is required and must not be empty: {field}")
            }
            WriterError::InvalidCatalogField(field) => {
                write!(
                    f,
                    "catalog field is not a safe YAML scalar (newline/metachar/whitespace): {field}"
                )
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

    /// True iff a line OPENS a fenced code block — its trimmed value starts with ```` ``` ````
    /// (with or without an info string like ` ```text `). Used by the single shared line scanner so
    /// the open-fence recognition cannot diverge between locating and extracting (the Defect-2 bug).
    fn is_open_fence(line: &str) -> bool {
        line.trim_start().starts_with(FENCE)
    }

    /// True iff a line CLOSES a fenced code block — its trimmed value is exactly ```` ``` ````
    /// (no info string is permitted on a closing fence, per CommonMark).
    fn is_close_fence(line: &str) -> bool {
        line.trim() == FENCE
    }

    /// True iff a line is any markdown heading (its first non-space char is `#`) — the boundary that
    /// ends the `## Governed surfaces` section.
    fn is_heading_line(line: &str) -> bool {
        line.trim_start().starts_with('#')
    }

    /// True iff a line is EXACTLY the canonical `## Governed surfaces` heading — a full-line match
    /// (Defect 5: a prefix like `## Governed surfaces (legacy)` must NOT match).
    fn is_governed_heading(line: &str) -> bool {
        line.trim_end() == HEADING
    }

    /// The repo-relative path of an ADR id's markdown file is resolved by the caller; the conventional
    /// shape is `docs/decisions/<id>-<slug>.md`. [`apply`] takes the full path so the writer stays
    /// repo-neutral about the slug.
    ///
    /// Compute the new ADR markdown content with `paths` upserted into the `## Governed surfaces`
    /// fenced block. PURE — no I/O. The block lists one VERBATIM tracked path per line, sorted +
    /// deduped (the canonical, byte-stable enumeration the producer's `resolve_justifications`
    /// tokenizes and credits).
    ///
    /// Placement (all confined to the heading's OWN section — i.e. up to the next markdown heading
    /// `#…` or EOF, so a foreign code block in a later section can never be hijacked, Defect 1):
    /// - heading present WITH a fenced block in its section → the existing in-block paths are merged
    ///   with `paths` and the block is rewritten IN PLACE (everything before/after, incl. later
    ///   sections, is preserved verbatim);
    /// - heading present WITHOUT a fence in its section → a fresh fenced block is INSERTED right
    ///   after the heading line (no second heading, no EOF append);
    /// - heading absent → `## Governed surfaces` + a fresh fenced block is appended at EOF, separated
    ///   from the body by exactly one blank line.
    ///
    /// Idempotent: re-running with paths already present yields byte-identical content.
    ///
    /// # Errors
    /// [`WriterError::BraceGlobInGovernedPath`] if any path carries brace-glob syntax (`{`/`}`);
    /// [`WriterError::MalformedGovernedPath`] if any path carries a newline, the fence sequence
    /// ```` ``` ````, or leading/trailing whitespace (defensive — the kernel already rejects these,
    /// but the writer fails CLOSED so every stored path is in a clean normal form where emit==parse).
    pub fn compute(current: &str, paths: &[String]) -> Result<String, WriterError> {
        for p in paths {
            if has_brace_glob(p) {
                return Err(WriterError::BraceGlobInGovernedPath(p.clone()));
            }
            // Fail-CLOSED: a newline would inject extra markdown lines, the fence sequence would
            // forge a fence, and surrounding whitespace is non-idempotent (emit verbatim, parse
            // trimmed). Reject so every stored path round-trips byte-for-byte.
            if p != p.trim() || p.contains('\n') || p.contains('\r') || p.contains(FENCE) {
                return Err(WriterError::MalformedGovernedPath(p.clone()));
            }
        }

        // The existing literal paths already in the block (if any), plus the new ones.
        let mut all: BTreeSet<String> = existing_block_paths(current).into_iter().collect();
        for p in paths {
            all.insert(p.clone());
        }
        let block_body: String = all.iter().map(|p| format!("{p}\n")).collect::<String>();
        // The fenced block body (opening fence is always bare — no info string — so re-emit is
        // canonical regardless of any info string the input used, Defect 2).
        let fenced = format!("{FENCE}\n{block_body}{FENCE}\n");

        match locate_section(current) {
            Some(section) => {
                let mut out = String::with_capacity(current.len() + block_body.len() + fenced.len());
                match section.fence {
                    Some((open_start, close_end)) => {
                        // Rewrite the existing fenced block in place; preserve everything outside it
                        // (incl. the heading line and any later sections) verbatim.
                        out.push_str(&current[..open_start]);
                        out.push_str(&fenced);
                        out.push_str(&current[close_end..]);
                    }
                    None => {
                        // Heading present but no fence in its section: insert a fresh fenced block
                        // immediately after the heading line, with a single blank-line separator.
                        let at = section.heading_line_end;
                        out.push_str(&current[..at]);
                        out.push('\n');
                        out.push_str(&fenced);
                        out.push_str(&current[at..]);
                    }
                }
                Ok(out)
            }
            None => {
                // Heading absent: append `## Governed surfaces` + a fresh block at EOF, separated
                // from the existing body by exactly one blank line.
                let new_block = format!("{HEADING}\n\n{fenced}");
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
    /// block exists in the heading's own section. Each non-empty trimmed line inside the fence is one
    /// path. Shares [`locate_section`] so fence recognition can never diverge from the locator.
    fn existing_block_paths(current: &str) -> Vec<String> {
        let Some(section) = locate_section(current) else {
            return Vec::new();
        };
        let Some((open_start, close_end)) = section.fence else {
            return Vec::new();
        };
        let block = &current[open_start..close_end];
        let mut paths = Vec::new();
        let mut seen_open = false;
        for line in block.lines() {
            if !seen_open {
                // The first line of the slice is the opening fence (possibly with an info string).
                seen_open = true;
                continue;
            }
            if is_close_fence(line) {
                break;
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.push(trimmed.to_owned());
            }
        }
        paths
    }

    /// The located `## Governed surfaces` section.
    struct Section {
        /// Byte offset just past the heading line's terminator (insertion point for a fresh fence).
        heading_line_end: usize,
        /// `(open_fence_start, close_fence_line_end)` byte offsets of the fenced block, when the
        /// section contains one. `open_fence_start` is the byte offset of the opening fence line;
        /// `close_fence_line_end` is just past the closing fence line's terminator (or EOF). `None`
        /// when the section has no fence.
        fence: Option<(usize, usize)>,
    }

    /// Locate the canonical `## Governed surfaces` section via a single line-based scan (Defect 5
    /// full-line heading match), confined to the heading's OWN section — the scan for an opening
    /// fence stops at the next markdown heading or EOF, so a foreign code block in a later section is
    /// never reached (Defect 1). Returns `None` when the canonical heading is absent.
    fn locate_section(current: &str) -> Option<Section> {
        let mut heading_line_end: Option<usize> = None;
        let mut fence: Option<(usize, usize)> = None;
        let mut open: Option<usize> = None;
        let mut offset = 0usize;

        for line in LinesWithEnds::new(current) {
            let line_start = offset;
            let line_end = offset + line.len();
            offset = line_end;
            let content = line.strip_suffix('\n').map_or(line, |l| {
                l.strip_suffix('\r').unwrap_or(l)
            });

            if heading_line_end.is_none() {
                if is_governed_heading(content) {
                    heading_line_end = Some(line_end);
                }
                continue;
            }

            // Inside the section. A new markdown heading ends the section.
            if open.is_none() {
                if is_heading_line(content) {
                    break; // section ended before any fence
                }
                if is_open_fence(content) {
                    open = Some(line_start);
                }
                continue;
            }

            // Inside the fenced block: look for the closing fence.
            if is_close_fence(content) {
                fence = Some((open.unwrap_or(line_start), line_end));
                break;
            }
        }

        heading_line_end.map(|heading_line_end| Section {
            heading_line_end,
            fence,
        })
    }

    /// Iterator over the lines of a string, each INCLUDING its `\n` terminator (the final line has
    /// no terminator iff the string does not end in `\n`). Unlike `str::lines`, this preserves byte
    /// lengths so the scanner's offsets reconstruct the input exactly.
    struct LinesWithEnds<'a> {
        rest: &'a str,
    }

    impl<'a> LinesWithEnds<'a> {
        fn new(s: &'a str) -> Self {
            LinesWithEnds { rest: s }
        }
    }

    impl<'a> Iterator for LinesWithEnds<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<&'a str> {
            if self.rest.is_empty() {
                return None;
            }
            match self.rest.find('\n') {
                Some(i) => {
                    let (line, rest) = self.rest.split_at(i + 1);
                    self.rest = rest;
                    Some(line)
                }
                None => {
                    let line = self.rest;
                    self.rest = "";
                    Some(line)
                }
            }
        }
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
    /// plus the human-decision `plane:` field, plus the `api_stability:` tier the
    /// `cloud-ci-lifecycle-status` api-stability-tier lane requires of every row). Both `plane` and
    /// `slo` are REQUIRED — an empty value is a [`WriterError::MissingCatalogField`] (never
    /// silently defaulted, per ADR-0548 D2). `api_stability` is not a parameter: see the inline
    /// note on why it is forced to `preview`.
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
        // Fail-CLOSED: each field is interpolated raw into the YAML, so a value carrying a newline
        // (forges a top-level key), a YAML-significant metacharacter (turns the scalar into a
        // map/sequence/anchor/etc.), or surrounding whitespace must be refused. Legit values are
        // simple identifiers (plane=`control`, slo=`ga-control-plane`), so this rejects nothing real.
        if !is_safe_yaml_scalar(plane) {
            return Err(WriterError::InvalidCatalogField("plane".to_owned()));
        }
        if !is_safe_yaml_scalar(slo) {
            return Err(WriterError::InvalidCatalogField("slo".to_owned()));
        }
        let capability = crate_leaf(crate_dir);
        // The minimal valid catalog record the gates consume: a top-level `slo:` scalar (the
        // slo-coverage / catalog-liveness contract) plus the human-supplied `plane:` and the
        // capability slug derived from the crate leaf. Rendered as canonical YAML (one
        // `key: value` per line, trailing newline) so re-rendering is byte-stable.
        //
        // `api_stability` is NOT decoration and NOT a human decision: the ci/facade/lifecycle-status
        // api-stability-tier lane is rooted on `registry/catalog/*.yaml` with `stage_field:
        // api_stability` and carries NO frozen violation row, so an absent (lane, kind) pair is
        // born-blocking and a row rendered without this key reds that required context as a
        // `stage_not_declared` unbaselined_violation the moment the next crate is registered.
        // The value is FORCED to `preview` rather than parameterised: it is the first tier of the
        // canonical [preview, stable, GA] ladder, and marketplace/facade/dev-cli/src/
        // governance_gates.rs validate_claim_ceiling_gate runs
        // FoundationClaimCeiling::preview_foundation().validate_catalog() over this exact
        // directory, which REJECTS any record declaring above Preview. A newly scaffolded crate
        // has no evidence for a higher tier, so any other default would be a claim the ceiling
        // gate rejects on sight. Promotion is a deliberate later edit to the row.
        Ok(format!(
            "capability: {capability}\nplane: {plane}\nslo: {slo}\napi_stability: preview\n"
        ))
    }

    /// YAML-significant metacharacters that, interpolated raw into a `key: value` line, would
    /// re-type the scalar (map/sequence/anchor/alias/tag/flow/quote/directive/comment) or otherwise
    /// break the byte-stable single-line render.
    const YAML_METACHARS: &[char] = &[
        ':', '{', '}', '[', ']', ',', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`', '#',
    ];

    /// True iff `value` is a safe single-line YAML scalar: no newline/carriage-return/tab, no
    /// YAML-significant metacharacter, and no leading/trailing whitespace. Fail-closed gate for the
    /// human-supplied `plane`/`slo` so an attacker can never forge keys or change the scalar type.
    fn is_safe_yaml_scalar(value: &str) -> bool {
        if value != value.trim() {
            return false;
        }
        !value
            .chars()
            .any(|c| c == '\n' || c == '\r' || c == '\t' || YAML_METACHARS.contains(&c))
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

// ───────────────────────────── 4. WorkspaceMemberGlobWriter ─────────────────────────────

/// Verify (never synthesize) that the root `[workspace].members` globs cover a crate dir.
pub mod workspace_member_glob {
    use super::{Path, WriterError, fs};
    use oya_workspace_members_kernel::{
        WorkspaceManifestEntries, member_entries_cover_dir, workspace_manifest_entries_from_str,
    };

    /// The repo-relative path of the root workspace manifest.
    pub const MANIFEST_PATH: &str = "Cargo.toml";

    /// Compute the new root `Cargo.toml` content for ensuring `dir` is covered by a
    /// `[workspace].members` glob. PURE — no I/O.
    ///
    /// Unlike the other three writers this is a coverage VERIFIER, not a mutator. The root members
    /// array is glob-only by ADR-0538 (the `cloud-ci-workspace-glob-coverage` gate makes a literal
    /// member a blocking violation) and each root's covering glob is a human ADR decision (ADR-0568
    /// D2 — automation applies decisions, never invents them). The [`Edit::WorkspaceMemberGlob`]
    /// edit carries only `dir` (no glob), so the only doctrine-clean outcomes are:
    /// - `dir` IS already covered by an existing members glob (and not removed by an `exclude`
    ///   subtree) → return `current` byte-unchanged (idempotent no-op; re-apply is byte-identical);
    /// - `dir` is NOT covered → [`WriterError::WorkspaceMemberUncovered`] (fail-closed; the writer
    ///   never widens the array, which could sweep unintended siblings and would forge a human
    ///   glob/ADR decision).
    ///
    /// Coverage is resolved purely through `oya-workspace-members-kernel`
    /// ([`member_entries_cover_dir`]) — the single source of `*`-per-component member-glob semantics
    /// and `exclude`-subtree rules — so this writer never re-derives glob matching (no drift). The
    /// filesystem `Cargo.toml`-presence check the kernel's full resolver applies is irrelevant here:
    /// the edit is only emitted for a crate dir that genuinely exists with a manifest.
    ///
    /// # Errors
    /// [`WriterError::WorkspaceManifest`] if the manifest lacks a parseable `[workspace]`
    /// members/exclude shape (fail-closed); [`WriterError::WorkspaceMemberUncovered`] if no glob
    /// covers `dir`.
    pub fn compute(current: &str, dir: &str) -> Result<String, WriterError> {
        let entries: WorkspaceManifestEntries = workspace_manifest_entries_from_str(current)
            .map_err(|e| WriterError::WorkspaceManifest(e.to_string()))?;
        let normalized = dir.trim_end_matches('/');
        if member_entries_cover_dir(&entries, normalized) {
            // Already covered: idempotent no-op (return the input verbatim — byte-identical
            // re-apply, no serialization round-trip, no comment loss).
            return Ok(current.to_owned());
        }
        Err(WriterError::WorkspaceMemberUncovered(normalized.to_owned()))
    }

    /// Apply the member-glob coverage check to the root `Cargo.toml` under `repo_root`. Reads the
    /// manifest, calls [`compute`], and writes ONLY if the bytes changed (which, for a coverage
    /// verifier, never happens — a covered dir yields byte-identical content and an uncovered dir
    /// fails closed). Returns `false` when `dir` is already covered (the no-op success); never
    /// returns `true` (this writer does not mutate the array) — see the module note on the
    /// verifier-vs-mutator asymmetry.
    ///
    /// # Errors
    /// [`WriterError::Io`] on a read failure, or any [`compute`] error (notably
    /// [`WriterError::WorkspaceMemberUncovered`]).
    pub fn apply(repo_root: &Path, dir: &str) -> Result<bool, WriterError> {
        let abs = repo_root.join(MANIFEST_PATH);
        let current = fs::read_to_string(&abs)
            .map_err(|e| WriterError::Io(format!("read {MANIFEST_PATH}: {e}")))?;
        let next = compute(&current, dir)?;
        if next == current {
            return Ok(false);
        }
        fs::write(&abs, &next)
            .map_err(|e| WriterError::Io(format!("write {MANIFEST_PATH}: {e}")))?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
