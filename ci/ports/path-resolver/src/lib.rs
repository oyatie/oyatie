//! Move-aware path resolution — PURE contract (ci/ports, no I/O).
//!
//! The self-referential CI gates hardcode their own file self-locations (e.g. the firewall's
//! `ratchet-policy.json`, the scm-facts faces). A capability-first strangler move (ADR-0562)
//! relocates those files, which breaks the gate two opposite ways depending on WHICH tree the
//! read is against (OYA-CI keystone unblock):
//!
//!  * a CANDIDATE (working-tree) read/write needs the file's CURRENT location (post-move = NEW);
//!  * the ONE merge-base read (the firewall's frozen reference, ADR-0551/FRIC-1781280000) needs
//!    the name the file bore AT `git merge-base <bootstrap> HEAD` — which is the pre-move (OLD)
//!    name during the move PR, and the NEW name once the move is in merge-base history.
//!
//! No single compiled literal is correct for both. This crate replaces the movable location
//! consts with a stable logical [`PathId`] resolved through a move-aware resolver: the
//! CANDIDATE side is a compiled CURRENT-canonical seed (rebased NEW when a move lands, in the
//! same reviewed PR), and the MERGE-BASE side is driven by the de-committed move-manifest bijection
//! ([`MoveBijection`]; ADR-0614: materialize-on-demand, never a contributor merge surface) over the
//! immutable git history ([`FrozenRefSource`]).
//!
//! SECURITY (why this cannot launder ratchet debt): the merge-base resolution reads the frozen
//! reference from IMMUTABLE MERGE-BASE HISTORY, at a name chosen by a TRUSTED COMPILED SEED
//! (never a candidate-tree data file — MUST-PASS #1) bridged by a manifest that is itself
//! REGENERATED-FROM-THE-COMMITTED-PLAN-BEFORE-READ (registry-drift regenerate-twice determinism
//! binds it to the codemod's deterministic output, so a hand-forged pair REDs before the firewall
//! consumes the snapshot). It is NOT a gate-ordering / registry-drift trick and NOT a candidate
//! fallback: a name the manifest declares but that is absent from BOTH sides of history is a HARD
//! ERROR, never an empty/bootstrap/candidate fallback (that is the laundering vector this crate
//! exists to close).
#![forbid(unsafe_code)]

/// A logical, move-stable identity for one self-referential gate self-location. The variant —
/// never a filesystem literal — is what the gate code names; the resolver maps it to a concrete
/// repo-relative path per tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathId {
    /// The merge-base ratchet policy (`ratchet-policy.json`). The ONLY self-ref path with a
    /// candidate-vs-merge-base split: candidate `repo_root.join` read of the local policy
    /// (supplies `out_path`) vs the frozen `show_file(merge_base, .)` read (frozen-policy-wins).
    RatchetPolicy,
    /// The de-committed accounting `scm-facts.generated.json` face (candidate WRITE target — the
    /// emitter's `--out` default; materialize-on-demand, not-tracked-in-git). Candidate-only; never
    /// read at the merge-base.
    ScmFactsFace,
    /// The untracked, gitignored `scm-volatile-facts.generated.json` snapshot (candidate WRITE
    /// target — the emitter's `--volatile-out` default). Candidate-only.
    VolatileFacts,
}

impl PathId {
    /// Every declared id (for exhaustive seed-table + injectivity self-tests).
    pub const ALL: &'static [PathId] = &[
        PathId::RatchetPolicy,
        PathId::ScmFactsFace,
        PathId::VolatileFacts,
    ];
}

