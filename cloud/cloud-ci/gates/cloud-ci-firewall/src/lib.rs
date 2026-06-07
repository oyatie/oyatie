//! # cloud-ci-firewall (GO-LIVE readiness ratchet)
//!
//! The single required status check for the Phase-0 firewall (PHASE-0-FIREWALL-PLAN
//! go-live readiness; register #20). The four born-blocking gates each prove they DETECT
//! the live exhibit (they go RED on today's corpus). This crate layers the committed
//! `gate-baseline.generated.json` as the SECOND predicate so the firewall blocks only NEW
//! debt, not the frozen pre-existing corpus debt.
//!
//! Two PURE, DATA-over-DATA predicates (no per-code special cases — the per-code behaviour
//! differences live entirely in the baseline DATA: the `mode` + `frozen_empty` fields).
//!
//! COMPARE-MODE — for each `(gate, code)` it computes `regressions = current_keys \
//! baseline_keys` (NEW debt), `tolerated = current_keys ∩ baseline_keys` (accepted
//! pre-existing debt — no fail), and `fixed = baseline_keys \ current_keys` (repaired —
//! informational, drives shrink). `FAIL_for_code` is `!regressions.is_empty()` for
//! `baseline-block-on-new`, and always `false` for `advisory-until-infra`. The gate FAILs
//! iff any code FAILs. Advisory codes still EMIT their counts (the burn-down dashboard) but
//! never flip the verdict until the disposition is flipped to `baseline-block-on-new` (a
//! DATA edit, not a code change).
//!
//! RATCHET-INVARIANT — the baseline may only ever SHRINK on regen. For each `(gate, code)`,
//! `growth = proposed_keys \ committed_keys` (keys a regen would ADD to the baseline). Empty
//! growth is an allowed regen (auto-shrinks to `committed ∩ proposed`). Non-empty growth is
//! a `ratchet_regression` FAILURE unless every grown key is in the founder-signed
//! `_sign_off_additions` allowlist (`gate-baseline.signoff.json`, the ONE-WAY DOOR — a
//! human-edited, NOT producer-generated file). `frozen_empty` codes have a permanently-empty
//! committed baseline, so ANY proposed key is growth — they can never accumulate a baseline.
//! Same predicate, no special case.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The verdict-name reused for a backwards ratchet (both the GATE-4 row-level downgrade and
/// a baseline-growth at regen mean "the ratchet went backwards").
pub const RATCHET_REGRESSION: &str = "ratchet_regression";

/// A baseline: `gate -> code -> (mode, frozen_empty, keys)`. Parsed from
/// `gate-baseline.generated.json` (committed) or from a freshly-regenerated face (proposed).
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
}

impl Baseline {
    /// Parse a `gate-baseline.generated.json` `Value` into the typed baseline.
    pub fn from_value(value: &Value) -> Self {
        let mut gates: BTreeMap<String, BTreeMap<String, CodeBaseline>> = BTreeMap::new();
        if let Some(gate_obj) = value.get("gates").and_then(Value::as_object) {
            for (gate, codes) in gate_obj {
                let mut code_map: BTreeMap<String, CodeBaseline> = BTreeMap::new();
                if let Some(codes_obj) = codes.as_object() {
                    for (code, entry) in codes_obj {
                        code_map.insert(
                            code.clone(),
                            CodeBaseline {
                                mode: entry
                                    .get("mode")
                                    .and_then(Value::as_str)
                                    .unwrap_or("baseline-block-on-new")
                                    .to_owned(),
                                frozen_empty: entry
                                    .get("frozen_empty")
                                    .and_then(Value::as_bool)
                                    == Some(true),
                                keys: str_set(entry.get("keys")),
                            },
                        );
                    }
                }
                gates.insert(gate.clone(), code_map);
            }
        }
        Self { gates }
    }
}

