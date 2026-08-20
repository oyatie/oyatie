//! # cloud-ci-firewall (GO-LIVE readiness ratchet)
//!
//! The single required status check for the Phase-0 firewall (PHASE-0-FIREWALL-PLAN
//! go-live readiness; register #20). The four born-blocking gates each prove they DETECT
//! the live exhibit (they go RED on today's corpus). This crate layers the gate-baseline
//! face as the SECOND predicate so the firewall blocks only NEW debt, not the frozen
//! pre-existing corpus debt.
//!
//! Two PURE, DATA-over-DATA predicates (no per-code special cases — the per-code behaviour
//! differences live entirely in the baseline DATA: the `mode` + `frozen_empty` fields).
//!
//! THE FROZEN REFERENCE IS THE MERGE-BASE BASELINE (FRIC-1781112000 / ADR-0551). Both
//! predicates compare against the gate-baseline face as committed at
//! `git merge-base <base_ref> HEAD` (base_ref is policy DATA in `ratchet-policy.json`,
//! default `origin/dev`), NEVER against the working-tree copy: the settle protocol mandates
//! producer regeneration and registry-drift mandates committed==regenerated, so the
//! PR-local face always equals the proposed face and a PR-local reference can be grown by
//! the very regen the protocol requires (the PR #670 laundering exhibit). The merge-base
//! snapshot is materialized out-of-graph by the scm-facts emitter (the single sanctioned
//! git boundary) into `gate-baseline.merge-base.generated.json`; this crate only parses it
//! ([`FrozenBaseline::from_value`]). Standard ratchet semantics: Betterer / eslint-ratchet
//! compare against the merge-base; Bazel target determination anchors on the merge-base.
//!
//! FROZEN-POLICY-WINS (FRIC-1781280000, ADR-0551 hardening): the policy facts that SELECT
//! the frozen reference (`base_ref`, `face_path`) are themselves read from the MERGE-BASE
//! tree by the emitter, never from the candidate tree — a same-PR `"base_ref": "HEAD"`
//! repoint (merge-base(HEAD, HEAD) = HEAD ⇒ frozen == proposed ⇒ total self-laundering)
//! cannot affect the PR's own frozen reference; it can only change FUTURE behavior after
//! merge. The snapshot declares `frozen_policy_source` and this parser rejects an
//! undeclared source fail-closed. Prow precedent: OWNERS files are read from the base
//! branch, never the PR head, for exactly this reason.
//!
//! INERT-DOOR DETECTOR (FRIC-1781280001; symmetrized FRIC-1781460000): a sign-off entry
//! whose key is absent from the CANDIDATE tree (current AND proposed) is a standing
//! re-introduction ticket and FAILS the firewall until retired — the mechanical expiry of
//! the "exempt for one regen" door ([`inert_signoff_entries`]). Inert-ness is read against
//! the candidate, NOT the frozen merge-base face: a PR that orphans a live entry (e.g. adds
//! an OWNERS file that resolves a previously-unowned key) FAILS CLOSED at PR time, symmetric
//! with the push tier on the integrated tip — closing the FRIC-1781460000 false-green.
//!
//! COMPARE-MODE — for each `(gate, code)` it computes `regressions = current_keys \
//! frozen_keys \ signed_off` (NEW debt), `signed_off = (current_keys \ frozen_keys) ∩
//! sign-off-door` (founder-admitted growth in flight — reported, no fail), `tolerated =
//! current_keys ∩ frozen_keys` (accepted pre-existing debt — no fail), and `fixed =
//! frozen_keys \ current_keys` (repaired — informational, drives shrink). `FAIL_for_code`
//! is `!regressions.is_empty()` for `baseline-block-on-new`, and always `false` for
//! `advisory-until-infra`. The gate FAILs iff any code FAILs. Advisory codes still EMIT
//! their counts (the burn-down dashboard) but never flip the verdict until the disposition
//! is flipped to `baseline-block-on-new` (a DATA edit, not a code change). The per-code
//! `mode` is read from the FROZEN baseline (merge-base DATA a PR cannot rewrite); only a
//! code absent at the merge-base falls back to the default blocking mode.
//!
//! RATCHET-INVARIANT — a `baseline-block-on-new` baseline may only ever SHRINK on regen.
//! For each blocking `(gate, code)`, `growth = proposed_keys \ frozen_keys` (keys a regen
//! would ADD relative to the merge-base). Empty growth is an allowed regen (auto-shrinks to
//! `frozen ∩ proposed`). Non-empty growth is a `ratchet_regression` FAILURE unless every
//! grown key is in the founder-signed `_sign_off_additions` allowlist
//! (`gate-baseline.signoff.json`, the ONE-WAY DOOR — a human-edited, NOT
//! producer-generated file). `frozen_empty` codes have a permanently-empty frozen baseline,
//! so ANY proposed key is growth — they can never accumulate a baseline. Same predicate,
//! no special case. `advisory-until-infra` codes (mode read from the FROZEN baseline; a
//! code absent at the merge-base uses its proposed stamp) are exempt from the growth check:
//! their keys exist only for the burn-down dashboard, every PR adding any file legitimately
//! grows them (born-unowned/unreachable), and their baseline is frozen by the reviewed DATA
//! edit that flips the disposition — growth-blocking them would turn every PR into a
//! signoff.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub mod run_observability_packet;
pub mod run_terminal_state;

/// The verdict-name reused for a backwards ratchet (both the GATE-4 row-level downgrade and
/// a baseline-growth at regen mean "the ratchet went backwards").
pub const RATCHET_REGRESSION: &str = "ratchet_regression";

/// The schema id of the merge-base frozen-baseline snapshot the scm-facts emitter
/// materializes (`gate-baseline.merge-base.generated.json`). Bumped only on a breaking
/// shape change. v2 (FRIC-1781280000 frozen-policy-wins): the snapshot must declare
/// `frozen_policy_source` — WHERE the policy facts that selected the frozen reference were
/// read from. A stale v1 snapshot is rejected fail-closed (re-run
/// `buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`).
pub const FROZEN_BASELINE_SCHEMA: &str = "oya-ci/merge-base-baseline/v2";

/// `frozen_policy_source` value: the ratchet policy was read from the MERGE-BASE tree (the
/// frozen-policy-wins normal path — a same-PR candidate policy edit cannot select the
/// frozen reference).
pub const FROZEN_POLICY_SOURCE_MERGE_BASE: &str = "merge-base";

/// `frozen_policy_source` value: the policy did not exist at the merge-base (the PR that
/// introduces the ratchet itself), so the candidate policy was used — the DECLARED
/// bootstrap path, review-visible in the provenance.
pub const FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP: &str = "candidate-bootstrap";

/// The compare-mode that blocks NEW debt; the only mode whose baseline is growth-protected.
pub const MODE_BASELINE_BLOCK_ON_NEW: &str = "baseline-block-on-new";

/// The compare-mode that reports debt but does not fail until prerequisite infra lands.
pub const MODE_ADVISORY_UNTIL_INFRA: &str = "advisory-until-infra";

/// Repo-relative path of the sign-off door file — the SINGLE owner of this path: the gate
/// test and the signoff fixer both consume it from here.
pub const SIGNOFF_PATH: &str = "ci/facade/baseline-ratchet/gate-baseline.signoff.json";

/// Repo-relative path of the committed ratchet policy (candidate copy; the FROZEN copy at
/// the merge-base is what selects the frozen reference — FRIC-1781280000).
pub const RATCHET_POLICY_PATH: &str = "ci/facade/baseline-ratchet/ratchet-policy.json";

/// Repo-relative path of the untracked merge-base frozen-baseline snapshot.
pub const FROZEN_SNAPSHOT_PATH: &str =
    "ci/facade/baseline-ratchet/gate-baseline.merge-base.generated.json";

/// The exact remediation command the gate prints when the inert-door detector fires
/// (automation-default, founder directive 2026-06-12: red-gating alone is not enough — a
/// mechanically-derivable retirement ships as a fixer, the gate is the backstop).
pub const SIGNOFF_FIXER_COMMAND: &str = "buck2 run \
//ci/facade/baseline-ratchet:oya-cloud-ci-firewall-signoff-fixer -- --fix";

/// A baseline: `gate -> code -> (mode, frozen_empty, keys)`. Parsed from the merge-base
/// frozen snapshot (the reference) or from a freshly-regenerated face (proposed).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    pub gates: BTreeMap<String, BTreeMap<String, CodeBaseline>>,
}

/// The frozen state of one `(gate, code)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeBaseline {
    pub mode: String,
    pub frozen_empty: bool,
    pub keys: BTreeSet<String>,
    /// The exact registration edit (or the precise design decision needed) printed when
    /// this code FAILs — stamped from the disposition DATA by the producer (ADR-0555: an
    /// unaccounted artifact is never a bare flag). Optional: older faces lack it.
    pub remediation: Option<String>,
}

