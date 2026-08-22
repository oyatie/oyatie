//! Move-aware path resolution — git + manifest ADAPTERS (ci/adapters).
//!
//! Implements the [`ci_path_resolver_ports`] contract:
//!  * [`ManifestBijection`] — loads the de-committed (materialized-on-demand) move-manifest
//!    fail-closed; identity (empty) on a foreign schema, a malformed row, or a NON-INJECTIVE side
//!    (MUST-PASS #3: both old AND new sides must be injective, else `new_to_old`/`old_to_new` are
//!    not well-defined functions). ABSENCE is a hard `Err` (ADR-0614 materialization precondition).
//!  * [`GitCliFrozenRefSource`] — the live `git merge-base` / `git show` seam (fail-closed).
//!  * [`ManifestPathResolver`] — the presence-verified, straddle-safe, fail-closed resolution.
//!
//! WHY THE MERGE-BASE SPLIT CANNOT LAUNDER DEBT (corrected rationale — MUST-PASS #6). The defense
//! is NOT "registry-drift gate-ordering". It is three concrete, independent facts:
//!   1. IMMUTABLE MERGE-BASE HISTORY — the frozen reference is read from `git merge-base
//!      <bootstrap> HEAD`, a point in committed history an attacker cannot rewrite in a PR.
//!   2. TRUSTED COMPILED SEED — the candidate/current name is a compiled-in const
//!      ([`canonical_current`]), never read from a candidate-tree data file, so there is no
//!      candidate-repointable anchor to aim the merge-base read through (MUST-PASS #1).
//!   3. MANIFEST REGENERATED-FROM-PLAN-BEFORE-READ — the new<->old bridge comes from the
//!      de-committed move-manifest (ADR-0614), which the materialize step regenerates from the
//!      committed move-plan and whose regenerate-twice determinism (registry-drift) binds it to
//!      the codemod's deterministic output; a hand-forged pair is RED before the firewall consumes
//!      the snapshot.
//!
//! On top of those, the resolver is FAIL-CLOSED: a manifest-declared name absent from BOTH sides
//! of history, or ambiguously present on both, is a HARD ERROR — never a fallback to an empty,
//! candidate, or bootstrap reference (that empty-reference fallback is the exact laundering vector
//! the naive NEW-const flip reopens).
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use ci_path_resolver_ports::{
    FrozenRefSource, MergeBaseName, MoveBijection, PathId, PathResolver, canonical_current,
};
use serde_json::Value;

/// The move-manifest schema id (must match the codemod's `REORG_MOVE_MANIFEST_SCHEMA`).
pub const MOVE_MANIFEST_SCHEMA: &str = "ci/reorg-move-manifest/v1";

/// The de-committed move-manifest's canonical repo-relative materialization path (a root-stable
/// anchor — a well-known config location, injected once, NOT a movable gate self-location). The
/// face is not-tracked-in-git (ADR-0614); CI materializes it before any relabel-read leg.
pub const MOVE_MANIFEST_PATH: &str = "specs/reorg/move-manifest.generated.json";

/// Logical resource prefix used by committed Cargo config for binaries that Cargo builds into
/// the active target/profile directory. The committed value carries no checkout or target path;
/// [`resolve_cargo_test_binary`] derives that runtime detail from the calling executable.
pub const CARGO_TEST_BINARY_PREFIX: &str = "cargo-test-binary:";

/// Resolve an externally supplied absolute/Buck-relative binary binding or a Cargo logical test
/// resource. Cargo tests live under `<profile>/deps`, while directly launched Cargo binaries live
/// in `<profile>`; deriving from `current_exe` covers either shape, custom `CARGO_TARGET_DIR`,
/// target triples, and non-default profiles without committed machine-specific paths.
pub fn resolve_cargo_test_binary(repo_root: &Path, value: &OsStr) -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("resolve current Cargo executable: {error}"))?;
    resolve_cargo_test_binary_from_executable(repo_root, value, &executable)
}