/// The COMPILED, CURRENT-canonical (post-move NEW) repo-relative path for each [`PathId`].
///
/// MUST-PASS #1: this seed is COMPILED-IN — it is NEVER read from a candidate-tree data file, so
/// there is no candidate-repointable anchor an attacker could aim the merge-base read through.
/// The value is the file's CURRENT canonical location: when a strangler move lands, this const is
/// rebased OLD->NEW in the SAME reviewed PR that lands the move (a code edit, the same review class
/// as editing the workflow), and the materialized (de-committed, ADR-0614) move-manifest supplies
/// the NEW->OLD bridge that the merge-base read needs for the single PR whose merge-base still
/// predates the move.
pub const fn canonical_current(id: PathId) -> &'static str {
    match id {
        PathId::RatchetPolicy => "ci/facade/baseline-ratchet/ratchet-policy.json",
        PathId::ScmFactsFace => {
            "ci/facade/artifact-inventory-registry/scm-facts.generated.json"
        }
        PathId::VolatileFacts => {
            "ci/facade/scm-facts-snapshot/scm-volatile-facts.generated.json"
        }
    }
}

/// The name a file bears at the merge-base: [`Present`](MergeBaseName::Present) with the
/// historical name to read, or [`Absent`](MergeBaseName::Absent) (genuine repo bootstrap — the
/// PR that introduces the file; feeds the existing `missing_at_merge_base` path unchanged).
///
/// There is DELIBERATELY no third "empty/fallback" variant: a name the manifest DECLARES but that
/// is absent from history is a HARD ERROR (`Result::Err`), never a silent empty reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeBaseName {
    Present(String),
    Absent,
}

/// The old<->new bijection loaded from the de-committed move-manifest (materialized on demand).
/// FAIL-CLOSED EMPTY => identity (no pending move). Injective on BOTH sides (MUST-PASS #3):
/// `new_to_old` must be a well-defined function, so a manifest whose new-side is non-injective is
/// rejected to the empty bijection.
pub trait MoveBijection {
    /// The pre-move OLD path a pending move declares for `new_path`; `None` => not a pending move.
    fn new_to_old(&self, new_path: &str) -> Option<&str>;
    /// The post-move NEW path a pending move declares for `old_path`; `None` => not a pending move.
    fn old_to_new(&self, old_path: &str) -> Option<&str>;
}

/// The narrow git seam the merge-base resolution needs (ADR-0515 D3 single sanctioned git
/// boundary). Implemented by the live git CLI in ci/adapters and by fakes in the must-pass unit
/// tests, so the fail-closed resolution is pinned by executable attack recipes.
pub trait FrozenRefSource {
    /// `git merge-base <base_ref> HEAD` (full hex revision id; hard error on failure).
    fn merge_base(&self, base_ref: &str) -> Result<String, String>;
    /// File content at `<revision>:<path>`; `Ok(None)` iff the path is absent there.
    fn show_file(&self, revision: &str, path: &str) -> Result<Option<String>, String>;
}

/// Resolve a movable gate self-location to a concrete repo-relative path per tree.
pub trait PathResolver {
    /// The CANDIDATE (working-tree) location = the file's current canonical path (post-move NEW).
    /// Used for `repo_root.join(..)` reads and writes.
    fn candidate(&self, id: PathId) -> String;
    /// The name `id` bears AT `merge_base`. Manifest-driven, PRESENCE-VERIFIED in immutable
    /// history, fail-closed (a declared-but-doubly-absent or ambiguously-doubly-present name is a
    /// HARD ERROR — never an empty/candidate/bootstrap fallback).
    fn at_merge_base(
        &self,
        id: PathId,
        merge_base: &str,
        src: &dyn FrozenRefSource,
    ) -> Result<MergeBaseName, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MUST-PASS #1 support: every declared id has a compiled, non-empty, NEW-namespace seed, and
    /// the seed table is injective (no two ids alias the same path). Compiled — no file is read.
    #[test]
    fn seed_table_is_total_injective_and_new_namespace() {
        let mut seen = std::collections::BTreeSet::new();
        for &id in PathId::ALL {
            let p = canonical_current(id);
            assert!(!p.is_empty(), "seed for {id:?} is empty");
            assert!(
                p.starts_with("ci/"),
                "seed for {id:?} is not in the NEW ci/ namespace: {p:?}"
            );
            assert!(
                !p.contains("cloud/cloud-ci"),
                "seed for {id:?} still points at the OLD location: {p:?}"
            );
            assert!(seen.insert(p), "seed table aliases {p:?} across two ids");
        }
    }
}