impl Baseline {
    /// Parse a `gate-baseline.generated.json` `Value` into the typed baseline.
    ///
    /// FAIL-CLOSED: the firewall consumes producer output as control-plane DATA. A malformed
    /// baseline cannot silently collapse into "empty", "blocking by default", or a typo-mode
    /// that behaves like advisory; the parse error is the gate failure.
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let mut gates: BTreeMap<String, BTreeMap<String, CodeBaseline>> = BTreeMap::new();
        let gate_obj = value
            .get("gates")
            .and_then(Value::as_object)
            .ok_or("baseline missing object field gates")?;
        for (gate, codes) in gate_obj {
            let codes_obj = codes
                .as_object()
                .ok_or_else(|| format!("baseline gate {gate:?} codes must be an object"))?;
            let mut code_map: BTreeMap<String, CodeBaseline> = BTreeMap::new();
            for (code, entry) in codes_obj {
                let entry_obj = entry
                    .as_object()
                    .ok_or_else(|| format!("baseline entry {gate:?}/{code:?} must be an object"))?;
                let mode = entry_obj
                    .get("mode")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        format!("baseline entry {gate:?}/{code:?} missing string mode")
                    })?;
                if mode != MODE_BASELINE_BLOCK_ON_NEW && mode != MODE_ADVISORY_UNTIL_INFRA {
                    return Err(format!(
                        "baseline entry {gate:?}/{code:?} has unknown mode {mode:?}; \
                         expected {MODE_BASELINE_BLOCK_ON_NEW:?} or {MODE_ADVISORY_UNTIL_INFRA:?}"
                    ));
                }

                let frozen_empty = match entry_obj.get("frozen_empty") {
                    Some(value) => value.as_bool().ok_or_else(|| {
                        format!("baseline entry {gate:?}/{code:?} frozen_empty must be bool")
                    })?,
                    None => false,
                };

                let keys_value = entry_obj
                    .get("keys")
                    .ok_or_else(|| format!("baseline entry {gate:?}/{code:?} missing keys"))?;
                let keys_array = keys_value.as_array().ok_or_else(|| {
                    format!("baseline entry {gate:?}/{code:?} keys must be an array")
                })?;
                let mut keys = BTreeSet::new();
                for (index, key) in keys_array.iter().enumerate() {
                    let key = key.as_str().ok_or_else(|| {
                        format!(
                            "baseline entry {gate:?}/{code:?} key at index {index} must be string"
                        )
                    })?;
                    if key.is_empty() {
                        return Err(format!(
                            "baseline entry {gate:?}/{code:?} key at index {index} is empty"
                        ));
                    }
                    if !keys.insert(key.to_owned()) {
                        return Err(format!(
                            "baseline entry {gate:?}/{code:?} duplicate key {key:?}"
                        ));
                    }
                }
                let remediation = match entry_obj.get("remediation") {
                    Some(value) => Some(
                        value
                            .as_str()
                            .ok_or_else(|| {
                                format!(
                                    "baseline entry {gate:?}/{code:?} remediation must be string"
                                )
                            })?
                            .to_owned(),
                    ),
                    None => None,
                };
                code_map.insert(
                    code.clone(),
                    CodeBaseline {
                        mode: mode.to_owned(),
                        frozen_empty,
                        keys,
                        remediation,
                    },
                );
            }
            gates.insert(gate.clone(), code_map);
        }
        Ok(Self { gates })
    }
}

/// The parsed merge-base frozen-baseline snapshot: the gate-baseline face exactly as
/// committed at `git merge-base <base_ref> HEAD`, wrapped with the provenance needed to
/// audit WHICH frozen point the firewall compared against. Materialized out-of-graph by
/// the scm-facts emitter (the single sanctioned git boundary); this parser is pure and
/// FAILS CLOSED: a missing/foreign schema, a malformed merge-base id, or an empty
/// baseline that does not declare `missing_at_merge_base` is an error, never a silent
/// empty reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenBaseline {
    /// The FROZEN policy base ref the merge-base was computed against (e.g. `origin/dev`).
    /// FROZEN-POLICY-WINS (FRIC-1781280000): this comes from the ratchet policy as
    /// committed at the merge-base (or the out-of-band bootstrap it must agree with), never
    /// from the candidate tree — a same-PR `base_ref` repoint cannot select the PR's own
    /// frozen reference.
    pub base_ref: String,
    /// The merge-base revision id the face was extracted from (full hex sha).
    pub merge_base: String,
    /// WHERE the frozen-side policy facts were read from: [`FROZEN_POLICY_SOURCE_MERGE_BASE`]
    /// (normal) or [`FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP`] (policy absent at the
    /// merge-base — the declared bootstrap path). Any other value is rejected fail-closed.
    pub frozen_policy_source: String,
    /// True iff the face did not exist at the merge-base (repo bootstrap): the frozen
    /// reference is then EMPTY, so every proposed key is growth until signed off.
    pub missing_at_merge_base: bool,
    /// The frozen reference baseline (empty when `missing_at_merge_base`).
    pub baseline: Baseline,
}

impl FrozenBaseline {
    /// Parse + validate the snapshot. Every rejection names the defect (fail-closed).
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let schema = value.get("schema").and_then(Value::as_str).unwrap_or("");
        if schema != FROZEN_BASELINE_SCHEMA {
            return Err(format!(
                "frozen baseline schema mismatch: expected {FROZEN_BASELINE_SCHEMA:?}, got \
                 {schema:?} — re-materialize the snapshot: \
                 buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root ."
            ));
        }
        let base_ref = value
            .get("base_ref")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if base_ref.is_empty() {
            return Err("frozen baseline missing base_ref".to_owned());
        }
        let merge_base = value
            .get("merge_base")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if merge_base.len() < 40 || !merge_base.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "frozen baseline merge_base is not a full hex revision id: {merge_base:?}"
            ));
        }
        let frozen_policy_source = value
            .get("frozen_policy_source")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if frozen_policy_source != FROZEN_POLICY_SOURCE_MERGE_BASE
            && frozen_policy_source != FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP
        {
            return Err(format!(
                "frozen baseline frozen_policy_source must be \
                 {FROZEN_POLICY_SOURCE_MERGE_BASE:?} or \
                 {FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP:?}, got {frozen_policy_source:?} \
                 — the snapshot must DECLARE where the frozen-side policy facts came from \
                 (FRIC-1781280000 frozen-policy-wins; fail-closed)"
            ));
        }
        let missing_at_merge_base =
            value.get("missing_at_merge_base").and_then(Value::as_bool) == Some(true);
        let baseline = match value.get("baseline") {
            Some(value) => Baseline::from_value(value)
                .map_err(|e| format!("frozen baseline embedded baseline malformed: {e}"))?,
            None => Baseline::default(),
        };
        if !missing_at_merge_base && baseline.gates.is_empty() {
            return Err(
                "frozen baseline carries no gates yet does not declare missing_at_merge_base \
                 — refusing the empty reference (fail-closed)"
                    .to_owned(),
            );
        }
        if missing_at_merge_base && !baseline.gates.is_empty() {
            return Err(
                "frozen baseline declares missing_at_merge_base but carries gates".to_owned(),
            );
        }
        // PROVENANCE (ADR-0616): the frozen reference is REGENERATED from the merge-base source
        // (`git rev-parse <merge_base>^{tree}` over the merge-base tree), NOT read from a committed
        // git blob. With no committed bytes to trust, the snapshot must carry provenance binding the
        // regeneration to the immutable merge-base tree. The firewall NEVER calls git, so it cannot
        // recompute the tree; it fail-closed VERIFIES that the emitter-computed provenance is present,
        // that `base_tree_sha` is a well-formed tree id, and that the provenance is bound to THIS
        // snapshot's `merge_base` (a provenance lifted from a different merge-base is rejected).
        // Cryptographic signing of this provenance is a fleet-wide follow-on (ADR-0616 §Trust
        // ceiling); this parser verifies the attestable facts a signer would later bind.
        Self::verify_provenance(value, &merge_base)?;
        Ok(Self {
            base_ref,
            merge_base,
            frozen_policy_source,
            missing_at_merge_base,
            baseline,
        })
    }

    /// Fail-closed provenance verification (ADR-0616): the snapshot must carry a `provenance`
    /// object with a well-formed `base_tree_sha` bound to this snapshot's `merge_base`.
    fn verify_provenance(value: &Value, merge_base: &str) -> Result<(), String> {
        let provenance = value.get("provenance").and_then(Value::as_object).ok_or(
            "frozen baseline missing provenance object (ADR-0616: the frozen reference is \
             regenerated from the merge-base source and must carry regeneration provenance binding \
             it to the merge-base tree; fail-closed) — re-materialize the snapshot: \
             buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .",
        )?;
        let base_tree_sha = provenance
            .get("base_tree_sha")
            .and_then(Value::as_str)
            .unwrap_or("");
        if base_tree_sha.len() < 40 || !base_tree_sha.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!(
                "frozen baseline provenance base_tree_sha is not a full hex tree id: \
                 {base_tree_sha:?} (fail-closed)"
            ));
        }
        let provenance_merge_base = provenance
            .get("merge_base")
            .and_then(Value::as_str)
            .unwrap_or("");
        if provenance_merge_base != merge_base {
            return Err(format!(
                "frozen baseline provenance merge_base {provenance_merge_base:?} disagrees with \
                 the snapshot merge_base {merge_base:?} — the regeneration provenance must be bound \
                 to THIS merge-base (fail-closed)"
            ));
        }
        Ok(())
    }
}