/// Deterministic/testable form of [`resolve_cargo_test_binary`] with the process executable
/// supplied explicitly.
pub fn resolve_cargo_test_binary_from_executable(
    repo_root: &Path,
    value: &OsStr,
    executable: &Path,
) -> Result<PathBuf, String> {
    let declared = PathBuf::from(value);
    if declared.is_absolute() {
        return Ok(declared);
    }

    let Some(value) = value.to_str() else {
        return Ok(repo_root.join(declared));
    };
    let Some(name) = value.strip_prefix(CARGO_TEST_BINARY_PREFIX) else {
        return Ok(repo_root.join(declared));
    };
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err(format!("invalid Cargo test binary resource name {name:?}"));
    }

    let executable_dir = executable
        .parent()
        .ok_or_else(|| format!("executable {} has no parent", executable.display()))?;
    let profile_dir = if executable_dir.file_name() == Some(OsStr::new("deps")) {
        executable_dir.parent().ok_or_else(|| {
            format!(
                "Cargo deps directory {} has no profile parent",
                executable_dir.display()
            )
        })?
    } else {
        executable_dir
    };
    Ok(profile_dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX)))
}

// ---------------------------------------------------------------------------
// ManifestBijection
// ---------------------------------------------------------------------------

/// The parsed de-committed move-manifest. FAIL-CLOSED: a foreign schema, any malformed row, or
/// a non-injective side in any pair list yields the EMPTY manifest (identity — no pending move),
/// so consumers never partially trust an ambiguous move map. File ABSENCE is a separate hard `Err`
/// (materialization precondition), not empty/identity.
#[derive(Debug, Clone, Default)]
pub struct MoveManifest {
    file_pairs: Vec<(String, String)>,
    crate_dir_pairs: Vec<(String, String)>,
    crate_ident_pairs: Vec<(String, String)>,
}

impl MoveManifest {
    /// The empty (identity) bijection.
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.file_pairs.is_empty()
            && self.crate_dir_pairs.is_empty()
            && self.crate_ident_pairs.is_empty()
    }

    pub fn file_pairs(&self) -> &[(String, String)] {
        &self.file_pairs
    }

    pub fn crate_dir_pairs(&self) -> &[(String, String)] {
        &self.crate_dir_pairs
    }

    pub fn crate_ident_pairs(&self) -> &[(String, String)] {
        &self.crate_ident_pairs
    }

    /// Parse a manifest `Value` fail-closed. MUST-PASS #3: BOTH sides injective — a duplicated old
    /// OR new side in any pair list collapses to the empty manifest (an ambiguous map is never
    /// partially trusted).
    pub fn from_manifest_value(value: &Value) -> Self {
        if value.get("schema").and_then(Value::as_str) != Some(MOVE_MANIFEST_SCHEMA) {
            return Self::empty();
        }
        let Some(file_pairs) = parse_path_pairs(value, "files") else {
            return Self::empty();
        };
        let Some(crate_dir_pairs) = parse_path_pairs(value, "crate_dirs") else {
            return Self::empty();
        };
        let Some(crate_ident_pairs) = parse_ident_pairs(value, "crate_idents") else {
            return Self::empty();
        };
        Self {
            file_pairs,
            crate_dir_pairs,
            crate_ident_pairs,
        }
    }

    /// Load + parse the move-manifest from the candidate tree. FAIL-CLOSED on ABSENT (ADR-0614):
    /// a missing/unreadable file is a HARD `Err` — a materialization precondition failure, not a
    /// no-move state. A PRESENT-but-unparseable/foreign/malformed body stays `Ok(empty)` (identity):
    /// the DELIBERATE anti-laundering leniency — a candidate-supplied forged manifest is never
    /// trusted, it collapses to identity (the registry-drift regenerate-twice byte-binding is the
    /// forgery trust root). So the split is purely ABSENT-file (`Err`) vs PRESENT-file (parse
    /// leniently, `Ok`).
    ///
    /// PRECONDITION (ADR-0614, amends ADR-0563): move-manifest is now DE-COMMITTED — not tracked in
    /// git (`materialization_mode: not-tracked-in-git`). cloud-ci materializes it on demand as STEP 1
    /// of `//ci/facade/generated-artifact-freshness:cloud-ci-materialize-generated-faces-bin`
    /// (`materialize_move_manifest`) BEFORE any relabel-read leg, so the manifest is present on disk
    /// when this loads it. Post-de-commit an ABSENT file means the materializer did not run — a
    /// pipeline precondition failure that must block loudly (a false-RED that never merges bad),
    /// NOT silently degrade to an identity relabel. Every scm-facts emitter invocation runs via the
    /// materializer (audited across `.github/workflows/presubmit.yml`), so this `Err` arm never
    /// triggers in practice — it converts a latent silent hazard into a loud precondition assertion.
    pub fn load(repo_root: &Path, manifest_rel_path: &str) -> Result<Self, String> {
        let path = repo_root.join(manifest_rel_path);
        let bytes = std::fs::read(&path).map_err(|e| {
            format!(
                "move-manifest absent/unreadable at {}: {e} — the materializer \
                 (materialize_move_manifest, step 1 of the generated-faces materializer) must run \
                 first (ADR-0614); refusing to silently relabel to identity",
                path.display()
            )
        })?;
        // PRESENT but unparseable => Ok(empty) identity, UNCHANGED (anti-laundering): a forged/foreign
        // body is never trusted; it collapses to identity, never a hard error (that would let a forged
        // manifest hard-fail CI on a different path than the registry-drift byte-binding catches it).
        let Ok(text) = std::str::from_utf8(&bytes) else {
            return Ok(Self::empty());
        };
        Ok(match serde_json::from_str::<Value>(text) {
            Ok(value) => Self::from_manifest_value(&value),
            Err(_) => Self::empty(),
        })
    }
}