/// The sign-off allowlist (`gate-baseline.signoff.json`): the ONE-WAY DOOR. A key listed
/// under `_sign_off_additions[gate][code]` is exempted from the GROWTH check for one regen.
/// This file is human-edited + founder-signed, NOT producer-generated.
#[derive(Debug, Clone, Default)]
pub struct SignOff {
    additions: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

impl SignOff {
    pub fn from_value(value: &Value) -> Self {
        let mut additions: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
        if let Some(gate_obj) = value
            .get("_sign_off_additions")
            .and_then(Value::as_object)
        {
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
}

impl CodeReport {
    /// A code FAILs iff its mode is baseline-block-on-new AND it has NEW (regression) keys.
    /// advisory-until-infra reports its counts but never fails (until the disposition flips).
    pub fn fails(&self) -> bool {
        self.mode == "baseline-block-on-new" && !self.regressions.is_empty()
    }
}

/// The full firewall report: the compare-mode per-code reports + the ratchet-invariant
/// growth findings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FirewallReport {
    pub codes: Vec<CodeReport>,
    /// `(gate, code, key)` triples a regen would ADD to a baseline that are NOT signed off:
    /// each is a `ratchet_regression` (debt cannot be laundered into the baseline by regen).
    pub ratchet_growth: Vec<(String, String, String)>,
}

impl FirewallReport {
    /// GREEN iff no code FAILs (compare-mode) AND no un-signed-off baseline growth (ratchet).
    pub fn is_green(&self) -> bool {
        !self.codes.iter().any(CodeReport::fails) && self.ratchet_growth.is_empty()
    }
}

/// COMPARE-MODE predicate: compare the current keyed violations against the committed
/// baseline, per `(gate, code)`. `current` is `gate -> code -> keys` (from running each
/// gate's `evaluate_keyed` over the live faces).
pub fn compare(
    committed: &Baseline,
    current: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
) -> Vec<CodeReport> {
    let mut reports: Vec<CodeReport> = Vec::new();
    // Union of gate/code keys present in either the baseline or the current set, so a code
    // that exists only in one side is still reported.
    for gate in union_keys(committed.gates.keys(), current.keys()) {
        let base_codes = committed.gates.get(&gate);
        let cur_codes = current.get(&gate);
        let codes = union_keys(
            base_codes.into_iter().flat_map(BTreeMap::keys),
            cur_codes.into_iter().flat_map(BTreeMap::keys),
        );
        for code in codes {
            let base = base_codes.and_then(|c| c.get(&code));
            let baseline_keys: BTreeSet<String> =
                base.map(|b| b.keys.clone()).unwrap_or_default();
            let mode = base
                .map(|b| b.mode.clone())
                .unwrap_or_else(|| "baseline-block-on-new".to_owned());
            let current_keys: BTreeSet<String> = cur_codes
                .and_then(|c| c.get(&code))
                .cloned()
                .unwrap_or_default();

            let regressions: BTreeSet<String> =
                current_keys.difference(&baseline_keys).cloned().collect();
            let fixed: BTreeSet<String> =
                baseline_keys.difference(&current_keys).cloned().collect();
            let tolerated: BTreeSet<String> = current_keys
                .intersection(&baseline_keys)
                .cloned()
                .collect();

            reports.push(CodeReport {
                gate: gate.clone(),
                code: code.clone(),
                mode,
                current: current_keys.len(),
                baseline: baseline_keys.len(),
                regressions,
                fixed,
                tolerated,
            });
        }
    }
    reports
}

/// RATCHET-INVARIANT predicate: the baseline may only SHRINK on regen. `proposed` is what
/// today's corpus WOULD freeze (the regenerated baseline keys); `committed` is the prior
/// frozen set. Any key the regen would ADD (growth) that is not signed off is a
/// `ratchet_regression`. `frozen_empty` codes have an empty committed set, so any proposed
/// key is growth — the same predicate enforces "never accumulate a baseline" for them.
pub fn ratchet_growth(
    committed: &Baseline,
    proposed: &Baseline,
    signoff: &SignOff,
) -> Vec<(String, String, String)> {
    let mut growth: Vec<(String, String, String)> = Vec::new();
    for (gate, proposed_codes) in &proposed.gates {
        for (code, proposed_code) in proposed_codes {
            let committed_keys = committed
                .gates
                .get(gate)
                .and_then(|c| c.get(code))
                .map(|c| &c.keys);
            for key in &proposed_code.keys {
                let in_committed =
                    committed_keys.is_some_and(|keys| keys.contains(key));
                if !in_committed && !signoff.is_signed_off(gate, code, key) {
                    growth.push((gate.clone(), code.clone(), key.clone()));
                }
            }
        }
    }
    growth
}

/// Run BOTH predicates and assemble the full firewall report.
pub fn evaluate_firewall(
    committed: &Baseline,
    proposed: &Baseline,
    current: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    signoff: &SignOff,
) -> FirewallReport {
    FirewallReport {
        codes: compare(committed, current),
        ratchet_growth: ratchet_growth(committed, proposed, signoff),
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
                    "registry_drift": {"mode": "baseline-block-on-new", "keys": [], "frozen_empty": true}
                }
            }
        }))
    }

    fn current(pairs: &[(&str, &str, &[&str])]) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
        let mut out: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
        for (gate, code, keys) in pairs {
            out.entry((*gate).to_owned())
                .or_default()
                .insert(
                    (*code).to_owned(),
                    keys.iter().map(|k| (*k).to_owned()).collect(),
                );
        }
        out
    }

    #[test]
    fn tolerated_baselined_debt_does_not_fail() {
        // current == baseline => all tolerated, no regressions, GREEN.
        let cur = current(&[("cloud-ci-total-accounting", "unjustified", &["a.rs", "b.rs"])]);
        let reports = compare(&baseline_fixture(), &cur);
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
        let reports = compare(&baseline_fixture(), &cur);
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
        let reports = compare(&baseline_fixture(), &cur);
        let unowned = reports.iter().find(|r| r.code == "unowned").unwrap();
        assert!(unowned.regressions.contains("z-NEW.rs"));
        assert!(!unowned.fails(), "advisory-until-infra must NOT fail the verdict");
    }

    #[test]
    fn fixed_keys_shrink_and_do_not_fail() {
        // current drops b.rs (fixed). No regression; informational only.
        let cur = current(&[("cloud-ci-total-accounting", "unjustified", &["a.rs"])]);
        let reports = compare(&baseline_fixture(), &cur);
        let unjust = reports.iter().find(|r| r.code == "unjustified").unwrap();
        assert!(unjust.fixed.contains("b.rs"));
        assert_eq!(unjust.regressions.len(), 0);
        assert!(!unjust.fails());
    }

    #[test]
    fn baseline_growth_without_signoff_is_ratchet_regression() {
        let committed = baseline_fixture();
        // A regen proposes a baseline that ADDS d-NEW.rs to unjustified.
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs", "d-NEW.rs"]}
            }}
        }));
        let growth = ratchet_growth(&committed, &proposed, &SignOff::default());
        assert!(growth.iter().any(|(_, c, k)| c == "unjustified" && k == "d-NEW.rs"));
    }

    #[test]
    fn signed_off_growth_is_exempt() {
        let committed = baseline_fixture();
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "unjustified": {"mode": "baseline-block-on-new", "keys": ["a.rs", "b.rs", "d-NEW.rs"]}
            }}
        }));
        let signoff = SignOff::from_value(&json!({
            "_sign_off_additions": {"cloud-ci-total-accounting": {"unjustified": ["d-NEW.rs"]}}
        }));
        let growth = ratchet_growth(&committed, &proposed, &signoff);
        assert!(growth.is_empty(), "a signed-off addition is exempt from the GROWTH check");
    }

    #[test]
    fn frozen_empty_code_growth_always_fails() {
        let committed = baseline_fixture();
        // A regen proposes a key for the frozen_empty registry_drift code.
        let proposed = Baseline::from_value(&json!({
            "gates": {"cloud-ci-total-accounting": {
                "registry_drift": {"mode": "baseline-block-on-new", "keys": ["<gate>"], "frozen_empty": true}
            }}
        }));
        let growth = ratchet_growth(&committed, &proposed, &SignOff::default());
        assert!(
            growth.iter().any(|(_, c, _)| c == "registry_drift"),
            "frozen_empty codes can never accumulate a baseline"
        );
    }

    #[test]
    fn green_corpus_with_baseline_is_green() {
        let committed = baseline_fixture();
        // current == committed baseline keys, proposed == committed => no regression, no growth.
        let cur = baseline_keys_map(&committed);
        let report =
            evaluate_firewall(&committed, &committed, &cur, &SignOff::default());
        assert!(report.is_green(), "frozen-at-today corpus must be GREEN with the baseline");
    }
}