/// The sign-off allowlist (`gate-baseline.signoff.json`): the ONE-WAY DOOR. A key listed
/// under `_sign_off_additions[gate][code]` is exempted from the GROWTH check (and reported
/// as `signed_off`, not a regression, in compare-mode) for one regen — once the admitted
/// key lands in the merge-base baseline the entry is inert and retirable. This file is
/// human-edited + founder-signed, NOT producer-generated.
#[derive(Debug, Clone, Default)]
pub struct SignOff {
    additions: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

impl SignOff {
    pub fn from_value(value: &Value) -> Self {
        let mut additions: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
        if let Some(gate_obj) = value.get("_sign_off_additions").and_then(Value::as_object) {
            for (gate, codes) in gate_obj {
                let mut code_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
                if let Some(codes_obj) = codes.as_object() {
                    for (code, keys) in codes_obj {
                        code_map.insert(code.clone(), str_set(Some(keys)));
                    }
                }
                additions.insert(gate.clone(), code_map);
            }
        }
        Self { additions }
    }

    fn is_signed_off(&self, gate: &str, code: &str, key: &str) -> bool {
        self.additions
            .get(gate)
            .and_then(|codes| codes.get(code))
            .is_some_and(|keys| keys.contains(key))
    }

    /// Every `(gate, code, key)` entry in the door, in deterministic order — the input to
    /// the inert-entry detector ([`inert_signoff_entries`]).
    pub fn entries(&self) -> Vec<(String, String, String)> {
        let mut out: Vec<(String, String, String)> = Vec::new();
        for (gate, codes) in &self.additions {
            for (code, keys) in codes {
                for key in keys {
                    out.push((gate.clone(), code.clone(), key.clone()));
                }
            }
        }
        out
    }
}

/// The per-code compare-mode report. `current`/`baseline` are counts; the key sets are
/// carried so the failing PR sees EXACTLY which new unit it added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeReport {
    pub gate: String,
    pub code: String,
    pub mode: String,
    pub current: usize,
    pub baseline: usize,
    pub regressions: BTreeSet<String>,
    pub fixed: BTreeSet<String>,
    pub tolerated: BTreeSet<String>,
    /// Keys new relative to the frozen reference but founder-admitted through the one-way
    /// sign-off door: reported for the audit trail, never a failure.
    pub signed_off: BTreeSet<String>,
    /// The exact registration edit (or the precise design decision needed) for this code's
    /// regressions — carried from the PROPOSED face (freshest disposition DATA), falling
    /// back to the frozen reference (ADR-0555: a FAIL is never a bare flag).
    pub remediation: Option<String>,
}

impl CodeReport {
    /// A code FAILs iff its mode is baseline-block-on-new AND it has NEW (regression) keys.
    /// advisory-until-infra reports its counts but never fails (until the disposition flips).
    pub fn fails(&self) -> bool {
        self.mode == MODE_BASELINE_BLOCK_ON_NEW && !self.regressions.is_empty()
    }
}

/// The full firewall report: the compare-mode per-code reports + the ratchet-invariant
/// growth findings + the inert sign-off-door findings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FirewallReport {
    pub codes: Vec<CodeReport>,
    /// `(gate, code, key)` triples a regen would ADD to a blocking baseline, relative to
    /// the merge-base frozen reference, that are NOT signed off: each is a
    /// `ratchet_regression` (debt cannot be laundered into the baseline by regen — not
    /// even by the same-PR regen the settle protocol mandates).
    pub ratchet_growth: Vec<(String, String, String)>,
    /// `(gate, code, key)` sign-off-door entries that exempt NOTHING (key absent from current
    /// AND from proposed in the candidate tree — the merge-base frozen face is NOT consulted,
    /// so a PR's own OWNERS addition that orphans an entry fails at PR tier, symmetric with
    /// push): each is a standing re-introduction ticket and a failure — remediation: retire the
    /// entry (see [`inert_signoff_entries`]).
    pub inert_signoff: Vec<(String, String, String)>,
}

impl FirewallReport {
    /// GREEN iff no code FAILs (compare-mode), no un-signed-off baseline growth (ratchet),
    /// AND no inert sign-off-door entry (mechanical door expiry).
    pub fn is_green(&self) -> bool {
        !self.codes.iter().any(CodeReport::fails)
            && self.ratchet_growth.is_empty()
            && self.inert_signoff.is_empty()
    }
}

/// Relabel a FROZEN baseline's keys through a `old_path -> new_path` rename map.
///
/// WHY THIS EXISTS. The baseline is PATH-KEYED, so a pure `git mv` scores as
/// `regressions = N` at the destination paths and `fixed = N` at the sources. The
/// ratchet blocks on any new key regardless of offsetting fixes, so a relocation
/// that changes not one byte of content REDs the firewall. With ~250 crates still
/// to move out of the legacy `oya/` and `cloud/` roots under ADR-0562, that turns
/// every structural move into a founder sign-off — the door meant for genuinely
/// new debt. Relabelling makes a move behave EXACTLY like an in-place edit: the
/// debt follows the file to its new address.
///
/// THIS IS NOT LAUNDERING — it is strictly more faithful tracking. The violation
/// is neither dropped nor exempted; it is re-keyed to where the file now lives, so
/// it stays in `tolerated` and still has to be burned down. The laundering
/// direction is the opposite one (dropping a baseline row), which this never does:
/// the returned baseline has the same key COUNT per code as the input.
///
/// Two guards keep it honest:
///
/// - EXISTENCE — a rename whose source carries no baselined violation is ignored.
///   Inventing a row at the destination would pre-tolerate real new debt landing
///   there.
/// - NON-COLLISION — a rename onto a key that is ALREADY baselined under the same
///   code is refused, leaving both keys intact. Merging two debt rows into one
///   would shrink the baseline with no burn-down, which is exactly the silent
///   weakening the ratchet exists to prevent.
///
/// Mode, `frozen_empty` and `remediation` are carried through untouched: they are
/// merge-base DATA, and moving a file must not let a PR rewrite them.
/// Derive DIRECTORY renames implied by a set of FILE renames.
///
/// The firewall baseline is path-keyed, and some gates key their debt by crate
/// DIRECTORY (`cloud-ci-target-parity` keys a member dir, not a file). Git reports
/// only file-level renames, so a directory that moved wholesale leaves those keys
/// unrelabelled: the move scores as a regression at the destination and a fix at
/// the source — net-zero debt that still REDs the gate. This closes that gap in the
/// same spirit as the file-level relabel, and with the same conservatism.
///
/// A candidate `old_dir -> new_dir` is emitted only when all of:
///   * at least one file rename supports it, with an IDENTICAL relative subpath;
///   * every file rename leaving `old_dir` agrees on that same `new_dir` — a
///     directory whose files scattered emits nothing;
///   * nothing tracked remains under `old_dir` at HEAD, so the directory moved
///     WHOLESALE rather than partially.
///
/// Each condition fails toward emitting NOTHING, which leaves the ratchet blocking.
/// The caller then applies the existence and non-collision guards, so a spurious
/// pair still cannot shrink the baseline.
pub fn derive_directory_renames(
    file_renames: &BTreeMap<String, String>,
    head_paths: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    derive_directory_renames_scored(file_renames, &BTreeMap::new(), head_paths)
}