fn parse_path_pairs(value: &Value, field: &str) -> Option<Vec<(String, String)>> {
    parse_pairs(value, field, "old_path", "new_path")
}

fn parse_ident_pairs(value: &Value, field: &str) -> Option<Vec<(String, String)>> {
    parse_pairs(value, field, "old", "new")
}

fn parse_pairs(
    value: &Value,
    field: &str,
    old_key: &str,
    new_key: &str,
) -> Option<Vec<(String, String)>> {
    let rows = value.get(field).and_then(Value::as_array)?;
    let mut pairs: Vec<(String, String)> = Vec::new();
    let mut old_to_new: BTreeMap<String, String> = BTreeMap::new();
    let mut new_to_old: BTreeMap<String, String> = BTreeMap::new();
    for row in rows {
        match (
            row.get(old_key).and_then(Value::as_str),
            row.get(new_key).and_then(Value::as_str),
        ) {
            (Some(old), Some(new)) if !old.is_empty() && !new.is_empty() => {
                // A malformed/duplicate row poisons the WHOLE manifest fail-closed (identity)
                // rather than silently trusting a partial, ambiguous map.
                if old_to_new.insert(old.to_owned(), new.to_owned()).is_some() {
                    return None;
                }
                if new_to_old.insert(new.to_owned(), old.to_owned()).is_some() {
                    return None;
                }
                pairs.push((old.to_owned(), new.to_owned()));
            }
            _ => return None,
        }
    }
    Some(pairs)
}

/// The old<->new bijection from the de-committed move-manifest `files[]`.
#[derive(Debug, Clone, Default)]
pub struct ManifestBijection {
    manifest: MoveManifest,
    old_to_new: BTreeMap<String, String>,
    new_to_old: BTreeMap<String, String>,
}

impl ManifestBijection {
    /// The empty (identity) bijection.
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.old_to_new.is_empty()
    }

    pub fn manifest(&self) -> &MoveManifest {
        &self.manifest
    }

    /// Parse a manifest `Value` fail-closed through the shared [`MoveManifest`] parser.
    pub fn from_manifest_value(value: &Value) -> Self {
        Self::from_manifest(MoveManifest::from_manifest_value(value))
    }

    pub fn from_manifest(manifest: MoveManifest) -> Self {
        let old_to_new = manifest
            .file_pairs()
            .iter()
            .map(|(old, new)| (old.clone(), new.clone()))
            .collect();
        let new_to_old = manifest
            .file_pairs()
            .iter()
            .map(|(old, new)| (new.clone(), old.clone()))
            .collect();
        Self {
            manifest,
            old_to_new,
            new_to_old,
        }
    }

    /// Load + parse the manifest from the candidate tree. FAIL-CLOSED on ABSENT (ADR-0614): a
    /// missing/unreadable file is a HARD `Err` (materialization precondition), while a
    /// PRESENT-but-unparseable body stays `Ok(empty)` — the anti-laundering leniency. See
    /// [`MoveManifest::load`] for the full four-way semantics.
    pub fn load(repo_root: &Path, manifest_rel_path: &str) -> Result<Self, String> {
        Ok(Self::from_manifest(MoveManifest::load(
            repo_root,
            manifest_rel_path,
        )?))
    }
}

impl MoveBijection for ManifestBijection {
    fn new_to_old(&self, new_path: &str) -> Option<&str> {
        self.new_to_old.get(new_path).map(String::as_str)
    }
    fn old_to_new(&self, old_path: &str) -> Option<&str> {
        self.old_to_new.get(old_path).map(String::as_str)
    }
}

// ---------------------------------------------------------------------------
// GitCliFrozenRefSource
// ---------------------------------------------------------------------------