/// As [`derive_directory_renames`], but able to break a tie using git's similarity score.
///
/// Scaffolded products make this necessary. A generated `Cargo.toml` or `BUCK` is nearly
/// identical across crates, so over a large diff git's best-match pairs them ACROSS crates:
/// one real move was reported with its `Cargo.toml` paired to a different product's crate,
/// its `BUCK` to a third, and only `src/lib.rs` paired correctly — at R100, byte-identical.
/// Unanimity alone therefore rejects moves that plainly happened.
///
/// The scores separate the cases: a byte-identical pairing is near-proof, an 80% match on
/// boilerplate is not. A destination also wins if it is the STRICT maximum — ties still emit
/// nothing, so this only ever resolves cases unanimity would have abandoned, and never
/// overrides an unambiguous one. `scores` maps a renamed source path to git's `R<score>`
/// value; absent entries score zero, which is why the unscored wrapper keeps the old
/// behaviour exactly.
pub fn derive_directory_renames_scored(
    file_renames: &BTreeMap<String, String>,
    scores: &BTreeMap<String, u32>,
    head_paths: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    // old_dir -> new_dir -> how many file renames support that exact pairing.
    let mut candidates: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    // old_dir -> new_dir -> the BEST similarity score among the renames supporting it.
    let mut best: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    // old_dir -> how many file renames leave it at all, by any destination.
    let mut departures: BTreeMap<String, usize> = BTreeMap::new();

    for (old, new) in file_renames {
        let old_parts: Vec<&str> = old.split('/').collect();
        let new_parts: Vec<&str> = new.split('/').collect();
        // Longest identical trailing subpath: the part of the path the move preserved.
        let mut common = 0usize;
        while common < old_parts.len().min(new_parts.len())
            && old_parts[old_parts.len() - 1 - common] == new_parts[new_parts.len() - 1 - common]
        {
            common += 1;
        }
        for k in 1..=common {
            let old_dir = old_parts[..old_parts.len() - k].join("/");
            let new_dir = new_parts[..new_parts.len() - k].join("/");
            if old_dir.is_empty() || new_dir.is_empty() || old_dir == new_dir {
                continue;
            }
            let score = scores.get(old).copied().unwrap_or(0);
            let slot = best
                .entry(old_dir.clone())
                .or_default()
                .entry(new_dir.clone())
                .or_default();
            *slot = (*slot).max(score);
            *candidates
                .entry(old_dir)
                .or_default()
                .entry(new_dir)
                .or_default() += 1;
        }
        // Count this rename against every ancestor dir it left, whether or not the
        // relative subpath was preserved — a scattered file must still veto its parent.
        for depth in 1..old_parts.len() {
            *departures.entry(old_parts[..depth].join("/")).or_default() += 1;
        }
    }

    let mut out = BTreeMap::new();
    for (old_dir, destinations) in candidates {
        let new_dir = if destinations.len() == 1 {
            // Unanimous: one destination accounting for EVERY departure from old_dir.
            let (only, supporting) = destinations.into_iter().next().expect("one destination");
            if departures.get(&old_dir).copied().unwrap_or(0) != supporting {
                continue;
            }
            only
        } else {
            // Disagreement. Resolve it ONLY on a strict maximum similarity — a tie, or no
            // scores at all, still emits nothing.
            let Some(ranked) = best.get(&old_dir) else {
                continue;
            };
            let mut by_score: Vec<(&String, u32)> =
                ranked.iter().map(|(dir, score)| (dir, *score)).collect();
            by_score.sort_by(|left, right| right.1.cmp(&left.1));
            match by_score.as_slice() {
                [(winner, top), (_, runner_up), ..] if top > runner_up => (*winner).clone(),
                _ => continue,
            }
        };
        // WHOLESALE: nothing tracked may remain behind.
        let prefix = format!("{old_dir}/");
        if head_paths.iter().any(|path| path.starts_with(&prefix)) {
            continue;
        }
        out.insert(old_dir, new_dir);
    }
    out
}

pub fn relabel_baseline_for_renames(
    frozen: &Baseline,
    renames: &BTreeMap<String, String>,
) -> Baseline {
    if renames.is_empty() {
        return frozen.clone();
    }
    let mut out = frozen.clone();
    for codes in out.gates.values_mut() {
        for code_baseline in codes.values_mut() {
            let mut keys = code_baseline.keys.clone();
            for (old, new) in renames {
                // EXISTENCE guard: only relabel debt that actually exists here.
                if !keys.contains(old) {
                    continue;
                }
                // NON-COLLISION guard: never merge two baselined rows.
                if keys.contains(new) {
                    continue;
                }
                keys.remove(old);
                keys.insert(new.clone());
            }
            code_baseline.keys = keys;
        }
    }
    out
}

/// COMPARE-MODE predicate: compare the current keyed violations against the FROZEN
/// (merge-base) baseline, per `(gate, code)`. `current` is `gate -> code -> keys` (from
/// running each gate's `evaluate_keyed` over the live faces). Keys new relative to the
/// frozen reference but listed in the sign-off door are `signed_off`, not regressions.
pub fn compare(
    frozen: &Baseline,
    current: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    signoff: &SignOff,
) -> Vec<CodeReport> {
    let mut reports: Vec<CodeReport> = Vec::new();
    // Union of gate/code keys present in either the baseline or the current set, so a code
    // that exists only in one side is still reported.
    for gate in union_keys(frozen.gates.keys(), current.keys()) {
        let base_codes = frozen.gates.get(&gate);
        let cur_codes = current.get(&gate);
        let codes = union_keys(
            base_codes.into_iter().flat_map(BTreeMap::keys),
            cur_codes.into_iter().flat_map(BTreeMap::keys),
        );
        for code in codes {
            let base = base_codes.and_then(|c| c.get(&code));
            let baseline_keys: BTreeSet<String> = base.map(|b| b.keys.clone()).unwrap_or_default();
            // The mode comes from the FROZEN reference (merge-base DATA a PR cannot
            // rewrite); a code absent at the merge-base defaults to blocking.
            let mode = base
                .map(|b| b.mode.clone())
                .unwrap_or_else(|| MODE_BASELINE_BLOCK_ON_NEW.to_owned());
            let current_keys: BTreeSet<String> = cur_codes
                .and_then(|c| c.get(&code))
                .cloned()
                .unwrap_or_default();

            let mut regressions: BTreeSet<String> = BTreeSet::new();
            let mut signed_off: BTreeSet<String> = BTreeSet::new();
            for key in current_keys.difference(&baseline_keys) {
                if signoff.is_signed_off(&gate, &code, key) {
                    signed_off.insert(key.clone());
                } else {
                    regressions.insert(key.clone());
                }
            }
            let fixed: BTreeSet<String> =
                baseline_keys.difference(&current_keys).cloned().collect();
            let tolerated: BTreeSet<String> =
                current_keys.intersection(&baseline_keys).cloned().collect();

            reports.push(CodeReport {
                gate: gate.clone(),
                code: code.clone(),
                mode,
                current: current_keys.len(),
                baseline: baseline_keys.len(),
                regressions,
                fixed,
                tolerated,
                signed_off,
                remediation: base.and_then(|b| b.remediation.clone()),
            });
        }
    }
    reports
}

/// RATCHET-INVARIANT predicate: a blocking baseline may only SHRINK relative to the FROZEN
/// (merge-base) reference. `proposed` is what today's corpus WOULD freeze (the regenerated
/// baseline keys); `frozen` is the merge-base set — NEVER the PR-local face, which the
/// settle protocol itself regenerates (FRIC-1781112000). Any key the regen would ADD
/// (growth) that is not signed off is a `ratchet_regression`. `frozen_empty` codes have an
/// empty frozen set, so any proposed key is growth — the same predicate enforces "never
/// accumulate a baseline" for them. `advisory-until-infra` codes (mode from the frozen
/// reference; proposed stamp only for codes new at the merge-base) are exempt: their
/// dashboards grow with every added file and their freeze happens at the reviewed
/// disposition flip.
pub fn ratchet_growth(
    frozen: &Baseline,
    proposed: &Baseline,
    signoff: &SignOff,
) -> Vec<(String, String, String)> {
    let mut growth: Vec<(String, String, String)> = Vec::new();
    for (gate, proposed_codes) in &proposed.gates {
        for (code, proposed_code) in proposed_codes {
            let frozen_code = frozen.gates.get(gate).and_then(|c| c.get(code));
            // The FROZEN mode wins (a PR cannot retro-flip an existing blocking code to
            // advisory in the same change); only a code absent at the merge-base uses its
            // proposed stamp.
            let mode = frozen_code
                .map(|c| c.mode.as_str())
                .unwrap_or(proposed_code.mode.as_str());
            if mode != MODE_BASELINE_BLOCK_ON_NEW {
                continue;
            }
            let frozen_keys = frozen_code.map(|c| &c.keys);
            for key in &proposed_code.keys {
                let in_frozen = frozen_keys.is_some_and(|keys| keys.contains(key));
                if !in_frozen && !signoff.is_signed_off(gate, code, key) {
                    growth.push((gate.clone(), code.clone(), key.clone()));
                }
            }
        }
    }
    growth
}

/// INERT-DOOR detector (FRIC-1781280001 / ADR-0551 hardening; symmetrized FRIC-1781460000):
/// door entries never mechanically expired — "exempt for one regen" was unenforced prose, so
/// a lingering entry for a key the candidate no longer carries was a STANDING re-introduction
/// ticket: that exact debt key could come back at any time and the stale entry would launder
/// it past both predicates.
///
/// An entry is INERT iff its key is absent from the CANDIDATE tree — `key ∉ current AND
/// key ∉ proposed` — i.e. the debt it admitted does not exist in the change under
/// evaluation. The FROZEN (merge-base) face does NOT keep an entry alive: a key present in
/// the frozen face is tolerated/not-growth by [`compare`]/[`ratchet_growth`] regardless of
/// sign-off, so a frozen-present-but-candidate-absent entry exempts nothing in flight and is
/// a pure standing ticket. The LIVE entry — its key still in the candidate's debt set
/// (`current`/`proposed`), whether or not it is also frozen — is tolerated: it is either
/// admitting in-flight growth (`∉ frozen`) or covering still-present baselined debt
/// (`∈ frozen`), and in both cases the debt the door names still exists.
///
/// FRIC-1781460000 (the asymmetry this closes): the previous predicate ALSO required the key
/// to be absent from the FROZEN face — so frozen-presence VETOED the inert verdict. At PR
/// time the frozen face is the merge-base baseline, BEFORE the candidate's own additions, so
/// a PR that ADDS an OWNERS file (resolving a previously-unowned key) ORPHANS that key's
/// sign-off entry: the key drops out of the candidate's `current`/`proposed` set, but it is
/// STILL present in the (pre-OWNERS) frozen face, so the frozen veto judged the entry LIVE →
/// PR passed GREEN. Post-merge the merge-base advanced past the OWNERS addition, the key
/// dropped from the frozen face too, and the push-tier firewall went RED on this same
/// invariant → dev broke AFTER merge. Dropping the frozen veto reads inert-ness against the
/// CANDIDATE tree's ownership/reachability — which already includes the PR's own additions —
/// making PR-admission and push-admission SYMMETRIC: a candidate that orphans a live entry
/// FAILS CLOSED at PR time.
///
/// Per-(gate,code): the lookup is scoped — a key live in one code's candidate set but absent
/// from another's stays exempted under the live code and is only inert under the code where
/// the candidate no longer carries it. Remediation: retire the entry (move it to
/// `_sign_off_retirements`); the [`SIGNOFF_FIXER_COMMAND`] derives and applies this
/// mechanically.
pub fn inert_signoff_entries(
    frozen: &Baseline,
    proposed: &Baseline,
    current: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    signoff: &SignOff,
) -> Vec<(String, String, String)> {
    // `frozen` is intentionally unused: the FROZEN (merge-base) face no longer vetoes the
    // inert verdict (FRIC-1781460000). Inert-ness is read against the CANDIDATE tree alone
    // so PR-tier (merge-base frozen) and push-tier (integrated tip) agree. Kept in the
    // signature so the gate, the fixer, and the four-input call sites stay uniform.
    let _ = frozen;
    let in_proposed = |b: &Baseline, gate: &str, code: &str, key: &str| -> bool {
        b.gates
            .get(gate)
            .and_then(|codes| codes.get(code))
            .is_some_and(|cb| cb.keys.contains(key))
    };
    let in_current = |gate: &str, code: &str, key: &str| -> bool {
        current
            .get(gate)
            .and_then(|codes| codes.get(code))
            .is_some_and(|keys| keys.contains(key))
    };
    signoff
        .entries()
        .into_iter()
        .filter(|(gate, code, key)| {
            // INERT iff the key is absent from the candidate tree (current AND proposed):
            // the door admits nothing that exists in the change under evaluation. A
            // frozen-present key does NOT rescue the entry — it is tolerated/not-growth
            // regardless of sign-off, so the entry exempts nothing in flight.
            !in_proposed(proposed, gate, code, key) && !in_current(gate, code, key)
        })
        .collect()
}

/// Run BOTH predicates + the inert-door detector and assemble the full firewall report.
/// `frozen` is the merge-base reference baseline (see [`FrozenBaseline`]). Per-code
/// remediation text is enriched from the PROPOSED face when present (ADR-0555: the freshest
/// disposition DATA — a pre-flip frozen snapshot has none), falling back to the frozen entry
/// from `compare`, so a FAIL prints the exact registration edit rather than a bare flag.
pub fn evaluate_firewall(
    frozen: &Baseline,
    proposed: &Baseline,
    current: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    signoff: &SignOff,
) -> FirewallReport {
    let mut codes = compare(frozen, current, signoff);
    for report in &mut codes {
        if let Some(text) = proposed
            .gates
            .get(&report.gate)
            .and_then(|c| c.get(&report.code))
            .and_then(|cb| cb.remediation.clone())
        {
            report.remediation = Some(text);
        }
    }
    FirewallReport {
        codes,
        ratchet_growth: ratchet_growth(frozen, proposed, signoff),
        inert_signoff: inert_signoff_entries(frozen, proposed, current, signoff),
    }
}

/// The proposed baseline's per-code keys, as the compare-mode current map (for the case
/// where the live `current` IS the regenerated proposed face — the runner's normal path).
pub fn baseline_keys_map(
    baseline: &Baseline,
) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    let mut out: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    for (gate, codes) in &baseline.gates {
        let mut code_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (code, cb) in codes {
            code_map.insert(code.clone(), cb.keys.clone());
        }
        out.insert(gate.clone(), code_map);
    }
    out
}