/// The live git merge-base seam (ADR-0515 D3 single sanctioned git boundary). Fail-closed: an
/// unresolvable ref/merge-base is a HARD error (never a silent PR-controlled fallback).
pub struct GitCliFrozenRefSource<'a> {
    repo_root: &'a Path,
}

impl<'a> GitCliFrozenRefSource<'a> {
    pub fn new(repo_root: &'a Path) -> Self {
        Self { repo_root }
    }
}

impl FrozenRefSource for GitCliFrozenRefSource<'_> {
    fn merge_base(&self, base_ref: &str) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(self.repo_root)
            .args(["merge-base", base_ref, "HEAD"])
            .output()
            .map_err(|e| format!("merge-base: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "git merge-base {base_ref} HEAD failed (exit {:?}): {} — the frozen ratchet \
                 reference REQUIRES the base ref; fetch it or repoint ratchet-policy.json base_ref",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if sha.len() < 40 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("git merge-base produced a non-revision: {sha:?}"));
        }
        Ok(sha)
    }

    fn show_file(&self, revision: &str, path: &str) -> Result<Option<String>, String> {
        let spec = format!("{revision}:{path}");
        let exists = Command::new("git")
            .arg("-C")
            .arg(self.repo_root)
            .args(["cat-file", "-e", &spec])
            .output()
            .map_err(|e| format!("cat-file: {e}"))?;
        if !exists.status.success() {
            return Ok(None);
        }
        let output = Command::new("git")
            .arg("-C")
            .arg(self.repo_root)
            .args(["show", &spec])
            .output()
            .map_err(|e| format!("show: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "git show {spec} failed (exit {:?}): {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        String::from_utf8(output.stdout)
            .map(Some)
            .map_err(|e| format!("show {spec}: {e}"))
    }
}

// ---------------------------------------------------------------------------
// ManifestPathResolver
// ---------------------------------------------------------------------------

/// The move-aware resolver: compiled current-canonical seed on the candidate side, manifest +
/// immutable-history driven on the merge-base side.
pub struct ManifestPathResolver {
    bijection: ManifestBijection,
}

impl ManifestPathResolver {
    pub fn new(bijection: ManifestBijection) -> Self {
        Self { bijection }
    }

    /// Load the resolver from the candidate tree. FAIL-CLOSED on ABSENT (ADR-0614): the
    /// move-manifest is DE-COMMITTED and materialized before this reads it, so a missing file is a
    /// HARD `Err` (the materializer did not run) rather than a silent identity relabel — see
    /// [`MoveManifest::load`] for the materialize-first precondition and the four-way semantics.
    pub fn load(repo_root: &Path) -> Result<Self, String> {
        Ok(Self::new(ManifestBijection::load(
            repo_root,
            MOVE_MANIFEST_PATH,
        )?))
    }

    /// The identity resolver (empty bijection). For callers that use the resolver ONLY for
    /// `candidate()` current-canonical path lookup (identity-safe) and legitimately run WITHOUT a
    /// materialized move-manifest — e.g. the freshness gate's non-`--merge-base-baseline` scm-facts
    /// regen. The move-aware RELABEL path fail-closes on absence separately (see [`MoveManifest::load`]).
    pub fn empty() -> Self {
        Self::new(ManifestBijection::empty())
    }
}

impl PathResolver for ManifestPathResolver {
    fn candidate(&self, id: PathId) -> String {
        // The current canonical (post-move NEW) location. `old_to_new` handles the defensive case
        // where the compiled seed is still a pre-move OLD name during the landing PR; once the seed
        // is rebased NEW it is not an old-key, so this is the identity (seed) — the stable path.
        let seed = canonical_current(id);
        if matches!(id, PathId::RatchetPolicy) {
            return seed.to_owned();
        }
        self.bijection
            .old_to_new(seed)
            .map(str::to_owned)
            .unwrap_or_else(|| seed.to_owned())
    }

    fn at_merge_base(
        &self,
        id: PathId,
        merge_base: &str,
        src: &dyn FrozenRefSource,
    ) -> Result<MergeBaseName, String> {
        if matches!(id, PathId::RatchetPolicy) {
            let seed = canonical_current(id);
            if let Some(mapped) = self.bijection.old_to_new(seed)
                && mapped != seed
            {
                return Err(format!(
                    "move-manifest treats RatchetPolicy canonical current seed {seed:?} as \
                     an OLD key and repoints it to {mapped:?}; manifest new-seed must equal \
                     canonical_current, fail-closed"
                ));
            }
        }
        let candidate = self.candidate(id);
        match self.bijection.new_to_old(&candidate) {
            // A pending move declares a pre-move OLD name for this path.
            Some(old) => {
                let old = old.to_owned();
                let old_present = src.show_file(merge_base, &old)?.is_some();
                let new_present = src.show_file(merge_base, &candidate)?.is_some();
                match (old_present, new_present) {
                    // Ambiguous: both names present at the merge-base (double-write / stem
                    // collision). HARD ERROR (MUST-PASS #2 / spec §1.2 step 5) — never guess.
                    (true, true) => Err(format!(
                        "move-manifest declares {old:?} -> {candidate:?} but BOTH are present at \
                         merge-base {merge_base} — ambiguous frozen reference, fail-closed"
                    )),
                    // Genuine pending move (the move PR): the file is under its OLD name at the
                    // pre-move merge-base. Read the frozen reference there.
                    (true, false) => Ok(MergeBaseName::Present(old)),
                    // The move already landed in merge-base history (STRADDLE — MUST-PASS #5: a
                    // rebase PAST the move commit puts NEW, not OLD, at the merge-base). Read the
                    // NEW name — PRESENCE-VERIFIED in immutable history, identical content — so a
                    // merged move cannot hard-error every subsequent PR. NOT an empty fallback.
                    (false, true) => Ok(MergeBaseName::Present(candidate)),
                    // The manifest declares a pre-move name that is absent from BOTH sides of
                    // history: the manifest lies about the pre-move state. HARD ERROR (MUST-PASS
                    // #2) — NEVER fall back to bootstrap/candidate/empty (that reopens laundering).
                    (false, false) => Err(format!(
                        "move-manifest declares {old:?} -> {candidate:?} but NEITHER is present at \
                         merge-base {merge_base} — refusing an empty/forged frozen reference, \
                         fail-closed"
                    )),
                }
            }
            // No pending move for this path: read it at its current name.
            None => match src.show_file(merge_base, &candidate)? {
                Some(_) => Ok(MergeBaseName::Present(candidate)),
                // Genuinely absent and NOT a declared move => genuine repo bootstrap (the PR that
                // introduces the file). Feeds the existing `missing_at_merge_base` path unchanged.
                None => Ok(MergeBaseName::Absent),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake merge-base tree: the set of paths that "exist" at the merge-base, each with content.
    struct FakeSource {
        present: BTreeMap<String, String>,
    }
    impl FakeSource {
        fn with(paths: &[(&str, &str)]) -> Self {
            Self {
                present: paths
                    .iter()
                    .map(|(p, c)| ((*p).to_owned(), (*c).to_owned()))
                    .collect(),
            }
        }
    }
    impl FrozenRefSource for FakeSource {
        fn merge_base(&self, _base_ref: &str) -> Result<String, String> {
            Ok("0".repeat(40))
        }
        fn show_file(&self, _revision: &str, path: &str) -> Result<Option<String>, String> {
            Ok(self.present.get(path).cloned())
        }
    }

    const OLD: &str = "cloud/cloud-ci/gates/cloud-ci-firewall-app/ratchet-policy.json";
    const NEW: &str = "ci/facade/baseline-ratchet/ratchet-policy.json";

    fn manifest_with(pairs: &[(&str, &str)]) -> Value {
        serde_json::json!({
            "schema": MOVE_MANIFEST_SCHEMA,
            "files": pairs.iter().map(|(o, n)| serde_json::json!({"old_path": o, "new_path": n})).collect::<Vec<_>>(),
            "crate_dirs": [],
            "crate_idents": [],
        })
    }

    fn resolver_with(pairs: &[(&str, &str)]) -> ManifestPathResolver {
        ManifestPathResolver::new(ManifestBijection::from_manifest_value(&manifest_with(
            pairs,
        )))
    }

    /// MUST-PASS #1: `candidate` is the compiled seed — it consults NO candidate data file, and is
    /// the NEW path. (The seed itself is compiled; there is no input through which a candidate tree
    /// could repoint it.)
    #[test]
    fn candidate_is_the_compiled_new_seed() {
        let r = resolver_with(&[(OLD, NEW)]);
        assert_eq!(r.candidate(PathId::RatchetPolicy), NEW);
        // Even with an EMPTY manifest the candidate is the compiled NEW seed (never OLD).
        let r0 = resolver_with(&[]);
        assert_eq!(r0.candidate(PathId::RatchetPolicy), NEW);
    }

    /// Genuine move PR: OLD present at the pre-move merge-base, NEW absent => read OLD.
    #[test]
    fn move_pr_reads_old_at_pre_move_merge_base() {
        let r = resolver_with(&[(OLD, NEW)]);
        let src = FakeSource::with(&[(OLD, "policy")]);
        assert_eq!(
            r.at_merge_base(PathId::RatchetPolicy, "mb", &src).unwrap(),
            MergeBaseName::Present(OLD.to_owned())
        );
    }

    /// MUST-PASS #5 (straddle): move already in merge-base history — OLD absent, NEW present =>
    /// read NEW (present-verified), NOT a hard error, NOT an empty reference.
    #[test]
    fn straddle_reads_new_when_move_already_in_history() {
        let r = resolver_with(&[(OLD, NEW)]);
        let src = FakeSource::with(&[(NEW, "policy")]);
        assert_eq!(
            r.at_merge_base(PathId::RatchetPolicy, "mb", &src).unwrap(),
            MergeBaseName::Present(NEW.to_owned())
        );
    }

    /// MUST-PASS #2 (both absent): manifest declares OLD->NEW but NEITHER exists at the merge-base
    /// => HARD ERROR (never empty/bootstrap/candidate fallback — the laundering vector).
    #[test]
    fn both_absent_is_hard_error_not_empty() {
        let r = resolver_with(&[(OLD, NEW)]);
        let src = FakeSource::with(&[]); // neither present
        let err = r
            .at_merge_base(PathId::RatchetPolicy, "mb", &src)
            .unwrap_err();
        assert!(err.contains("NEITHER"), "unexpected error: {err}");
    }

    /// MUST-PASS #2 (both present): ambiguous double-write at the merge-base => HARD ERROR.
    #[test]
    fn both_present_is_hard_error_ambiguous() {
        let r = resolver_with(&[(OLD, NEW)]);
        let src = FakeSource::with(&[(OLD, "a"), (NEW, "b")]);
        let err = r
            .at_merge_base(PathId::RatchetPolicy, "mb", &src)
            .unwrap_err();
        assert!(err.contains("BOTH"), "unexpected error: {err}");
    }

    /// No pending move + file absent at merge-base => Absent (genuine bootstrap), NOT an error:
    /// preserves the existing `missing_at_merge_base` path (MUST-PASS #4 support).
    #[test]
    fn no_move_and_absent_is_bootstrap_absent() {
        let r = resolver_with(&[]); // empty manifest
        let src = FakeSource::with(&[]);
        assert_eq!(
            r.at_merge_base(PathId::RatchetPolicy, "mb", &src).unwrap(),
            MergeBaseName::Absent
        );
    }

    /// No pending move + file present at merge-base under its current name => Present(NEW).
    #[test]
    fn no_move_and_present_reads_current_name() {
        let r = resolver_with(&[]);
        let src = FakeSource::with(&[(NEW, "policy")]);
        assert_eq!(
            r.at_merge_base(PathId::RatchetPolicy, "mb", &src).unwrap(),
            MergeBaseName::Present(NEW.to_owned())
        );
    }

    /// MUST-PASS #3 (new-side injectivity): a manifest whose NEW side is non-injective (two OLDs
    /// map to the same NEW) is rejected to the EMPTY bijection — `new_to_old` would otherwise be
    /// ill-defined. (The stock parser only checked old-side uniqueness.)
    #[test]
    fn new_side_non_injective_manifest_collapses_to_empty() {
        let b = ManifestBijection::from_manifest_value(&manifest_with(&[
            ("old/a", "new/shared"),
            ("old/b", "new/shared"),
        ]));
        assert!(
            b.is_empty(),
            "non-injective new side must fail closed to empty"
        );
        assert_eq!(b.new_to_old("new/shared"), None);
    }

    /// Old-side injectivity is also enforced (duplicate old_path => empty).
    #[test]
    fn old_side_non_injective_manifest_collapses_to_empty() {
        let b = ManifestBijection::from_manifest_value(&manifest_with(&[
            ("old/a", "new/x"),
            ("old/a", "new/y"),
        ]));
        assert!(
            b.is_empty(),
            "non-injective old side must fail closed to empty"
        );
    }

    /// A foreign schema fails closed to empty (identity).
    #[test]
    fn foreign_schema_is_empty() {
        let v = serde_json::json!({"schema": "not-ours", "files": [{"old_path": OLD, "new_path": NEW}]});
        assert!(ManifestBijection::from_manifest_value(&v).is_empty());
    }

    /// A malformed row poisons the whole manifest fail-closed.
    #[test]
    fn malformed_row_is_empty() {
        let v = serde_json::json!({"schema": MOVE_MANIFEST_SCHEMA, "files": [{"old_path": OLD}], "crate_dirs": [], "crate_idents": []});
        assert!(ManifestBijection::from_manifest_value(&v).is_empty());
    }

    /// The shared parser covers the emitter's relabel-only pair lists too: duplicate NEW crate
    /// dirs/idents are ambiguous and collapse the whole manifest to identity.
    #[test]
    fn relabel_pair_lists_are_both_side_injective() {
        let duplicate_new_dir = serde_json::json!({
            "schema": MOVE_MANIFEST_SCHEMA,
            "files": [],
            "crate_dirs": [
                {"old_path": "old/a", "new_path": "new/shared"},
                {"old_path": "old/b", "new_path": "new/shared"}
            ],
            "crate_idents": [],
        });
        assert!(
            MoveManifest::from_manifest_value(&duplicate_new_dir).is_empty(),
            "duplicate new crate_dirs must fail closed"
        );

        let duplicate_new_ident = serde_json::json!({
            "schema": MOVE_MANIFEST_SCHEMA,
            "files": [],
            "crate_dirs": [],
            "crate_idents": [
                {"old": "old-a", "new": "new-shared"},
                {"old": "old-b", "new": "new-shared"}
            ],
        });
        assert!(
            MoveManifest::from_manifest_value(&duplicate_new_ident).is_empty(),
            "duplicate new crate_idents must fail closed"
        );
    }

    /// Sanity: the built bijection is a real inverse pair on a well-formed manifest.
    #[test]
    fn well_formed_bijection_inverts() {
        let b = ManifestBijection::from_manifest_value(&manifest_with(&[(OLD, NEW)]));
        assert_eq!(b.old_to_new(OLD), Some(NEW));
        assert_eq!(b.new_to_old(NEW), Some(OLD));
    }

    /// AC4 anti-forgery: a manifest must not treat the compiled current RatchetPolicy seed as an
    /// OLD key and repoint it to an attacker-chosen NEW key.
    #[test]
    fn ratchet_policy_rejects_seed_as_old_key_repoint() {
        let forged_new = "ci/facade/baseline-ratchet/forged-ratchet-policy.json";
        let r = resolver_with(&[(NEW, forged_new)]);
        let src = FakeSource::with(&[(NEW, "policy")]);

        assert_eq!(r.candidate(PathId::RatchetPolicy), NEW);

        let err = r
            .at_merge_base(PathId::RatchetPolicy, "mb", &src)
            .unwrap_err();

        assert!(
            err.contains("canonical current seed"),
            "unexpected error: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // MoveManifest::load — ABSENT-vs-PRESENT four-way semantics (ADR-0614 fail-closed hardening).
    // ABSENT => Err (materialization precondition); PRESENT parses leniently => Ok (empty/bijection/
    // identity). The anti-laundering property (present-but-forged => Ok identity, NEVER Err) is
    // asserted explicitly so it can never silently regress into a hard-fail.
    // -----------------------------------------------------------------------

    use std::sync::atomic::{AtomicU64, Ordering};

    /// A unique, writable repo-root under the temp dir — no external test dependency (the crate
    /// carries only serde_json + the ports crate).
    fn fresh_repo_root() -> std::path::PathBuf {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "path-resolver-load-{}-{nanos}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_manifest_body(root: &std::path::Path, body: &str) {
        let path = root.join(MOVE_MANIFEST_PATH);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
    }

    /// ABSENT (file does not exist) => HARD `Err`: the materializer did not run — a pipeline
    /// precondition failure, NOT a silent identity relabel. The whole-resolver load fails the same.
    #[test]
    fn load_absent_manifest_is_hard_error() {
        let root = fresh_repo_root(); // manifest deliberately NOT written
        let err = MoveManifest::load(&root, MOVE_MANIFEST_PATH).unwrap_err();
        assert!(
            err.contains("materialize_move_manifest"),
            "absent must name the materializer precondition: {err}"
        );
        assert!(
            ManifestPathResolver::load(&root).is_err(),
            "the whole-resolver load must fail closed on an absent manifest"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// PRESENT but empty (schema + empty pair lists) => `Ok(identity)`: a legitimate no-move state.
    #[test]
    fn load_present_empty_manifest_is_ok_identity() {
        let root = fresh_repo_root();
        write_manifest_body(&root, &manifest_with(&[]).to_string());
        let m = MoveManifest::load(&root, MOVE_MANIFEST_PATH).expect("present-empty must be Ok");
        assert!(m.is_empty(), "no-move manifest must load as identity");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// PRESENT with a move => `Ok(non-empty bijection)`: the manifest is trusted (its bytes are
    /// bound to the codemod's deterministic output by the upstream registry-drift/freshness gate).
    #[test]
    fn load_present_with_move_is_ok_non_empty() {
        let root = fresh_repo_root();
        write_manifest_body(&root, &manifest_with(&[(OLD, NEW)]).to_string());
        let b =
            ManifestBijection::load(&root, MOVE_MANIFEST_PATH).expect("present-move must be Ok");
        assert!(!b.is_empty());
        assert_eq!(b.old_to_new(OLD), Some(NEW));
        assert_eq!(b.new_to_old(NEW), Some(OLD));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// ANTI-LAUNDERING (must NOT regress): a PRESENT-but-forged body — a foreign schema OR
    /// unparseable JSON — stays `Ok(identity)`, NEVER an `Err`. A candidate-supplied forged manifest
    /// is never trusted; it collapses to identity rather than hard-failing CI on a path other than
    /// the registry-drift byte-binding that actually catches the forgery.
    #[test]
    fn load_present_but_forged_is_ok_identity_not_error() {
        let root = fresh_repo_root();

        // (a) foreign schema — parses as JSON, wrong schema id.
        write_manifest_body(
            &root,
            r#"{"schema":"attacker/forged/v1","files":[{"old_path":"a","new_path":"b"}]}"#,
        );
        let foreign = MoveManifest::load(&root, MOVE_MANIFEST_PATH)
            .expect("foreign schema must stay Ok (anti-laundering), never Err");
        assert!(
            foreign.is_empty(),
            "forged foreign schema must collapse to identity"
        );

        // (b) unparseable body — present file, not valid JSON.
        write_manifest_body(&root, "{ this is not json");
        let garbage = MoveManifest::load(&root, MOVE_MANIFEST_PATH)
            .expect("unparseable body must stay Ok (anti-laundering), never Err");
        assert!(
            garbage.is_empty(),
            "unparseable manifest must collapse to identity"
        );

        // (c) non-UTF-8 body — present file, not decodable JSON text.
        let path = root.join(MOVE_MANIFEST_PATH);
        std::fs::write(&path, [0xff, 0xfe, b'{']).unwrap();
        let non_utf8 = MoveManifest::load(&root, MOVE_MANIFEST_PATH)
            .expect("non-UTF-8 present body must stay Ok (anti-laundering), never Err");
        assert!(
            non_utf8.is_empty(),
            "non-UTF-8 manifest must collapse to identity"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn cargo_test_binary_binding_derives_custom_target_profile_at_runtime() {
        let executable =
            Path::new("/fresh/custom-target/aarch64-unknown-linux-gnu/debug/deps/gate-abc");
        let expected = Path::new("/fresh/custom-target/aarch64-unknown-linux-gnu/debug")
            .join(format!("producer{}", std::env::consts::EXE_SUFFIX));
        assert_eq!(
            resolve_cargo_test_binary_from_executable(
                Path::new("/different/checkout"),
                OsStr::new("cargo-test-binary:producer"),
                executable,
            ),
            Ok(expected)
        );
    }

    #[test]
    fn cargo_test_binary_binding_preserves_external_and_rejects_traversal() {
        let executable = Path::new("/custom-target/debug/deps/gate-abc");
        assert_eq!(
            resolve_cargo_test_binary_from_executable(
                Path::new("/repo"),
                OsStr::new("buck-out/producer"),
                executable,
            ),
            Ok(PathBuf::from("/repo/buck-out/producer"))
        );
        assert_eq!(
            resolve_cargo_test_binary_from_executable(
                Path::new("/repo"),
                OsStr::new("/declared/producer"),
                executable,
            ),
            Ok(PathBuf::from("/declared/producer"))
        );
        for invalid in [
            "cargo-test-binary:",
            "cargo-test-binary:../producer",
            "cargo-test-binary:dir/producer",
            "cargo-test-binary:dir\\producer",
        ] {
            assert!(
                resolve_cargo_test_binary_from_executable(
                    Path::new("/repo"),
                    OsStr::new(invalid),
                    executable,
                )
                .is_err(),
                "binding {invalid:?} must fail closed"
            );
        }
    }
}