fn str_set(value: Option<&Value>) -> BTreeSet<String> {
    value
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

fn union_keys<'a, A, B>(a: A, b: B) -> BTreeSet<String>
where
    A: IntoIterator<Item = &'a String>,
    B: IntoIterator<Item = &'a String>,
{
    a.into_iter().chain(b).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn baseline_fixture() -> Baseline {
        Baseline::from_value(&json!({
            "gates": {
                "cloud-ci-total-accounting": {
                    "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs"]},
                    "unowned": {"mode": "advisory-until-infra", "infra_prereq": "owners", "keys": ["a.rs"]},
                    "ci_inventory_registry_drift": {"mode": "baseline-block-on-new", "keys": [], "frozen_empty": true}
                }
            }
        })).unwrap()
    }

    fn current(
        pairs: &[(&str, &str, &[&str])],
    ) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
        let mut out: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
        for (gate, code, keys) in pairs {
            out.entry((*gate).to_owned()).or_default().insert(
                (*code).to_owned(),
                keys.iter().map(|k| (*k).to_owned()).collect(),
            );
        }
        out
    }

    // ── RENAME RELABEL (ADR-0562 structural moves) ──────────────────────────
    //
    // The baseline is PATH-KEYED. A pure `git mv` therefore reads as
    // `regressions = N` at the new paths and `fixed = N` at the old ones — net
    // zero debt, but the firewall blocks on new keys regardless of offsetting
    // fixes, so every crate move REDs the gate for no reason. These tests pin
    // the relabel that makes a move behave exactly like an in-place edit.

    #[test]
    fn pure_rename_without_relabel_is_a_false_regression() {
        // The DEFECT, pinned: relabelling nothing leaves the move looking like
        // brand-new debt. This is the pre-fix behaviour and must stay visible.
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["moved/a.rs", "b.rs"],
        )]);
        let reports = compare(&baseline_fixture(), &cur, &SignOff::default());
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert_eq!(
            unjust.regressions,
            ["moved/a.rs".to_owned()].into_iter().collect(),
            "an unrelabelled move must still surface as a regression"
        );
        assert_eq!(unjust.fixed, ["a.rs".to_owned()].into_iter().collect());
        assert!(unjust.fails());
    }

    fn files(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(old, new)| ((*old).to_owned(), (*new).to_owned()))
            .collect()
    }

    fn head(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|path| (*path).to_owned()).collect()
    }

    #[test]
    fn a_wholesale_directory_move_yields_the_directory_rename() {
        let renames = files(&[
            (
                "oya/gw/crates/oya-gw-github/src/lib.rs",
                "app/gw/adapters/github/src/lib.rs",
            ),
            (
                "oya/gw/crates/oya-gw-github/Cargo.toml",
                "app/gw/adapters/github/Cargo.toml",
            ),
        ]);
        let dirs =
            derive_directory_renames(&renames, &head(&["app/gw/adapters/github/Cargo.toml"]));
        assert_eq!(
            dirs.get("oya/gw/crates/oya-gw-github").map(String::as_str),
            Some("app/gw/adapters/github")
        );
    }

    #[test]
    fn a_strict_similarity_maximum_resolves_a_scaffold_mispairing() {
        // The real case: over a large diff git paired this crate's generated Cargo.toml to a
        // DIFFERENT product's crate and its BUCK to a third, because scaffold boilerplate is
        // near-identical. Only src/lib.rs paired correctly — byte-identical, R100.
        let renames = files(&[
            (
                "oya/app/crates/oya-surface/src/lib.rs",
                "app/x/core/surface/src/lib.rs",
            ),
            (
                "oya/app/crates/oya-surface/Cargo.toml",
                "app/payroll/core/run-domain/Cargo.toml",
            ),
            (
                "oya/app/crates/oya-surface/BUCK",
                "app/community/core/social-domain/BUCK",
            ),
        ]);
        let scores: BTreeMap<String, u32> = [
            ("oya/app/crates/oya-surface/src/lib.rs".to_owned(), 100u32),
            ("oya/app/crates/oya-surface/Cargo.toml".to_owned(), 81),
            ("oya/app/crates/oya-surface/BUCK".to_owned(), 80),
        ]
        .into_iter()
        .collect();
        let dirs = derive_directory_renames_scored(&renames, &scores, &BTreeSet::new());
        assert_eq!(
            dirs.get("oya/app/crates/oya-surface").map(String::as_str),
            Some("app/x/core/surface"),
            "the byte-identical pairing must win over two boilerplate mis-pairings"
        );
    }

    #[test]
    fn a_similarity_tie_still_emits_nothing() {
        let renames = files(&[
            ("oya/app/crates/oya-surface/a.rs", "app/x/core/surface/a.rs"),
            ("oya/app/crates/oya-surface/b.rs", "app/y/core/other/b.rs"),
        ]);
        let scores: BTreeMap<String, u32> = [
            ("oya/app/crates/oya-surface/a.rs".to_owned(), 90u32),
            ("oya/app/crates/oya-surface/b.rs".to_owned(), 90),
        ]
        .into_iter()
        .collect();
        let dirs = derive_directory_renames_scored(&renames, &scores, &BTreeSet::new());
        assert!(
            !dirs.contains_key("oya/app/crates/oya-surface"),
            "a tie is still ambiguous and must not be resolved"
        );
    }

    #[test]
    fn without_scores_disagreement_still_emits_nothing() {
        // The unscored wrapper must keep its original, stricter behaviour exactly.
        let renames = files(&[
            ("oya/app/crates/oya-surface/a.rs", "app/x/core/surface/a.rs"),
            ("oya/app/crates/oya-surface/b.rs", "app/y/core/other/b.rs"),
        ]);
        let dirs = derive_directory_renames(&renames, &BTreeSet::new());
        assert!(!dirs.contains_key("oya/app/crates/oya-surface"));
    }

    #[test]
    fn a_directory_whose_files_scattered_emits_nothing_for_that_directory() {
        // Same source dir, two destinations: not a directory move.
        let renames = files(&[
            ("oya/gw/crates/x/src/lib.rs", "app/gw/adapters/x/src/lib.rs"),
            ("oya/gw/crates/x/README.md", "docs/x/README.md"),
        ]);
        let dirs = derive_directory_renames(&renames, &BTreeSet::new());
        assert!(!dirs.contains_key("oya/gw/crates/x"));
    }

    #[test]
    fn a_partial_move_that_leaves_files_behind_emits_nothing() {
        let renames = files(&[("oya/gw/crates/x/src/lib.rs", "app/gw/adapters/x/src/lib.rs")]);
        // A tracked file REMAINS directly under the crate dir, so the CRATE did not move
        // wholesale and must not be relabelled. (`x/src` below it did move wholesale and
        // may legitimately be paired — the veto binds at the level that still holds files.)
        let dirs = derive_directory_renames(&renames, &head(&["oya/gw/crates/x/Cargo.toml"]));
        assert!(!dirs.contains_key("oya/gw/crates/x"));
        assert!(!dirs.contains_key("oya/gw/crates"));
    }

    #[test]
    fn a_move_that_preserves_no_relative_subpath_emits_nothing() {
        let renames = files(&[("oya/gw/crates/x/old.rs", "app/gw/adapters/x/new.rs")]);
        let dirs = derive_directory_renames(&renames, &BTreeSet::new());
        assert!(dirs.is_empty());
    }

    #[test]
    fn derived_directory_renames_relabel_a_directory_keyed_baseline() {
        // The live defect: target-parity keys a crate DIRECTORY, so the file-level map
        // alone leaves it unrelabelled and the pure move REDs the gate.
        let renames = files(&[
            (
                "oya/gw/crates/oya-gw-github/src/lib.rs",
                "app/gw/adapters/github/src/lib.rs",
            ),
            (
                "oya/gw/crates/oya-gw-github/Cargo.toml",
                "app/gw/adapters/github/Cargo.toml",
            ),
        ]);
        let dirs = derive_directory_renames(&renames, &BTreeSet::new());
        let frozen = Baseline::from_value(&json!({
            "gates": {
                "cloud-ci-target-parity": {
                    "member_test_code_without_rust_test_target": {
                        "mode": "baseline-block-on-new",
                        "keys": ["oya/gw/crates/oya-gw-github"]
                    }
                }
            }
        }))
        .unwrap();
        let relabelled = relabel_baseline_for_renames(&frozen, &dirs);
        let keys = &relabelled.gates["cloud-ci-target-parity"]
            ["member_test_code_without_rust_test_target"]
            .keys;
        assert!(keys.contains("app/gw/adapters/github"));
        assert!(!keys.contains("oya/gw/crates/oya-gw-github"));
    }

    #[test]
    fn relabelled_pure_rename_is_neither_regression_nor_fix() {
        // With the rename map applied, a pure move is a NO-OP: the debt follows
        // the file. Not laundering — the violation stays tracked at its new key.
        let renames = [("a.rs".to_owned(), "moved/a.rs".to_owned())]
            .into_iter()
            .collect();
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &renames);
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["moved/a.rs", "b.rs"],
        )]);
        let reports = compare(&relabelled, &cur, &SignOff::default());
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert!(unjust.regressions.is_empty(), "a pure move is not new debt");
        assert!(
            unjust.fixed.is_empty(),
            "a pure move is not burn-down either"
        );
        assert_eq!(unjust.tolerated.len(), 2);
        assert!(!unjust.fails());
    }

    #[test]
    fn relabel_still_catches_new_debt_introduced_alongside_a_move() {
        // ANTI-LAUNDERING: relabelling must not become a cloak. A move that also
        // ADDS a violating file is still a regression for the added file.
        let renames = [("a.rs".to_owned(), "moved/a.rs".to_owned())]
            .into_iter()
            .collect();
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &renames);
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["moved/a.rs", "b.rs", "brand-new.rs"],
        )]);
        let reports = compare(&relabelled, &cur, &SignOff::default());
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert_eq!(
            unjust.regressions,
            ["brand-new.rs".to_owned()].into_iter().collect(),
            "debt added during a move must still fail"
        );
        assert!(unjust.fails());
    }

    #[test]
    fn relabel_ignores_renames_whose_source_is_not_baselined() {
        // EXISTENCE GUARD: a rename of a file that carried no violation must not
        // invent a baseline row at the destination (which would tolerate real
        // new debt landing at that path).
        let renames = [("never-violated.rs".to_owned(), "moved/x.rs".to_owned())]
            .into_iter()
            .collect();
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &renames);
        let unjust = &relabelled.gates["cloud-ci-total-accounting"]["unjustified"];
        assert_eq!(
            unjust.keys,
            ["a.rs".to_owned(), "b.rs".to_owned()].into_iter().collect(),
            "relabel must not fabricate keys"
        );
    }

    #[test]
    fn relabel_refuses_to_collide_two_baselined_keys() {
        // NON-COLLISION GUARD: renaming a baselined file ONTO another baselined
        // key would silently merge two debt rows into one, shrinking the
        // baseline without any burn-down. Refuse; leave both keys intact.
        let renames = [("a.rs".to_owned(), "b.rs".to_owned())]
            .into_iter()
            .collect();
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &renames);
        let unjust = &relabelled.gates["cloud-ci-total-accounting"]["unjustified"];
        assert_eq!(
            unjust.keys,
            ["a.rs".to_owned(), "b.rs".to_owned()].into_iter().collect(),
            "a colliding relabel must be refused, not silently merged"
        );
    }

    #[test]
    fn relabel_is_per_code_not_global() {
        // `a.rs` is baselined under BOTH `unjustified` and `unowned`. The relabel
        // must move it in every code that carries it, and must not touch codes
        // that do not.
        let renames = [("a.rs".to_owned(), "moved/a.rs".to_owned())]
            .into_iter()
            .collect();
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &renames);
        let gates = &relabelled.gates["cloud-ci-total-accounting"];
        assert!(gates["unjustified"].keys.contains("moved/a.rs"));
        assert!(gates["unowned"].keys.contains("moved/a.rs"));
        assert!(!gates["unjustified"].keys.contains("a.rs"));
        assert!(!gates["unowned"].keys.contains("a.rs"));
        // frozen_empty code stays empty — nothing to relabel, nothing invented.
        assert!(gates["ci_inventory_registry_drift"].keys.is_empty());
        assert!(gates["ci_inventory_registry_drift"].frozen_empty);
    }

    #[test]
    fn relabel_preserves_mode_and_remediation() {
        // The relabel moves KEYS only. Mode is merge-base DATA a PR must not be
        // able to rewrite by moving a file.
        let renames = [("a.rs".to_owned(), "moved/a.rs".to_owned())]
            .into_iter()
            .collect();
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &renames);
        let gates = &relabelled.gates["cloud-ci-total-accounting"];
        assert_eq!(gates["unjustified"].mode, MODE_BASELINE_BLOCK_ON_NEW);
        assert_eq!(gates["unowned"].mode, "advisory-until-infra");
    }

    #[test]
    fn empty_rename_map_is_the_identity() {
        let relabelled = relabel_baseline_for_renames(&baseline_fixture(), &BTreeMap::new());
        assert_eq!(relabelled, baseline_fixture());
    }

    #[test]
    fn tolerated_baselined_debt_does_not_fail() {
        // current == baseline => all tolerated, no regressions, GREEN.
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["a.rs", "b.rs"],
        )]);
        let reports = compare(&baseline_fixture(), &cur, &SignOff::default());
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert_eq!(unjust.regressions.len(), 0);
        assert_eq!(unjust.tolerated.len(), 2);
        assert!(!unjust.fails());
    }

    #[test]
    fn new_violation_not_in_baseline_fails() {
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["a.rs", "b.rs", "c-NEW.rs"],
        )]);
        let reports = compare(&baseline_fixture(), &cur, &SignOff::default());
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert!(unjust.regressions.contains("c-NEW.rs"));
        assert!(unjust.fails(), "a NEW unjustified file must FAIL");
    }

    #[test]
    fn advisory_code_reports_but_never_fails() {
        // A brand-new unowned key (not in baseline) would be a regression, but unowned is
        // advisory-until-infra so it reports the count and does NOT fail.
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unowned",
            &["a.rs", "z-NEW.rs"],
        )]);
        let reports = compare(&baseline_fixture(), &cur, &SignOff::default());
        let unowned = reports.iter().find(|r| r.code == "unowned").unwrap();
        assert!(unowned.regressions.contains("z-NEW.rs"));
        assert!(
            !unowned.fails(),
            "advisory-until-infra must NOT fail the verdict"
        );
    }

    #[test]
    fn fixed_keys_shrink_and_do_not_fail() {
        // current drops b.rs (fixed). No regression; informational only.
        let cur = current(&[("cloud-ci-total-accounting", "unjustified", &["a.rs"])]);
        let reports = compare(&baseline_fixture(), &cur, &SignOff::default());
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert!(unjust.fixed.contains("b.rs"));
        assert_eq!(unjust.regressions.len(), 0);
        assert!(!unjust.fails());
    }

    #[test]
    fn signed_off_key_in_current_is_not_a_regression() {
        // The one-regen window: the frozen (merge-base) reference predates the admitted
        // key, the live tree already carries it, and the sign-off door lists it. It must
        // land in signed_off, not regressions, and the code must not fail.
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["a.rs", "b.rs", "d-SIGNED.rs"],
        )]);
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["d-SIGNED.rs"]}}
        }));
        let reports = compare(&baseline_fixture(), &cur, &signoff);
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert!(unjust.signed_off.contains("d-SIGNED.rs"));
        assert!(unjust.regressions.is_empty());
        assert!(
            !unjust.fails(),
            "a founder-admitted key must not fail compare-mode"
        );
    }

    #[test]
    fn baseline_growth_without_signoff_is_ratchet_regression() {
        let frozen = baseline_fixture();
        // A regen proposes a baseline that ADDS d-NEW.rs to unjustified.
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs", "d-NEW.rs"]}
            }}
        })).unwrap();
        let growth = ratchet_growth(&frozen, &proposed, &SignOff::default());
        assert!(
            growth
                .iter()
                .any(|(_, c, k)| c == "unjustified" && k == "d-NEW.rs")
        );
    }

    #[test]
    fn signed_off_growth_is_exempt() {
        let frozen = baseline_fixture();
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs", "d-NEW.rs"]}
            }}
        })).unwrap();
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["d-NEW.rs"]}}
        }));
        let growth = ratchet_growth(&frozen, &proposed, &signoff);
        assert!(
            growth.is_empty(),
            "a signed-off addition is exempt from the GROWTH check"
        );
    }

    #[test]
    fn frozen_empty_code_growth_always_fails() {
        let frozen = baseline_fixture();
        // A regen proposes a key for the frozen_empty ci_inventory_registry_drift code.
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "ci_inventory_registry_drift": {"mode": "baseline-block-on-new", "keys": ["<gate>"], "frozen_empty": true}
            }}
        })).unwrap();
        let growth = ratchet_growth(&frozen, &proposed, &SignOff::default());
        assert!(
            growth
                .iter()
                .any(|(_, c, _)| c == "ci_inventory_registry_drift"),
            "frozen_empty codes can never accumulate a baseline"
        );
    }

    #[test]
    fn advisory_code_growth_is_not_ratchet_regression() {
        let frozen = baseline_fixture();
        // Every PR adding a file legitimately grows the advisory dashboards
        // (born-unowned); the growth check must not turn each PR into a signoff.
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unowned": {"mode": "advisory-until-infra", "keys": ["a.rs", "new-file.rs"]}
            }}
        }))
        .unwrap();
        let growth = ratchet_growth(&frozen, &proposed, &SignOff::default());
        assert!(
            growth.is_empty(),
            "advisory-until-infra baselines may grow without signoff: {growth:?}"
        );
    }

    #[test]
    fn remediation_is_carried_from_proposed_face_to_failing_report() {
        // ADR-0555: a FAIL is never a bare flag — the report carries the exact
        // registration edit, enriched from the PROPOSED face (freshest disposition DATA).
        let frozen = baseline_fixture();
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs"],
                                 "remediation": "register it: the exact edit"}
            }}
        }))
        .unwrap();
        let cur = current(&[(
            "cloud-ci-total-accounting",
            "unjustified",
            &["a.rs", "b.rs", "c-NEW.rs"],
        )]);
        let report = evaluate_firewall(&frozen, &proposed, &cur, &SignOff::default());
        let unjust = report
            .codes
            .iter()
            .find(|r| r.code == "unjustified")
            .unwrap();
        assert!(unjust.fails());
        assert_eq!(
            unjust.remediation.as_deref(),
            Some("register it: the exact edit"),
            "the failing report must carry the registration remediation"
        );
    }

    #[test]
    fn frozen_mode_wins_over_proposed_stamp() {
        let frozen = baseline_fixture();
        // An attack regen re-stamps the blocking unjustified code as advisory AND grows
        // it. The frozen (merge-base) mode is the authority: growth must still fire.
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "advisory-until-infra", "keys": ["a.rs", "b.rs", "d-FLIPPED.rs"]}
            }}
        })).unwrap();
        let growth = ratchet_growth(&frozen, &proposed, &SignOff::default());
        assert!(
            growth
                .iter()
                .any(|(_, c, k)| c == "unjustified" && k == "d-FLIPPED.rs"),
            "a same-PR mode flip must not disarm the growth check"
        );
    }

    #[test]
    fn malformed_baseline_mode_is_rejected_fail_closed() {
        let err = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "typo-not-a-mode", "keys": ["a.rs"]}
            }}
        }))
        .unwrap_err();
        assert!(err.contains("unknown mode"), "{err}");
    }

    #[test]
    fn baseline_rejects_non_array_keys() {
        let err = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": "a.rs"}
            }}
        }))
        .unwrap_err();
        assert!(err.contains("keys must be an array"), "{err}");
    }

    #[test]
    fn baseline_rejects_non_bool_frozen_empty() {
        let err = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "ci_inventory_registry_drift": {
                    "mode": "baseline-block-on-new",
                    "frozen_empty": "yes",
                    "keys": []
                }
            }}
        }))
        .unwrap_err();
        assert!(err.contains("frozen_empty must be bool"), "{err}");
    }

    #[test]
    fn baseline_rejects_remaining_malformed_shapes() {
        let cases = [
            (
                "missing gates",
                json!({}),
                "baseline missing object field gates",
            ),
            (
                "non-object gates",
                json!({"gates": []}),
                "baseline missing object field gates",
            ),
            (
                "non-object code map",
                json!({"gates": {"cloud-ci-total-accounting": []}}),
                "codes must be an object",
            ),
            (
                "non-object entry",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": []}}}),
                "must be an object",
            ),
            (
                "missing mode",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": {"keys": []}}}}),
                "missing string mode",
            ),
            (
                "missing keys",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": {"mode": "baseline-block-on-new"}}}}),
                "missing keys",
            ),
            (
                "non-string key",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": {"mode": "baseline-block-on-new", "keys": [7]}}}}),
                "key at index 0 must be string",
            ),
            (
                "empty key",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": {"mode": "baseline-block-on-new", "keys": [""]}}}}),
                "key at index 0 is empty",
            ),
            (
                "duplicate key",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "a.rs"]}}}}),
                "duplicate key",
            ),
            (
                "non-string remediation",
                json!({"gates": {"cloud-ci-total-accounting": {"unjustified": {"mode": "baseline-block-on-new", "keys": [], "remediation": false}}}}),
                "remediation must be string",
            ),
        ];

        for (name, value, expected) in cases {
            let err = Baseline::from_value(&value).unwrap_err();
            assert!(
                err.contains(expected),
                "{name}: expected {expected:?} in error, got {err:?}"
            );
        }
    }

    #[test]
    fn green_corpus_with_baseline_is_green() {
        let frozen = baseline_fixture();
        // current == frozen baseline keys, proposed == frozen => no regression, no growth.
        let cur = baseline_keys_map(&frozen);
        let report = evaluate_firewall(&frozen, &frozen, &cur, &SignOff::default());
        assert!(
            report.is_green(),
            "frozen-at-today corpus must be GREEN with the baseline"
        );
    }

    #[test]
    fn live_signoff_entry_in_flight_is_not_inert() {
        // The one-regen admission window: the key is absent from the frozen (merge-base)
        // face but PRESENT in current + proposed — the entry is doing its job (LIVE).
        let frozen = baseline_fixture();
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs", "d-ADMITTED.rs"]}
            }}
        })).unwrap();
        let cur = baseline_keys_map(&proposed);
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["d-ADMITTED.rs"]}}
        }));
        let report = evaluate_firewall(&frozen, &proposed, &cur, &signoff);
        assert!(
            report.inert_signoff.is_empty(),
            "a LIVE in-flight admission must be tolerated"
        );
        assert!(report.is_green(), "the admitting PR itself must stay GREEN");
    }

    #[test]
    fn inert_signoff_entry_is_flagged_red() {
        // The standing-ticket shape: the entry's key exists NOWHERE (not frozen, not
        // current, not proposed) — the debt it admitted is gone or was never there. Left
        // in place it would silently launder a future re-introduction of that exact key.
        let frozen = baseline_fixture();
        let cur = baseline_keys_map(&frozen);
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["d-GONE.rs"]}}
        }));
        let report = evaluate_firewall(&frozen, &frozen, &cur, &signoff);
        assert_eq!(
            report.inert_signoff,
            vec![(
                "cloud-ci-total-accounting".to_owned(),
                "unjustified".to_owned(),
                "d-GONE.rs".to_owned()
            )],
            "an entry exempting nothing must be flagged inert (remediation: retire it)"
        );
        assert!(!report.is_green(), "an inert door entry must be RED");
    }

    #[test]
    fn signoff_entry_for_frozen_key_that_candidate_still_carries_is_not_inert() {
        // The still-present baselined exemption: the admitted key is in the merge-base face
        // AND the candidate's debt set (current + proposed). The debt the door names still
        // exists, so the entry is LIVE and must NOT fail innocent PRs.
        let frozen = baseline_fixture();
        let cur = baseline_keys_map(&frozen);
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["a.rs"]}}
        }));
        let report = evaluate_firewall(&frozen, &frozen, &cur, &signoff);
        assert!(
            report.inert_signoff.is_empty(),
            "still-present baselined debt is LIVE"
        );
        assert!(report.is_green());
    }

    #[test]
    fn signoff_entry_orphaned_by_candidate_fails_at_pr_time_symmetric() {
        // FRIC-1781460000 — THE SYMMETRY FIX. The candidate RESOLVES the key out of its debt
        // set (e.g. a PR adds an OWNERS file, rendering a previously-unowned key owned), so
        // a.rs leaves current + proposed while the FROZEN merge-base face — which predates
        // the PR's OWNERS addition — still carries it. The old frozen-veto judged the entry
        // LIVE (key ∈ frozen) → PR passed GREEN, then dev went RED post-merge once the
        // merge-base advanced. With the veto dropped, inert-ness reads against the candidate
        // tree: the orphaned entry is INERT and FAILS CLOSED at PR time, symmetric with the
        // push tier.
        let frozen = baseline_fixture();
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["a.rs"]}}
        }));
        // Candidate resolved a.rs out of the debt set; frozen (merge-base) still carries it.
        let fixed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["b.rs"]}
            }}
        }))
        .unwrap();
        let cur_fixed = baseline_keys_map(&fixed);
        let report = evaluate_firewall(&frozen, &fixed, &cur_fixed, &signoff);
        assert_eq!(
            report.inert_signoff,
            vec![(
                "cloud-ci-total-accounting".to_owned(),
                "unjustified".to_owned(),
                "a.rs".to_owned()
            )],
            "an entry the candidate orphaned must be flagged inert at PR time (key ∈ frozen \
             must NOT veto the verdict — that is the FRIC-1781460000 asymmetry)"
        );
        assert!(
            !report.is_green(),
            "the orphaning PR must FAIL CLOSED, not pass GREEN"
        );

        // SYMMETRY: the push tier (frozen advanced to the integrated tip := fixed) reaches
        // the IDENTICAL verdict — the same lingering entry, the same inert RED.
        let push = evaluate_firewall(&fixed, &fixed, &cur_fixed, &signoff);
        assert_eq!(
            push.inert_signoff, report.inert_signoff,
            "PR-tier and push-tier inert detection must be symmetric"
        );
        assert!(!push.is_green());
    }

    const FROZEN_MERGE_BASE: &str = "d5d8be5d4121e91655d7ba361f63271c98c57a68";
    const FROZEN_BASE_TREE: &str = "9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f9f";

    fn frozen_value() -> Value {
        json!({
            "schema": FROZEN_BASELINE_SCHEMA,
            "base_ref": "origin/dev",
            "merge_base": FROZEN_MERGE_BASE,
            "frozen_policy_source": FROZEN_POLICY_SOURCE_MERGE_BASE,
            "missing_at_merge_base": false,
            "provenance": {
                "base_tree_sha": FROZEN_BASE_TREE,
                "merge_base": FROZEN_MERGE_BASE,
                "analyzer": {"emitter": "//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot",
                             "producer": "//ci/facade/artifact-inventory-registry:x"},
                "computed_by": "ADR-0616 regenerate-from-merge-base-source"
            },
            "baseline": {
                "gates": {
                    "cloud-ci-total-accounting": {
                        "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs"]}
                    }
                }
            }
        })
    }

    #[test]
    fn frozen_baseline_parses_happy_path() {
        let frozen = FrozenBaseline::from_value(&frozen_value()).unwrap();
        assert_eq!(frozen.base_ref, "origin/dev");
        assert_eq!(
            frozen.merge_base,
            "d5d8be5d4121e91655d7ba361f63271c98c57a68"
        );
        assert_eq!(frozen.frozen_policy_source, FROZEN_POLICY_SOURCE_MERGE_BASE);
        assert!(!frozen.missing_at_merge_base);
        assert!(
            frozen
                .baseline
                .gates
                .contains_key("cloud-ci-total-accounting")
        );
    }

    #[test]
    fn frozen_baseline_rejects_foreign_schema() {
        let mut value = frozen_value();
        value["schema"] = json!("oya-ci/scm-facts/v1");
        assert!(FrozenBaseline::from_value(&value).is_err());
    }

    #[test]
    fn frozen_baseline_rejects_stale_v1_schema() {
        // A v1 snapshot predates frozen-policy-wins: its frozen reference may have been
        // selected by a candidate-tree policy edit, so it must be rejected fail-closed.
        let mut value = frozen_value();
        value["schema"] = json!("oya-ci/merge-base-baseline/v1");
        let err = FrozenBaseline::from_value(&value).unwrap_err();
        assert!(err.contains("schema mismatch"), "{err}");
    }

    #[test]
    fn frozen_baseline_rejects_missing_or_unknown_policy_source() {
        // The snapshot must DECLARE where the frozen-side policy facts came from — an
        // undeclared source could hide a candidate-policy-selected reference.
        let mut value = frozen_value();
        value
            .as_object_mut()
            .unwrap()
            .remove("frozen_policy_source");
        assert!(FrozenBaseline::from_value(&value).is_err());
        let mut value = frozen_value();
        value["frozen_policy_source"] = json!("working-tree");
        assert!(FrozenBaseline::from_value(&value).is_err());
    }

    #[test]
    fn frozen_baseline_accepts_declared_candidate_bootstrap_source() {
        let mut value = frozen_value();
        value["frozen_policy_source"] = json!(FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP);
        let frozen = FrozenBaseline::from_value(&value).unwrap();
        assert_eq!(
            frozen.frozen_policy_source,
            FROZEN_POLICY_SOURCE_CANDIDATE_BOOTSTRAP
        );
    }

    #[test]
    fn frozen_baseline_rejects_malformed_merge_base() {
        let mut value = frozen_value();
        value["merge_base"] = json!("not-a-sha");
        assert!(FrozenBaseline::from_value(&value).is_err());
    }

    #[test]
    fn frozen_baseline_rejects_undeclared_empty_reference() {
        // An empty reference that does not declare missing_at_merge_base is a tampered or
        // broken snapshot — fail closed, never silently compare against nothing.
        let mut value = frozen_value();
        value["baseline"] = json!({"gates": {}});
        assert!(FrozenBaseline::from_value(&value).is_err());
    }

    #[test]
    fn frozen_baseline_rejects_malformed_embedded_baseline() {
        let mut value = frozen_value();
        value["baseline"]["gates"]["cloud-ci-total-accounting"]["unjustified"]["keys"] =
            json!("not-an-array");
        let err = FrozenBaseline::from_value(&value).unwrap_err();
        assert!(err.contains("embedded baseline malformed"), "{err}");
        assert!(err.contains("keys must be an array"), "{err}");
    }

    #[test]
    fn frozen_baseline_missing_at_merge_base_is_empty_reference() {
        let mut value = frozen_value();
        value["missing_at_merge_base"] = json!(true);
        value["baseline"] = json!({"gates": {}});
        let frozen = FrozenBaseline::from_value(&value).unwrap();
        assert!(frozen.missing_at_merge_base);
        assert!(frozen.baseline.gates.is_empty());
    }

    #[test]
    fn frozen_baseline_rejects_missing_provenance() {
        // ADR-0616: the frozen reference is regenerated from the merge-base source (no committed
        // blob), so a snapshot WITHOUT regeneration provenance is fail-closed rejected.
        let mut value = frozen_value();
        value.as_object_mut().unwrap().remove("provenance");
        let err = FrozenBaseline::from_value(&value).unwrap_err();
        assert!(err.contains("provenance"), "{err}");
    }

    #[test]
    fn frozen_baseline_rejects_malformed_base_tree_sha() {
        let mut value = frozen_value();
        value["provenance"]["base_tree_sha"] = json!("not-a-tree");
        let err = FrozenBaseline::from_value(&value).unwrap_err();
        assert!(err.contains("base_tree_sha"), "{err}");
    }

    #[test]
    fn frozen_baseline_rejects_provenance_bound_to_a_different_merge_base() {
        // A provenance lifted from a DIFFERENT merge-base (tamper / stale snapshot) is rejected:
        // the firewall cannot recompute the tree, so it binds provenance to THIS merge_base.
        let mut value = frozen_value();
        value["provenance"]["merge_base"] = json!("cccccccccccccccccccccccccccccccccccccccc");
        let err = FrozenBaseline::from_value(&value).unwrap_err();
        assert!(err.contains("disagrees"), "{err}");
    }
}
