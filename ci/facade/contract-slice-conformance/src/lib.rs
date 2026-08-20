//! # cloud-ci-contract-slice-conformance
//!
//! Paved-road Rust/Buck2 gate that replaces the fleet-wide
//! `scripts/tests/*_check.py` "contract slice" validators with a single owned,
//! declarative, owned-Rust gate.
//!
//! A worker declares a slice as one committed fragment file under `slices/`
//! (the committed spec path, its required fields, enum constraints, forbidden
//! content markers, and — for a migration — the retired Python source) and
//! ships the slice's committed spec JSON. No new Python, no shell, no CLI, no
//! new crate: the gate reads the declared slices and validates the live
//! committed specs.
//!
//! The surface is API/config shaped: callers pass the policy plus the typed
//! JSON corpus to [`evaluate_configured`]. The gate is pure — it never shells
//! out, spawns an interpreter, mutates files, or reads ambient repository
//! state. Repository-specific paths and per-slice rules live as DATA, sharded
//! one slice per file under `slices/`; `contract-slice-policy.json` is the
//! GENERATED aggregate `evaluate_configured` consumes (see `fragments.rs` and
//! the README) — its committed path is unchanged (external `governed_surfaces`
//! / root-hub-pointers / ADR references cite it by exact path), but it is no
//! longer hand-edited.
//!
//! Mirrors the `resource-contract-conformance` gate (ADR-0515 WS-D pure gate
//! shape; the `source_migration_slice` Python→Rust retirement pattern).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use regex::Regex;
use serde_json::Value;

mod fragments;
pub use fragments::{FragmentLoad, aggregate_policy, load_slice_fragments, render_policy_json};

/// Cloud-ci gate id for the contract-slice conformance gate.
pub const GATE_ID: &str = "cloud-ci-contract-slice-conformance";

/// The one legitimate `primary_execution_path` for a contract-slice policy.
pub const REQUIRED_PRIMARY_EXECUTION_PATH: &str = "rust_buck2_cloud_ci_gate";

/// Content markers that must never appear inside a contract-slice *spec*:
/// retired CLI authority and non-Rust interpreter invocations. This enforces
/// the no-shell / no-interpreter / no-retired-CLI doctrine inside the contract
/// content itself (e.g. a `python3 …` verification command baked into a spec).
///
/// Matched case-insensitively. Retired Python sources are declared in the
/// policy's `source_migration_slice`, which is validated separately and never
/// scanned here, so naming a retired `*.py` in the ledger is not a violation.
const FORBIDDEN_SPEC_MARKERS: &[&str] = &[
    "oya gate",
    "oya-dev-cli",
    "python3",
    "cargo run",
    "terraform apply",
    "opentofu apply",
    "kubectl apply",
    "aws cli",
    "gcloud cli",
];

/// Bare-code verdict of a gate run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed gate violation: a stable `code` plus the offending unit `key`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String, // data_class: INTERNAL_ONLY
    pub key: String,  // data_class: INTERNAL_ONLY
}

impl Finding {
    fn new(code: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            key: key.into(),
        }
    }
}

/// The outcome of a gate run: verdict, the keyed findings, and the bare set of
/// violation codes (for terse assertions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,             // data_class: INTERNAL_ONLY
    pub findings: BTreeSet<Finding>,  // data_class: INTERNAL_ONLY
    pub violations: BTreeSet<String>, // data_class: INTERNAL_ONLY
}

impl Report {
    fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let violations = findings.iter().map(|f| f.code.clone()).collect();
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            findings,
            violations,
        }
    }
}

/// Evaluate a contract-slice policy against a corpus of committed slice specs.
///
/// `corpus` maps each slice's declared `spec_path` to its parsed JSON. The gate
/// is pure and total: any structural gap becomes a keyed [`Finding`] rather than
/// a panic, so a malformed policy is RED, never a crash.
#[must_use]
pub fn evaluate_configured(policy: &Value, corpus: &BTreeMap<String, Value>) -> Report {
    let mut findings = BTreeSet::new();

    if policy.get("primary_execution_path").and_then(Value::as_str)
        != Some(REQUIRED_PRIMARY_EXECUTION_PATH)
    {
        findings.insert(Finding::new(
            "contract_slice_primary_path_not_rust",
            "primary_execution_path",
        ));
    }

    let slices = policy.get("slices").and_then(Value::as_array);
    match slices {
        Some(slices) if !slices.is_empty() => {
            for slice in slices {
                evaluate_slice(slice, corpus, &mut findings);
            }
        }
        _ => {
            findings.insert(Finding::new(
                "contract_slice_policy_has_no_slices",
                "slices",
            ));
        }
    }

    Report::from_findings(findings)
}

/// Fail-closed on nested policy shape: emit an unknown-policy-key finding for any
/// key of a requirement `object` not in `allowed`. A typo in a nested key that
/// carries membership (`members`, `member_required_fields`) would otherwise
/// silently disarm that requirement (a `field`-typo fails closed by construction,
/// but a `members`-typo yields an empty loop and zero findings).
fn check_keys(
    object: &Value,
    allowed: &[&str],
    slice_id: &str,
    context: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if let Some(map) = object.as_object() {
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                findings.insert(Finding::new(
                    "contract_slice_unknown_policy_key",
                    format!("{slice_id}:{context}:{key}"),
                ));
            }
        }
    }
}

fn evaluate_slice(
    slice: &Value,
    corpus: &BTreeMap<String, Value>,
    findings: &mut BTreeSet<Finding>,
) {
    let slice_id = slice
        .get("slice_id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown-slice>");

    let Some(spec_path) = slice.get("spec_path").and_then(Value::as_str) else {
        findings.insert(Finding::new(
            "contract_slice_missing_spec_path",
            slice_id.to_owned(),
        ));
        return;
    };

    let Some(spec) = corpus.get(spec_path) else {
        findings.insert(Finding::new(
            "contract_slice_spec_absent",
            format!("{slice_id}:{spec_path}"),
        ));
        return;
    };

    // 0. Fail-closed policy shape: an unrecognized slice key is almost always a
    //    typo that SILENTLY disarms a rule (`"required_field"` instead of
    //    `"required_fields"`) — the exact false-green this gate exists to kill.
    //    Surface it rather than ignore it. (Policy is self-serve data authored by
    //    workers without Rust review, so the gate must fail closed on its own shape.)
    const RECOGNIZED_SLICE_KEYS: &[&str] = &[
        "slice_id",
        "spec_path",
        "required_fields",
        "required_true_fields",
        "required_false_fields",
        "enum_constraints",
        "required_array_members",
        "exact_array_fields",
        "required_object_array_members",
        "forbidden_markers",
        "forbidden_field_markers",
        "marker_exclude_fields",
        "required_markers",
        "field_patterns",
        "exact_projected_sequence",
        "array_cardinality",
        "projected_value_sets",
        "source_migration_slice",
        "skip_universal_markers",
    ];
    if let Some(object) = slice.as_object() {
        for key in object.keys() {
            if !RECOGNIZED_SLICE_KEYS.contains(&key.as_str()) {
                findings.insert(Finding::new(
                    "contract_slice_unknown_policy_key",
                    format!("{slice_id}:{key}"),
                ));
            }
        }
    }

    // 1. Forbidden content markers (universal doctrine set + per-slice extras).
    //    Opt-out (`skip_universal_markers`) is narrow and slice-declared: it
    //    exists only for a slice whose `spec_path` is a SHARED cross-cutting
    //    registry this slice doesn't own the doctrine content of (e.g.
    //    `root-hub-pointers.json`, which legitimately narrates the *retirement*
    //    of `oya-dev-cli`/`oya gate` in its own provenance prose — the exact
    //    strings this scan exists to catch when a slice's OWN spec bakes them in
    //    as live usage). It does not skip the slice's own `forbidden_markers`.
    //    Matching canonicalizes both marker and leaf to an `[a-z0-9]`-only form
    //    (see `canonical_alnum`), so `production-ready` trips `production ready`.
    //
    //    `marker_exclude_fields` carves named sub-trees out of the universal +
    //    per-slice whole-spec scan for a spec that legitimately quotes a forbidden
    //    phrase in a bounded place (e.g. a `claim_boundary` list of the phrases it
    //    forbids); the excluded sub-trees are still subject to field-scoped
    //    `forbidden_field_markers` and every other check.
    //
    //    Bidi-REORDER controls (LRE/RLE/PDF/LRO/RLO, LRI/RLI/FSI/PDI) make the
    //    visual order differ from the logical order, so `production <RLO>ydaer`
    //    renders "production ready" yet canonicalizes reversed and would evade the
    //    scan. They have no legitimate use in claim content, so their PRESENCE is
    //    itself a violation (fail-closed) — stripping them would leave the reversed
    //    text and still evade. Plain directional marks (U+200E/200F) and general
    //    non-ASCII (i18n / the Korea localization pack) are NOT rejected.
    if contains_bidi_reorder_control(spec) {
        findings.insert(Finding::new(
            "contract_slice_bidi_control_in_content",
            slice_id.to_owned(),
        ));
    }
    let excluded: Vec<&Value> = string_array(slice, "marker_exclude_fields")
        .iter()
        .filter_map(|path| get_dotted(spec, path))
        .collect();
    let skip_universal_markers =
        slice.get("skip_universal_markers").and_then(Value::as_bool) == Some(true);
    if !skip_universal_markers {
        for marker in FORBIDDEN_SPEC_MARKERS {
            if recursively_contains_normalized(spec, &canonical_alnum(marker), &excluded) {
                findings.insert(Finding::new(
                    "contract_slice_forbidden_marker",
                    format!("{slice_id}:{marker}"),
                ));
            }
        }
    }
    if has_non_string_element(slice.get("forbidden_markers")) {
        findings.insert(Finding::new(
            "contract_slice_malformed_policy_value",
            format!("{slice_id}:forbidden_markers"),
        ));
    }
    for marker in string_array(slice, "forbidden_markers") {
        if recursively_contains_normalized(spec, &canonical_alnum(&marker), &excluded) {
            findings.insert(Finding::new(
                "contract_slice_forbidden_marker",
                format!("{slice_id}:{marker}"),
            ));
        }
    }

    // 1a. Field-scoped forbidden markers: a phrase that is forbidden only inside a
    //     dotted sub-tree (e.g. a `can_claim_now` block must not contain a
    //     cannot-claim-yet phrase, though that phrase may legitimately appear
    //     elsewhere). Separator-normalized like the whole-spec scan.
    if let Some(requirements) = rules_array(slice, "forbidden_field_markers", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "markers"],
                slice_id,
                "forbidden_field_markers",
                findings,
            );
            let field = requirement.get("field").and_then(Value::as_str);
            let markers: Vec<&str> = requirement
                .get("markers")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            // Fail-closed on shape: an empty markers list, a non-string markers
            // element, or a missing field enforces nothing and must RED.
            if markers.is_empty()
                || has_non_string_element(requirement.get("markers"))
                || field.is_none_or(str::is_empty)
            {
                findings.insert(Finding::new(
                    "contract_slice_forbidden_field_markers_malformed",
                    format!("{slice_id}:{}", field.unwrap_or("<no-field>")),
                ));
                continue;
            }
            let field = field.unwrap_or_default();
            if let Some(scope) = get_dotted(spec, field) {
                for marker in markers {
                    if recursively_contains_normalized(scope, &canonical_alnum(marker), &[]) {
                        findings.insert(Finding::new(
                            "contract_slice_forbidden_field_marker",
                            format!("{slice_id}:{field}:{marker}"),
                        ));
                    }
                }
            }
        }
    }

    // 1b. Required content markers: the inverse of forbidden_markers, scoped to a
    //     dotted field. Proves a conditional sub-tree (e.g. a JSON Schema `allOf`
    //     tier-conditional block) actually names the values it claims to gate on.
    //     Scoping to `field` (rather than the whole spec) matters: an unscoped
    //     search would trivially pass on a marker that merely appears elsewhere
    //     in the document (e.g. a tier name already required by an enum
    //     constraint), silently disarming the check it exists to make.
    //
    //     `quantifier` (default `all_of`) selects between "every marker must be
    //     present" and `any_of` ("at least one must be present" — e.g. an explicit
    //     nonclaim satisfied by any one of several accepted wordings). `scope`
    //     (default field-scoped) may be `whole_spec` for a marker that is
    //     legitimately document-wide rather than confined to one sub-tree.
    if let Some(requirements) = rules_array(slice, "required_markers", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "markers", "quantifier", "scope"],
                slice_id,
                "required_markers",
                findings,
            );
            let markers: Vec<&str> = requirement
                .get("markers")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let quantifier = requirement
                .get("quantifier")
                .and_then(Value::as_str)
                .unwrap_or("all_of");
            let whole_spec = requirement.get("scope").and_then(Value::as_str) == Some("whole_spec");
            let target_field = requirement.get("field").and_then(Value::as_str);
            // Fail-closed on shape: an empty markers list is a silent no-op that
            // enforces nothing; a non-string markers element would be dropped; an
            // unknown quantifier/scope, or a missing field when not searching the
            // whole spec, would evaluate vacuously — all must RED.
            let bad_quantifier = !matches!(quantifier, "all_of" | "any_of");
            let bad_scope = requirement
                .get("scope")
                .is_some_and(|s| s.as_str() != Some("whole_spec"));
            let bad_field = !whole_spec && target_field.is_none_or(str::is_empty);
            let bad_markers =
                markers.is_empty() || has_non_string_element(requirement.get("markers"));
            if bad_markers || bad_quantifier || bad_scope || bad_field {
                findings.insert(Finding::new(
                    "contract_slice_required_markers_malformed",
                    format!("{slice_id}:{}", target_field.unwrap_or("<no-field>")),
                ));
                continue;
            }
            let target_field = target_field.unwrap_or_default();
            let scope = if whole_spec {
                Some(spec)
            } else {
                get_dotted(spec, target_field)
            };
            let contains =
                |marker: &str| scope.is_some_and(|value| recursively_contains(value, marker));
            if quantifier == "any_of" {
                // any_of REDs only when NO declared marker is present.
                if !markers.iter().any(|marker| contains(marker)) {
                    findings.insert(Finding::new(
                        "contract_slice_required_marker_none_present",
                        format!("{slice_id}:{target_field}"),
                    ));
                }
            } else {
                for marker in markers {
                    if !contains(marker) {
                        findings.insert(Finding::new(
                            "contract_slice_required_marker_missing",
                            format!("{slice_id}:{target_field}:{marker}"),
                        ));
                    }
                }
            }
        }
    }

    // 2. Required fields (dotted paths) must resolve to a non-null value.
    for field in string_array(slice, "required_fields") {
        if get_dotted(spec, &field).is_none_or(Value::is_null) {
            findings.insert(Finding::new(
                "contract_slice_missing_required_field",
                format!("{slice_id}:{field}"),
            ));
        }
    }

    // 3. Enum constraints: the dotted field's STRING value must be allowed. By
    //    default this is type-preserving string equality (a spec number `90` does
    //    NOT satisfy `allowed: ["90"]`), so a type-sensitive Python `==` converts
    //    faithfully. A constraint may opt into scalar canonicalization with
    //    `match_scalar: true` to pin a numeric/bool leaf authored as a string
    //    literal (e.g. a `14.4` threshold). A non-string element in `allowed` is a
    //    mistyped list and fails closed.
    if let Some(constraints) = rules_array(slice, "enum_constraints", slice_id, findings) {
        for constraint in constraints {
            check_keys(
                constraint,
                &["field", "allowed", "match_scalar"],
                slice_id,
                "enum_constraints",
                findings,
            );
            let field = constraint
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            if has_non_string_element(constraint.get("allowed")) {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:enum_constraints.allowed:{field}"),
                ));
                continue;
            }
            let allowed: Vec<&str> = constraint
                .get("allowed")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if constraint
                .get("match_scalar")
                .is_some_and(|flag| !flag.is_boolean())
            {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:enum_constraints.match_scalar:{field}"),
                ));
                continue;
            }
            let match_scalar =
                constraint.get("match_scalar").and_then(Value::as_bool) == Some(true);
            let actual = if match_scalar {
                get_dotted(spec, field).and_then(scalar_str)
            } else {
                get_dotted(spec, field)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            };
            if !actual.is_some_and(|value| allowed.contains(&value.as_str())) {
                findings.insert(Finding::new(
                    "contract_slice_enum_violation",
                    format!("{slice_id}:{field}"),
                ));
            }
        }
    }

    // 4. Required array members: a dotted array field must contain (be a
    //    superset of) every declared member. Covers "this contract must enumerate
    //    exactly these source ADRs / nonclaims / filters" without hardcoding them
    //    in Rust — they stay data in the policy. Matching is type-preserving
    //    string membership by default (a numeric spec element does NOT satisfy a
    //    string-authored member); `match_scalar: true` opts into scalar
    //    canonicalization for a JSON number array (e.g. pinned rollout-stage
    //    percentages). A non-string element in `members` is a mistyped list and
    //    fails closed.
    if let Some(requirements) = rules_array(slice, "required_array_members", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "members", "match_scalar"],
                slice_id,
                "required_array_members",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            if has_non_string_element(requirement.get("members")) {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:required_array_members.members:{field}"),
                ));
                continue;
            }
            if requirement
                .get("match_scalar")
                .is_some_and(|flag| !flag.is_boolean())
            {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:required_array_members.match_scalar:{field}"),
                ));
                continue;
            }
            let match_scalar =
                requirement.get("match_scalar").and_then(Value::as_bool) == Some(true);
            let present: Vec<String> = get_dotted(spec, field)
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|element| {
                            if match_scalar {
                                scalar_str(element)
                            } else {
                                element.as_str().map(str::to_owned)
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();
            for member in requirement
                .get("members")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                if !present.iter().any(|value| value == member) {
                    findings.insert(Finding::new(
                        "contract_slice_missing_array_member",
                        format!("{slice_id}:{field}:{member}"),
                    ));
                }
            }
        }
    }

    // 4a. Exact array fields: a dotted array field must equal the declared
    //     `values` exactly (same members, same order, no extras). Ports the
    //     retired Python's `== TIERS` list-equality checks (tier enum, strictest
    //     tier order), which `required_array_members`'s superset check cannot
    //     express: a superset check alone would still green a spec that grew an
    //     unlisted extra value or reordered the strictness ladder.
    if let Some(requirements) = rules_array(slice, "exact_array_fields", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "values"],
                slice_id,
                "exact_array_fields",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            if has_non_string_element(requirement.get("values")) {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:exact_array_fields.values:{field}"),
                ));
                continue;
            }
            let expected: Vec<&str> = requirement
                .get("values")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            // Fail-closed on shape: match element-wise with a length check rather
            // than `filter_map(as_str)`, so a non-string element (or an extra one)
            // is a mismatch instead of being silently dropped before the compare.
            let matches = get_dotted(spec, field)
                .and_then(Value::as_array)
                .is_some_and(|actual| {
                    actual.len() == expected.len()
                        && actual
                            .iter()
                            .zip(&expected)
                            .all(|(value, want)| value.as_str() == Some(*want))
                });
            if !matches {
                findings.insert(Finding::new(
                    "contract_slice_array_not_exact",
                    format!("{slice_id}:{field}"),
                ));
            }
        }
    }

    // 4b. Required-true fields: a dotted field must be boolean `true`. Pins a
    //     fail-closed claim-control (`no_runtime_mutation`) that required-fields
    //     presence alone cannot (presence accepts `false`).
    for field in string_array(slice, "required_true_fields") {
        if get_dotted(spec, &field).and_then(Value::as_bool) != Some(true) {
            findings.insert(Finding::new(
                "contract_slice_field_not_true",
                format!("{slice_id}:{field}"),
            ));
        }
    }

    // 4b'. Required-false fields: the mirror of required_true_fields — a dotted
    //      field must be boolean `false`. Pins a negative default (e.g. a
    //      `non_necessary_default == false` claim-control) that required-fields
    //      presence alone cannot (presence accepts `true`). A non-string element in
    //      the list is a mistyped config and fails closed.
    if has_non_string_element(slice.get("required_false_fields")) {
        findings.insert(Finding::new(
            "contract_slice_malformed_policy_value",
            format!("{slice_id}:required_false_fields"),
        ));
    }
    for field in string_array(slice, "required_false_fields") {
        if get_dotted(spec, &field).and_then(Value::as_bool) != Some(false) {
            findings.insert(Finding::new(
                "contract_slice_field_not_false",
                format!("{slice_id}:{field}"),
            ));
        }
    }

    // 4c. Required object-array members: an array-of-objects field must contain an
    //     object per declared member (matched on `member_key`, default "id"), each
    //     object carrying `member_required_fields` and satisfying
    //     `member_enum_constraints`. Expresses "the six-input promotion gate must
    //     enumerate exactly these inputs, each fail-closed" as policy DATA.
    if let Some(requirements) =
        rules_array(slice, "required_object_array_members", slice_id, findings)
    {
        for requirement in requirements {
            check_keys(
                requirement,
                &[
                    "field",
                    "member_key",
                    "members",
                    "exact_members",
                    "member_required_fields",
                    "member_enum_constraints",
                    "conditional_assertions",
                    "field_implies_required",
                ],
                slice_id,
                "required_object_array_members",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let member_key = requirement
                .get("member_key")
                .and_then(Value::as_str)
                .unwrap_or("id");
            if has_non_string_element(requirement.get("members")) {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:required_object_array_members.members:{field}"),
                ));
                continue;
            }
            // Validate member_enum_constraints `allowed` SHAPE independent of the
            // spec rows, so a mistyped list fails closed even with zero committed
            // rows (a row-gated check would silently pass then).
            for constraint in requirement
                .get("member_enum_constraints")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if has_non_string_element(constraint.get("allowed")) {
                    let enum_field = constraint
                        .get("field")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    findings.insert(Finding::new(
                        "contract_slice_malformed_policy_value",
                        format!("{slice_id}:member_enum_constraints.allowed:{field}:{enum_field}"),
                    ));
                }
            }
            let Some(objects) = get_dotted(spec, field).and_then(Value::as_array) else {
                findings.insert(Finding::new(
                    "contract_slice_missing_object_array",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            };
            let present: BTreeSet<&str> = objects
                .iter()
                .filter_map(|object| object.get(member_key).and_then(Value::as_str))
                .collect();
            let declared: BTreeSet<&str> = requirement
                .get("members")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            for member in &declared {
                if !present.contains(member) {
                    findings.insert(Finding::new(
                        "contract_slice_missing_object_array_member",
                        format!("{slice_id}:{field}:{member}"),
                    ));
                }
            }
            // Exact membership (the reverse direction): no member key beyond the
            // declared set. Ports the retired Python's `set(...) == set(...)`
            // membership checks (regime ids, audit wire classes), which the
            // superset check above cannot express: a superset check alone would
            // still green a spec that grew an extra, unreviewed member.
            if requirement.get("exact_members").and_then(Value::as_bool) == Some(true) {
                for member in &present {
                    if !declared.contains(member) {
                        findings.insert(Finding::new(
                            "contract_slice_unexpected_object_array_member",
                            format!("{slice_id}:{field}:{member}"),
                        ));
                    }
                }
            }
            let member_required: Vec<&str> = requirement
                .get("member_required_fields")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let member_enums = requirement
                .get("member_enum_constraints")
                .and_then(Value::as_array);
            let conditional_assertions = requirement
                .get("conditional_assertions")
                .and_then(Value::as_array);
            let field_implies_required = requirement
                .get("field_implies_required")
                .and_then(Value::as_array);
            for object in objects {
                let member_id = object
                    .get(member_key)
                    .and_then(Value::as_str)
                    .unwrap_or("<unkeyed>");
                for required_field in &member_required {
                    // Dotted (get_dotted, not a flat `object.get`) so a member field
                    // nested under an object (e.g. `capability_overrides.enforcement`)
                    // is reachable the same way a top-level `required_fields` entry is.
                    if get_dotted(object, required_field).is_none_or(Value::is_null) {
                        findings.insert(Finding::new(
                            "contract_slice_object_member_missing_field",
                            format!("{slice_id}:{field}:{member_id}:{required_field}"),
                        ));
                    }
                }
                // Conditional presence: when a boolean gate field on this member is
                // `true`, a companion set of fields becomes required (and, if those
                // fields are arrays, must be non-empty — an empty list is presence
                // without content, matching the retired Python `require(row.get(...))`
                // truthiness check). Ports "third_party_attestor_required implies
                // attestor_required_for_tiers + evidence_requirements" generically,
                // so it still holds for any future member with the gate flag set,
                // not just the currently-committed ones.
                for rule in field_implies_required.into_iter().flatten() {
                    check_keys(
                        rule,
                        &["if_field", "then_required_fields"],
                        slice_id,
                        "field_implies_required",
                        findings,
                    );
                    let if_field = rule.get("if_field").and_then(Value::as_str);
                    let then_required: Vec<&str> = rule
                        .get("then_required_fields")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();
                    // Fail-closed on shape: a missing/non-string if_field would
                    // silently skip the whole rule, and an empty then_required_fields
                    // would enforce nothing — both must RED.
                    if if_field.is_none_or(str::is_empty) || then_required.is_empty() {
                        findings.insert(Finding::new(
                            "contract_slice_field_implies_required_malformed",
                            format!("{slice_id}:{field}:{member_id}"),
                        ));
                        continue;
                    }
                    let if_field = if_field.unwrap_or_default();
                    // Trigger the implication when the antecedent is boolean `true`
                    // OR present-but-wrong-typed (a string `"true"` must not silently
                    // disable the rule — fail closed). Absent / null / boolean `false`
                    // does not trigger.
                    let triggered = !matches!(
                        get_dotted(object, if_field),
                        None | Some(Value::Null) | Some(Value::Bool(false))
                    );
                    if !triggered {
                        continue;
                    }
                    for then_field in then_required {
                        let value = get_dotted(object, then_field);
                        let satisfied = match value {
                            Some(Value::Array(items)) => !items.is_empty(),
                            Some(v) => !v.is_null(),
                            None => false,
                        };
                        if !satisfied {
                            findings.insert(Finding::new(
                                "contract_slice_conditional_required_field_absent",
                                format!("{slice_id}:{field}:{member_id}:{if_field}=>{then_field}"),
                            ));
                        }
                    }
                }
                for constraint in member_enums.into_iter().flatten() {
                    let enum_field = constraint
                        .get("field")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let allowed: Vec<&str> = constraint
                        .get("allowed")
                        .and_then(Value::as_array)
                        .map(|a| a.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();
                    let actual = get_dotted(object, enum_field).and_then(Value::as_str);
                    if !actual.is_some_and(|value| allowed.contains(&value)) {
                        findings.insert(Finding::new(
                            "contract_slice_object_member_enum_violation",
                            format!("{slice_id}:{field}:{member_id}:{enum_field}"),
                        ));
                    }
                }
                for assertion in conditional_assertions.into_iter().flatten() {
                    evaluate_conditional_assertion(
                        assertion, object, member_id, slice_id, field, findings,
                    );
                }
            }
        }
    }

    // 4d. Field patterns: a dotted scalar field must match a declared regular
    //     expression. Covers cryptographic/id shapes the enum/marker checks cannot
    //     express (Ed25519 `[0-9a-f]{128}`, SHA-256 `[0-9a-f]{64}`, a region id
    //     grammar, base64url length). A malformed pattern fails closed rather than
    //     silently accepting everything.
    if let Some(requirements) = rules_array(slice, "field_patterns", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "pattern"],
                slice_id,
                "field_patterns",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let pattern = requirement.get("pattern").and_then(Value::as_str);
            // Fail-closed: a missing or EMPTY pattern would match everything
            // (`Regex::new("")` is Ok and matches any input) — a vacuous green.
            let Some(pattern) = pattern.filter(|p| !p.is_empty()) else {
                findings.insert(Finding::new(
                    "contract_slice_bad_pattern",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            };
            let Ok(regex) = Regex::new(pattern) else {
                findings.insert(Finding::new(
                    "contract_slice_bad_pattern",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            };
            let value = get_dotted(spec, field).and_then(scalar_str);
            // Fail-closed: a missing/non-scalar field cannot match, so it is a
            // mismatch rather than a skipped check.
            if !value.as_deref().is_some_and(|text| regex.is_match(text)) {
                findings.insert(Finding::new(
                    "contract_slice_pattern_mismatch",
                    format!("{slice_id}:{field}"),
                ));
            }
        }
    }

    // 4e. Exact projected sequence: project `member_field` across the array-of-
    //     objects at `field` and require the projection to equal `values` exactly
    //     (same members, same order, no extras). Expresses an ordered pipeline
    //     (canonical purposes, a release stage flow) that the superset/exact-set
    //     checks cannot: order and length both matter here. Non-string/missing
    //     projected members and length differences are mismatches (fail-closed).
    if let Some(requirements) = rules_array(slice, "exact_projected_sequence", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "member_field", "values"],
                slice_id,
                "exact_projected_sequence",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let member_field = requirement
                .get("member_field")
                .and_then(Value::as_str)
                .unwrap_or("");
            if has_non_string_element(requirement.get("values")) {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:exact_projected_sequence.values:{field}"),
                ));
                continue;
            }
            let expected: Vec<&str> = requirement
                .get("values")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let matches = get_dotted(spec, field)
                .and_then(Value::as_array)
                .is_some_and(|objects| {
                    objects.len() == expected.len()
                        && objects.iter().zip(&expected).all(|(object, want)| {
                            get_dotted(object, member_field).and_then(Value::as_str) == Some(*want)
                        })
                });
            if !matches {
                findings.insert(Finding::new(
                    "contract_slice_projected_sequence_mismatch",
                    format!("{slice_id}:{field}:{member_field}"),
                ));
            }
        }
    }

    // 4f. Array cardinality: a dotted array field must satisfy an optional `min`,
    //     `max`, and/or `unique_by` (uniqueness of a projected member field).
    //     A non-array field or a requirement declaring no constraint at all fails
    //     closed rather than passing vacuously.
    if let Some(requirements) = rules_array(slice, "array_cardinality", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "min", "max", "unique_by"],
                slice_id,
                "array_cardinality",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let min = requirement.get("min").and_then(Value::as_u64);
            let max = requirement.get("max").and_then(Value::as_u64);
            let unique_by = requirement.get("unique_by").and_then(Value::as_str);
            // Fail-closed: a present-but-wrong-typed bound (e.g. `min: "1"`) would be
            // read as absent and silently ignored, weakening the check.
            let bad_bound = |name: &str| {
                requirement
                    .get(name)
                    .is_some_and(|value| value.as_u64().is_none())
            };
            if bad_bound("min")
                || bad_bound("max")
                || requirement
                    .get("unique_by")
                    .is_some_and(|value| !value.is_string())
            {
                findings.insert(Finding::new(
                    "contract_slice_array_cardinality_malformed",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            }
            if min.is_none() && max.is_none() && unique_by.is_none() {
                findings.insert(Finding::new(
                    "contract_slice_array_cardinality_malformed",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            }
            let Some(objects) = get_dotted(spec, field).and_then(Value::as_array) else {
                findings.insert(Finding::new(
                    "contract_slice_array_cardinality_bad_field",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            };
            let count = objects.len() as u64;
            if min.is_some_and(|min| count < min) {
                findings.insert(Finding::new(
                    "contract_slice_array_below_min",
                    format!("{slice_id}:{field}"),
                ));
            }
            if max.is_some_and(|max| count > max) {
                findings.insert(Finding::new(
                    "contract_slice_array_above_max",
                    format!("{slice_id}:{field}"),
                ));
            }
            if let Some(unique_by) = unique_by {
                let mut seen = BTreeSet::new();
                for object in objects {
                    let Some(key) = get_dotted(object, unique_by).and_then(scalar_str) else {
                        findings.insert(Finding::new(
                            "contract_slice_array_not_unique",
                            format!("{slice_id}:{field}:{unique_by}:<missing>"),
                        ));
                        continue;
                    };
                    if !seen.insert(key.clone()) {
                        findings.insert(Finding::new(
                            "contract_slice_array_not_unique",
                            format!("{slice_id}:{field}:{unique_by}:{key}"),
                        ));
                    }
                }
            }
        }
    }

    // 4g. Projected value-set coverage: the SET of `member_field` values across the
    //     array-of-objects at `field` must equal `exact_values` exactly — every
    //     declared value present, nothing extra. Catches a copy-pasted row that
    //     drops a distinct class (a registry that must cover a fixed class set).
    //     A non-array field or a non-string projected member fails closed.
    if let Some(requirements) = rules_array(slice, "projected_value_sets", slice_id, findings) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "member_field", "exact_values"],
                slice_id,
                "projected_value_sets",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let member_field = requirement
                .get("member_field")
                .and_then(Value::as_str)
                .unwrap_or("");
            if has_non_string_element(requirement.get("exact_values")) {
                findings.insert(Finding::new(
                    "contract_slice_malformed_policy_value",
                    format!("{slice_id}:projected_value_sets.exact_values:{field}"),
                ));
                continue;
            }
            let expected: BTreeSet<&str> = requirement
                .get("exact_values")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let Some(objects) = get_dotted(spec, field).and_then(Value::as_array) else {
                findings.insert(Finding::new(
                    "contract_slice_projected_set_bad_field",
                    format!("{slice_id}:{field}"),
                ));
                continue;
            };
            let mut actual: BTreeSet<&str> = BTreeSet::new();
            for object in objects {
                match get_dotted(object, member_field).and_then(Value::as_str) {
                    Some(value) => {
                        actual.insert(value);
                    }
                    None => {
                        findings.insert(Finding::new(
                            "contract_slice_projected_set_unexpected",
                            format!("{slice_id}:{field}:{member_field}:<non-string>"),
                        ));
                    }
                }
            }
            for want in &expected {
                if !actual.contains(want) {
                    findings.insert(Finding::new(
                        "contract_slice_projected_set_missing",
                        format!("{slice_id}:{field}:{member_field}:{want}"),
                    ));
                }
            }
            for got in &actual {
                if !expected.contains(got) {
                    findings.insert(Finding::new(
                        "contract_slice_projected_set_unexpected",
                        format!("{slice_id}:{field}:{member_field}:{got}"),
                    ));
                }
            }
        }
    }

    // 5. Migration declarations (optional): each retired source must declare a
    //    retired_primary_path disposition, a Buck2 gate replacement target, and
    //    an interpreter-script legacy path. This proves a Python validator is
    //    being retired onto this gate rather than run in parallel.
    if let Some(rows) = slice
        .get("source_migration_slice")
        .and_then(Value::as_array)
    {
        for (index, row) in rows.iter().enumerate() {
            let key = format!("{slice_id}:migration[{index}]");
            if row.get("disposition").and_then(Value::as_str) != Some("retired_primary_path") {
                findings.insert(Finding::new(
                    "contract_slice_migration_not_retired",
                    key.clone(),
                ));
            }
            let target = row
                .get("replacement_target")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !(target.starts_with("//ci/facade/") && target.ends_with("-gate")) {
                findings.insert(Finding::new(
                    "contract_slice_migration_bad_target",
                    key.clone(),
                ));
            }
            let legacy = row
                .get("legacy_path")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !(legacy.ends_with(".py") || legacy.ends_with(".sh")) {
                findings.insert(Finding::new("contract_slice_migration_bad_legacy", key));
            }
        }
    }
}

/// Evaluate one `conditional_assertions` entry against a single object-array
/// member. Ports the retired Python validators' member-specific business
/// rules (e.g. "the KR-CSAP pack row's `min_tier` must be `sovereign_cell`",
/// "the `cross_border.refused` event row's `severity` must be `CRITICAL`")
/// as policy DATA rather than hardcoded Rust — the gate stays generic while
/// per-slice authors express which specific member pins which specific value.
///
/// Exactly one filter key (`when_member` / `when_member_in`, or neither for
/// "applies to every member") and exactly one mode key (`must_equal` /
/// `must_be_true` / `must_contain` / `must_subset_of`) is expected; an
/// assertion with no recognized mode key is itself flagged rather than
/// silently matching everything, so a policy typo cannot silently disarm it.
fn evaluate_conditional_assertion(
    assertion: &Value,
    object: &Value,
    member_id: &str,
    slice_id: &str,
    field: &str,
    findings: &mut BTreeSet<Finding>,
) {
    check_keys(
        assertion,
        &[
            "when_member",
            "when_member_in",
            "field",
            "must_equal",
            "must_be_true",
            "must_contain",
            "must_subset_of",
        ],
        slice_id,
        "conditional_assertions",
        findings,
    );

    let target_field = assertion.get("field").and_then(Value::as_str).unwrap_or("");
    let key = format!("{slice_id}:{field}:{member_id}:{target_field}");

    // Validate the assertion SHAPE first, independent of whether the selector
    // matches this row: a malformed selector/mode on a row the selector skips must
    // still RED, otherwise a typo'd rule that happens to match no committed row is a
    // silent fail-open.
    let when_member = assertion.get("when_member");
    let when_member_in = assertion.get("when_member_in");

    // Exactly one selector, well-typed: reject both present, a non-string
    // `when_member`, and a `when_member_in` that is not an array of only strings
    // (a non-string element would be silently dropped and narrow the selector).
    let selector_ok = match (when_member, when_member_in) {
        (Some(_), Some(_)) => false,
        (Some(one), None) => one.is_string(),
        (None, Some(many)) => many
            .as_array()
            .is_some_and(|list| list.iter().all(Value::is_string)),
        (None, None) => true,
    };
    if !selector_ok {
        findings.insert(Finding::new(
            "contract_slice_conditional_assertion_bad_selector",
            key,
        ));
        return;
    }

    // Exactly one mode, well-typed. Zero modes (a typo'd key), more than one, or a
    // present-but-wrong-typed mode value all fail closed.
    const MODE_KEYS: [&str; 4] = [
        "must_equal",
        "must_be_true",
        "must_contain",
        "must_subset_of",
    ];
    let modes_present = MODE_KEYS
        .iter()
        .filter(|mode| assertion.get(**mode).is_some())
        .count();
    if modes_present == 0 {
        findings.insert(Finding::new(
            "contract_slice_conditional_assertion_no_mode",
            key,
        ));
        return;
    }
    if modes_present > 1 {
        findings.insert(Finding::new(
            "contract_slice_conditional_assertion_multiple_modes",
            key,
        ));
        return;
    }
    let mode_well_typed = if let Some(mode) = assertion.get("must_equal") {
        mode.is_string()
    } else if let Some(mode) = assertion.get("must_be_true") {
        mode.as_bool() == Some(true)
    } else if let Some(mode) = assertion.get("must_contain") {
        mode.is_array()
    } else if let Some(mode) = assertion.get("must_subset_of") {
        mode.is_array()
    } else {
        unreachable!("exactly one mode present")
    };
    if !mode_well_typed {
        findings.insert(Finding::new(
            "contract_slice_conditional_assertion_bad_mode",
            key,
        ));
        return;
    }

    // Shape is valid — decide whether the assertion applies to THIS row.
    let applies = match (when_member, when_member_in) {
        (Some(one), None) => one.as_str() == Some(member_id),
        (None, Some(many)) => many.as_array().is_some_and(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .any(|m| m == member_id)
        }),
        (None, None) => true,
        (Some(_), Some(_)) => unreachable!("both-selector case rejected above"),
    };
    if !applies {
        return;
    }

    // Evaluate the single, well-typed mode against the row. Each is dotted (not a
    // flat `object.get`) so a rule can pin a nested member field (e.g.
    // `capability_overrides.enforcement.lane_id`).
    if let Some(expected) = assertion.get("must_equal").and_then(Value::as_str) {
        if get_dotted(object, target_field).and_then(Value::as_str) != Some(expected) {
            findings.insert(Finding::new(
                "contract_slice_conditional_field_not_equal",
                key,
            ));
        }
        return;
    }
    if assertion.get("must_be_true").is_some() {
        if get_dotted(object, target_field).and_then(Value::as_bool) != Some(true) {
            findings.insert(Finding::new(
                "contract_slice_conditional_field_not_true",
                key,
            ));
        }
        return;
    }
    if let Some(members) = assertion.get("must_contain").and_then(Value::as_array) {
        // Fail-closed: an absent/non-array subject cannot contain the members.
        let Some(subject) = get_dotted(object, target_field).and_then(Value::as_array) else {
            findings.insert(Finding::new(
                "contract_slice_conditional_field_missing_contains",
                format!("{key}:<subject-not-array>"),
            ));
            return;
        };
        let present: BTreeSet<&str> = subject.iter().filter_map(Value::as_str).collect();
        for expected in members.iter().filter_map(Value::as_str) {
            if !present.contains(expected) {
                findings.insert(Finding::new(
                    "contract_slice_conditional_field_missing_contains",
                    format!("{key}:{expected}"),
                ));
            }
        }
        return;
    }
    if let Some(allowed_values) = assertion.get("must_subset_of").and_then(Value::as_array) {
        let allowed: BTreeSet<&str> = allowed_values.iter().filter_map(Value::as_str).collect();
        // Fail-closed: an absent/non-array subject is a violation, not a vacuous
        // pass (a missing field would otherwise loop zero times and green).
        let Some(subject) = get_dotted(object, target_field).and_then(Value::as_array) else {
            findings.insert(Finding::new(
                "contract_slice_conditional_field_not_subset",
                format!("{key}:<subject-not-array>"),
            ));
            return;
        };
        // A non-string subject element is a violation, not silently dropped.
        for value in subject {
            let ok = value.as_str().is_some_and(|v| allowed.contains(v));
            if !ok {
                let repr = scalar_str(value).unwrap_or_else(|| "<non-scalar>".to_owned());
                findings.insert(Finding::new(
                    "contract_slice_conditional_field_not_subset",
                    format!("{key}:{repr}"),
                ));
            }
        }
    }
}

/// Collect a slice field that is an array-of-strings into owned strings.
fn string_array(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a dotted path (`a.b.c`) to a nested value.
fn get_dotted<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

/// Canonicalize a JSON scalar leaf (string, number, or bool) to its string
/// form so `enum_constraints` / `required_array_members` can pin a numeric or
/// boolean literal (e.g. a `14.4` burn-rate threshold or a `90`-day lifetime)
/// the same way they pin a string literal. `Null`/array/object leaves have no
/// canonical scalar form and yield `None`, matching the existing fail-closed
/// (non-match) behavior for a missing/wrong-shaped field.
fn scalar_str(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

/// True when `needle` (matched case-insensitively) appears in any string leaf
/// of `value` — including object keys.
fn recursively_contains(value: &Value, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    contains_lowered(value, &needle)
}

fn contains_lowered(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text.to_ascii_lowercase().contains(needle),
        Value::Array(items) => items.iter().any(|item| contains_lowered(item, needle)),
        Value::Object(map) => map.iter().any(|(key, val)| {
            key.to_ascii_lowercase().contains(needle) || contains_lowered(val, needle)
        }),
        _ => false,
    }
}

/// True for a Unicode format / zero-width / bidi-control code point that carries
/// no visible glyph and so can be injected to split a forbidden phrase past the
/// scan (`produc<U+200B>tion-ready`). A FORBIDDEN check must be fail-safe (a
/// false RED is safe; a false GREEN hides a prohibited claim), so we canonicalize
/// to an `[a-z0-9]`-only sequence: lowercase and drop EVERYTHING else — spaces,
/// punctuation, and all zero-width/bidi/control chars — uniformly. Marker
/// `production ready` -> `productionready`; this catches `production-ready`,
/// `production ready`, `production<U+200B>ready`, and every other separator or
/// zero-width obfuscation with one rule, and single-token markers (`python3`)
/// stay substring so `python311` still trips `python3`.
///
/// ponytail: intentionally OVER-STRICT — a legitimate `preproductionreadying`
/// token trips `productionready`. Accepted: over-match is the safe direction for
/// a prohibition check.
fn canonical_alnum(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

/// A bidirectional REORDER control (embeddings/overrides U+202A–202E and
/// isolates U+2066–2069) — the codepoints that make rendered order differ from
/// logical order. Plain directional marks (U+200E/200F) are excluded: they are
/// used legitimately in i18n text and do not reorder surrounding runs.
fn is_bidi_reorder_control(character: char) -> bool {
    matches!(character, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// True when any string leaf or object key of `value` contains a bidi-reorder
/// control. Presence alone is the violation, so it is checked before (not by)
/// canonicalization, which would drop the control and leave the reordered text.
fn contains_bidi_reorder_control(value: &Value) -> bool {
    match value {
        Value::String(text) => text.chars().any(is_bidi_reorder_control),
        Value::Array(items) => items.iter().any(contains_bidi_reorder_control),
        Value::Object(map) => map.iter().any(|(key, val)| {
            key.chars().any(is_bidi_reorder_control) || contains_bidi_reorder_control(val)
        }),
        _ => false,
    }
}

/// True when the canonical (`[a-z0-9]`-only) `needle` appears as a substring of
/// the canonical form of any string leaf of `value` — including object keys. Any
/// node whose reference is in `excluded` (and its subtree) is skipped, so a
/// slice can carve out a sub-tree that legitimately quotes a forbidden phrase
/// (e.g. a `claim_boundary` block that enumerates the phrases it forbids)
/// without self-tripping the scan.
fn recursively_contains_normalized(
    value: &Value,
    needle_canonical: &str,
    excluded: &[&Value],
) -> bool {
    if needle_canonical.is_empty() {
        return false;
    }
    if excluded.iter().any(|node| std::ptr::eq(*node, value)) {
        return false;
    }
    match value {
        Value::String(text) => canonical_alnum(text).contains(needle_canonical),
        Value::Array(items) => items
            .iter()
            .any(|item| recursively_contains_normalized(item, needle_canonical, excluded)),
        Value::Object(map) => map.iter().any(|(key, val)| {
            canonical_alnum(key).contains(needle_canonical)
                || recursively_contains_normalized(val, needle_canonical, excluded)
        }),
        _ => false,
    }
}

/// True when `list` is a JSON array containing at least one non-string element.
/// A mistyped string-list policy config (`values`, `allowed`, `markers`, …) must
/// fail closed rather than let `filter_map(as_str)` silently drop the element and
/// weaken the check to something the author did not intend.
fn has_non_string_element(list: Option<&Value>) -> bool {
    list.and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(|item| !item.is_string()))
}

/// The array at `key`, or `None` — but a `key` that is PRESENT yet not an array
/// (a mistyped `{...}` where a list-of-rules is expected) emits a malformed
/// finding, so the whole primitive fails closed instead of silently no-opping to
/// Green. Every list-of-rules primitive routes its top-level lookup through here.
fn rules_array<'a>(
    container: &'a Value,
    key: &str,
    slice_id: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<&'a Vec<Value>> {
    match container.get(key) {
        Some(Value::Array(items)) => Some(items),
        Some(_) => {
            findings.insert(Finding::new(
                "contract_slice_malformed_policy_value",
                format!("{slice_id}:{key}"),
            ));
            None
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_slice_spec() -> Value {
        json!({
            "slice_id": "exemplar",
            "spec_kind": "contract-slice",
            "cloud_ci_gate": GATE_ID,
            "required_contract_fields": ["field_a", "field_b"],
            "non_claims": ["fixture only; not live evidence"]
        })
    }

    fn policy_with(slice: Value) -> Value {
        json!({
            "gate_id": GATE_ID,
            "primary_execution_path": REQUIRED_PRIMARY_EXECUTION_PATH,
            "slices": [slice]
        })
    }

    fn exemplar_slice() -> Value {
        json!({
            "slice_id": "exemplar",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": ["slice_id", "spec_kind", "cloud_ci_gate", "required_contract_fields", "non_claims"],
            "enum_constraints": [{ "field": "spec_kind", "allowed": ["contract-slice"] }]
        })
    }

    fn corpus_with(spec: Value) -> BTreeMap<String, Value> {
        BTreeMap::from([("fixtures/exemplar-slice.json".to_owned(), spec)])
    }

    #[test]
    fn get_dotted_walks_nested_objects() {
        let v = json!({ "a": { "b": { "c": 1 } } });
        assert_eq!(get_dotted(&v, "a.b.c"), Some(&json!(1)));
        assert_eq!(get_dotted(&v, "a.b.missing"), None);
    }

    #[test]
    fn recursively_contains_is_case_insensitive_and_scans_keys() {
        let v = json!({ "verification": "Python3 check.py" });
        assert!(recursively_contains(&v, "python3"));
        let keyed = json!({ "kubectl apply": true });
        assert!(recursively_contains(&keyed, "kubectl apply"));
        assert!(!recursively_contains(
            &json!({ "ok": "rust gate" }),
            "python3"
        ));
    }

    #[test]
    fn valid_slice_is_green() {
        let report = evaluate_configured(
            &policy_with(exemplar_slice()),
            &corpus_with(valid_slice_spec()),
        );
        assert_eq!(report.verdict, Verdict::Green, "{:#?}", report.findings);
    }

    #[test]
    fn missing_required_field_is_red() {
        let mut spec = valid_slice_spec();
        spec.as_object_mut().unwrap().remove("non_claims");
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_missing_required_field")
        );
    }

    #[test]
    fn baked_in_interpreter_is_red() {
        let mut spec = valid_slice_spec();
        spec.as_object_mut().unwrap().insert(
            "verification".to_owned(),
            json!("python3 scripts/tests/x_check.py"),
        );
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_forbidden_marker")
        );
    }

    #[test]
    fn enum_violation_is_red() {
        let mut spec = valid_slice_spec();
        spec.as_object_mut()
            .unwrap()
            .insert("spec_kind".to_owned(), json!("not-a-contract-slice"));
        let report = evaluate_configured(&policy_with(exemplar_slice()), &corpus_with(spec));
        assert!(report.violations.contains("contract_slice_enum_violation"));
    }

    #[test]
    fn wrong_primary_path_is_red() {
        let mut policy = policy_with(exemplar_slice());
        policy
            .as_object_mut()
            .unwrap()
            .insert("primary_execution_path".to_owned(), json!("python_script"));
        let report = evaluate_configured(&policy, &corpus_with(valid_slice_spec()));
        assert!(
            report
                .violations
                .contains("contract_slice_primary_path_not_rust")
        );
    }

    #[test]
    fn absent_spec_is_red_not_panic() {
        let report = evaluate_configured(&policy_with(exemplar_slice()), &BTreeMap::new());
        assert!(report.violations.contains("contract_slice_spec_absent"));
    }

    #[test]
    fn bad_migration_declaration_is_red() {
        let slice = json!({
            "slice_id": "mig",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "source_migration_slice": [
                { "legacy_path": "scripts/tests/x.py", "replacement_target": "//somewhere:else", "disposition": "kept" }
            ]
        });
        let report = evaluate_configured(&policy_with(slice), &corpus_with(valid_slice_spec()));
        assert!(
            report
                .violations
                .contains("contract_slice_migration_not_retired")
        );
        assert!(
            report
                .violations
                .contains("contract_slice_migration_bad_target")
        );
    }

    #[test]
    fn enum_constraint_matches_numeric_and_bool_scalar_leaves() {
        let slice = json!({
            "slice_id": "numeric",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "enum_constraints": [
                { "field": "threshold", "allowed": ["14.4"], "match_scalar": true },
                { "field": "enabled", "allowed": ["true"], "match_scalar": true }
            ]
        });
        let mut spec = valid_slice_spec();
        spec["threshold"] = json!(14.4);
        spec["enabled"] = json!(true);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        let mut spec = valid_slice_spec();
        spec["threshold"] = json!(99.9);
        spec["enabled"] = json!(true);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(report.violations.contains("contract_slice_enum_violation"));
    }

    #[test]
    fn required_array_members_matches_numeric_array_leaves() {
        let slice = json!({
            "slice_id": "numeric-arr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_array_members": [
                { "field": "canary_stages_percent", "members": ["1", "10", "50", "100"], "match_scalar": true }
            ]
        });
        let mut spec = valid_slice_spec();
        spec["canary_stages_percent"] = json!([1, 10, 50, 100]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        let mut spec = valid_slice_spec();
        spec["canary_stages_percent"] = json!([1, 10, 50]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_missing_array_member")
        );
    }

    #[test]
    fn required_array_members_superset_is_green_missing_is_red() {
        let slice = json!({
            "slice_id": "arr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_array_members": [
                { "field": "required_contract_fields", "members": ["field_a", "field_b"] }
            ]
        });
        // valid_slice_spec has required_contract_fields = [field_a, field_b] -> green
        assert_eq!(
            evaluate_configured(
                &policy_with(slice.clone()),
                &corpus_with(valid_slice_spec())
            )
            .verdict,
            Verdict::Green
        );
        // drop field_b -> missing member is red
        let mut spec = valid_slice_spec();
        spec["required_contract_fields"] = json!(["field_a"]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_missing_array_member")
        );
    }

    #[test]
    fn unknown_slice_key_is_red_fail_closed_on_policy_typo() {
        let slice = json!({
            "slice_id": "typo",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_field": ["slice_id"]  // typo: should be required_fields
        });
        let report = evaluate_configured(&policy_with(slice), &corpus_with(valid_slice_spec()));
        assert!(
            report
                .violations
                .contains("contract_slice_unknown_policy_key")
        );
    }

    #[test]
    fn nested_requirement_typo_is_red_not_silently_disarmed() {
        // `member` instead of `members` would otherwise skip the membership check
        // and green a slice that enforces nothing; the nested key validator REDs it.
        let slice = json!({
            "slice_id": "nested",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_array_members": [{ "field": "required_contract_fields", "member": ["field_a"] }]
        });
        let report = evaluate_configured(&policy_with(slice), &corpus_with(valid_slice_spec()));
        assert!(
            report
                .violations
                .contains("contract_slice_unknown_policy_key")
        );
    }

    #[test]
    fn required_true_field_rejects_false_and_absent() {
        let slice = json!({
            "slice_id": "boolt",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_true_fields": ["flag"]
        });
        let mut spec = valid_slice_spec();
        spec["flag"] = json!(true);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        let mut spec = valid_slice_spec();
        spec["flag"] = json!(false);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_field_not_true")
        );
    }

    #[test]
    fn object_array_members_enforce_ids_fields_and_per_member_enums() {
        let slice = json!({
            "slice_id": "oarr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "gate.inputs",
                "member_key": "id",
                "members": ["G1", "G2"],
                "member_required_fields": ["id", "source_adr"],
                "member_enum_constraints": [{ "field": "refusal", "allowed": ["fail_closed"] }]
            }]
        });
        // green: both ids present, fields present, refusal pinned
        let mut spec = valid_slice_spec();
        spec["gate"] = json!({ "inputs": [
            { "id": "G1", "source_adr": "ADR-1", "refusal": "fail_closed" },
            { "id": "G2", "source_adr": "ADR-1", "refusal": "fail_closed" }
        ]});
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: drop G2, drop a field, flip refusal to best-effort
        let mut spec = valid_slice_spec();
        spec["gate"] = json!({ "inputs": [
            { "id": "G1", "refusal": "best_effort" }
        ]});
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_missing_object_array_member")
        );
        assert!(
            report
                .violations
                .contains("contract_slice_object_member_missing_field")
        );
        assert!(
            report
                .violations
                .contains("contract_slice_object_member_enum_violation")
        );
    }

    #[test]
    fn required_markers_scoped_to_field_not_whole_document() {
        let slice = json!({
            "slice_id": "markers",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_markers": [
                { "field": "conditionals", "markers": ["sovereign_cell", "airgapped_cell"] }
            ]
        });
        // green: both markers present inside the scoped field.
        let mut spec = valid_slice_spec();
        spec["conditionals"] =
            json!([{ "if_tier": "sovereign_cell" }, { "if_tier": "airgapped_cell" }]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: marker only appears OUTSIDE the scoped field (e.g. in an unrelated
        // enum) — an unscoped whole-document search would have falsely greened this.
        let mut spec = valid_slice_spec();
        spec["tier_enum"] = json!(["sovereign_cell", "airgapped_cell"]);
        spec["conditionals"] = json!([{ "if_tier": "sovereign_cell" }]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_required_marker_missing")
        );
    }

    #[test]
    fn conditional_assertions_pin_a_specific_members_field_value() {
        let slice = json!({
            "slice_id": "pins",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "matrix_rows",
                "member_key": "pack_id",
                "members": ["CSAP", "SOC2"],
                "conditional_assertions": [
                    { "when_member": "CSAP", "field": "min_tier", "must_equal": "sovereign_cell" },
                    { "when_member_in": ["CSAP", "SOC2"], "field": "attestor_required", "must_be_true": true },
                    { "field": "supported_tiers", "must_subset_of": ["multi_region", "sovereign_cell"] },
                    { "when_member": "CSAP", "field": "evidence_fields", "must_contain": ["hash", "signer"] }
                ]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["matrix_rows"] = json!([
            { "pack_id": "CSAP", "min_tier": "sovereign_cell", "attestor_required": true, "supported_tiers": ["sovereign_cell"], "evidence_fields": ["hash", "signer", "extra"] },
            { "pack_id": "SOC2", "min_tier": "multi_region", "attestor_required": true, "supported_tiers": ["multi_region"] }
        ]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );

        // red: CSAP downgraded off sovereign_cell, SOC2 loses attestor_required,
        // a row's supported_tiers escapes the allowed set, CSAP evidence_fields
        // drops a required member.
        let mut spec = valid_slice_spec();
        spec["matrix_rows"] = json!([
            { "pack_id": "CSAP", "min_tier": "multi_region", "attestor_required": true, "supported_tiers": ["airgapped_cell"], "evidence_fields": ["hash"] },
            { "pack_id": "SOC2", "min_tier": "multi_region", "attestor_required": false, "supported_tiers": ["multi_region"] }
        ]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_conditional_field_not_equal")
        );
        assert!(
            report
                .violations
                .contains("contract_slice_conditional_field_not_true")
        );
        assert!(
            report
                .violations
                .contains("contract_slice_conditional_field_not_subset")
        );
        assert!(
            report
                .violations
                .contains("contract_slice_conditional_field_missing_contains")
        );
    }

    #[test]
    fn conditional_assertion_without_a_mode_key_is_red_not_a_silent_noop() {
        let slice = json!({
            "slice_id": "nomode",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "matrix_rows",
                "member_key": "pack_id",
                "members": ["CSAP"],
                "conditional_assertions": [{ "when_member": "CSAP", "field": "min_tier" }]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["matrix_rows"] = json!([{ "pack_id": "CSAP", "min_tier": "sovereign_cell" }]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_conditional_assertion_no_mode")
        );
    }

    #[test]
    fn skip_universal_markers_bypasses_the_universal_scan_only() {
        let slice = json!({
            "slice_id": "sharedreg",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "skip_universal_markers": true
        });
        // A shared-registry doc that legitimately narrates a retired CLI's name
        // in provenance prose (e.g. "oya-dev-cli constitution-cite gate removed")
        // stays green when the slice opts out.
        let mut spec = valid_slice_spec();
        spec["retirement_note"] =
            json!("the oya-dev-cli gate was removed; see python3 x.py history");
        assert_eq!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec)).verdict,
            Verdict::Green
        );

        // Without the opt-out, the exact same content REDs (regression guard: the
        // opt-out must not become the default).
        let slice_no_skip = json!({
            "slice_id": "sharedreg",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": []
        });
        let mut spec = valid_slice_spec();
        spec["retirement_note"] = json!("the oya-dev-cli gate was removed");
        assert!(
            evaluate_configured(&policy_with(slice_no_skip.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_forbidden_marker")
        );

        // The slice's OWN forbidden_markers still fire even with the universal
        // scan skipped — the opt-out is narrow, not a blanket bypass.
        let slice_with_custom = json!({
            "slice_id": "sharedreg",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "skip_universal_markers": true,
            "forbidden_markers": ["production ready"]
        });
        let mut spec = valid_slice_spec();
        spec["claim"] = json!("production ready");
        assert!(
            evaluate_configured(&policy_with(slice_with_custom), &corpus_with(spec))
                .violations
                .contains("contract_slice_forbidden_marker")
        );
    }

    #[test]
    fn exact_array_fields_reject_reorder_and_extras_not_just_missing_members() {
        let slice = json!({
            "slice_id": "exactarr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "exact_array_fields": [
                { "field": "tier_enum", "values": ["multi_region", "single_region", "sovereign_cell"] }
            ]
        });
        let mut spec = valid_slice_spec();
        spec["tier_enum"] = json!(["multi_region", "single_region", "sovereign_cell"]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );

        // red: reordered (a required_array_members superset check would miss this).
        let mut spec = valid_slice_spec();
        spec["tier_enum"] = json!(["single_region", "multi_region", "sovereign_cell"]);
        assert!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_not_exact")
        );

        // red: an unlisted extra value snuck in.
        let mut spec = valid_slice_spec();
        spec["tier_enum"] = json!([
            "multi_region",
            "single_region",
            "sovereign_cell",
            "airgapped_cell"
        ]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_not_exact")
        );
    }

    #[test]
    fn exact_members_rejects_an_unlisted_extra_member_key() {
        let slice = json!({
            "slice_id": "exactmembers",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "regimes",
                "member_key": "regime_id",
                "members": ["KR_PIPA", "EU_GDPR"],
                "exact_members": true
            }]
        });
        let mut spec = valid_slice_spec();
        spec["regimes"] = json!([{ "regime_id": "KR_PIPA" }, { "regime_id": "EU_GDPR" }]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );

        // red: an extra, undeclared regime row slipped in.
        let mut spec = valid_slice_spec();
        spec["regimes"] = json!([
            { "regime_id": "KR_PIPA" },
            { "regime_id": "EU_GDPR" },
            { "regime_id": "US_HIPAA" }
        ]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_unexpected_object_array_member")
        );
    }

    #[test]
    fn object_array_member_checks_resolve_nested_dotted_fields() {
        // Proves member_required_fields / member_enum_constraints /
        // conditional_assertions can reach a nested field (e.g.
        // `capability_overrides.enforcement.lane_id`), not only a flat one —
        // needed to pin the RESIDENCY-001 artifact-registry governance rows.
        let slice = json!({
            "slice_id": "nesteddotted",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "rows",
                "member_key": "artifact_id",
                "members": ["a1"],
                "member_required_fields": ["overrides.enforcement.lane_id"],
                "member_enum_constraints": [{ "field": "overrides.profile", "allowed": ["schema"] }],
                "conditional_assertions": [
                    { "when_member": "a1", "field": "overrides.enforcement.lane_id", "must_equal": "residency-lane" }
                ]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{
            "artifact_id": "a1",
            "overrides": { "profile": "schema", "enforcement": { "lane_id": "residency-lane" } }
        }]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );

        // red: nested lane_id pinned to the wrong value.
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{
            "artifact_id": "a1",
            "overrides": { "profile": "schema", "enforcement": { "lane_id": "other-lane" } }
        }]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert!(
            report
                .violations
                .contains("contract_slice_conditional_field_not_equal")
        );
    }

    #[test]
    fn field_implies_required_ports_the_flag_gated_companion_fields_rule() {
        let slice = json!({
            "slice_id": "implies",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "regimes",
                "member_key": "regime_id",
                "members": ["KR_PIPA", "EU_GDPR"],
                "field_implies_required": [
                    { "if_field": "third_party_attestor_required", "then_required_fields": ["attestor_required_for_tiers", "evidence_requirements"] }
                ]
            }]
        });
        // green: flag not set on EU_GDPR (companion fields irrelevant), flag set
        // and satisfied with a non-empty array on KR_PIPA.
        let mut spec = valid_slice_spec();
        spec["regimes"] = json!([
            { "regime_id": "KR_PIPA", "third_party_attestor_required": true, "attestor_required_for_tiers": ["sovereign_cell"], "evidence_requirements": ["x"] },
            { "regime_id": "EU_GDPR" }
        ]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: flag set but attestor_required_for_tiers is an empty array (present
        // but content-free) and evidence_requirements is absent entirely.
        let mut spec = valid_slice_spec();
        spec["regimes"] = json!([
            { "regime_id": "KR_PIPA", "third_party_attestor_required": true, "attestor_required_for_tiers": [] },
            { "regime_id": "EU_GDPR" }
        ]);
        let report = evaluate_configured(&policy_with(slice), &corpus_with(spec));
        assert_eq!(
            report
                .findings
                .iter()
                .filter(|f| f.code == "contract_slice_conditional_required_field_absent")
                .count(),
            2,
            "{:#?}",
            report.findings
        );
    }

    // ---- Phase 2: Group A fail-closed hardening (RED-first per CodeRabbit) ----

    #[test]
    fn exact_array_fields_counts_a_nonstring_extra_as_mismatch() {
        // A non-string extra element must be a mismatch, not silently dropped by
        // `filter_map(as_str)` (which would false-green `["a","b", 42]` vs `["a","b"]`).
        let slice = json!({
            "slice_id": "exactarr-nonstr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "exact_array_fields": [
                { "field": "tier_enum", "values": ["a", "b"] }
            ]
        });
        let mut spec = valid_slice_spec();
        spec["tier_enum"] = json!(["a", "b", 42]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_not_exact"),
            "a non-string extra must be counted as a mismatch"
        );
    }

    #[test]
    fn conditional_assertions_reject_both_selectors_present() {
        // Both `when_member` and `when_member_in` present: the pre-hardening code
        // silently honored only `when_member`, dropping the `when_member_in` intent.
        let slice = json!({
            "slice_id": "bothsel",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "rows",
                "member_key": "id",
                "members": ["A"],
                "conditional_assertions": [
                    { "when_member": "A", "when_member_in": ["A", "B"], "field": "x", "must_be_true": true }
                ]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{ "id": "A", "x": true }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_conditional_assertion_bad_selector"),
            "an assertion carrying two selectors must fail closed"
        );
    }

    #[test]
    fn conditional_assertions_reject_multiple_modes() {
        // Two mode keys: the pre-hardening code silently took `must_equal` first and
        // ignored the rest, so a typo could disarm the intended mode.
        let slice = json!({
            "slice_id": "multimode",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "rows",
                "member_key": "id",
                "members": ["A"],
                "conditional_assertions": [
                    { "when_member": "A", "field": "x", "must_equal": "v", "must_be_true": true }
                ]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{ "id": "A", "x": "v" }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_conditional_assertion_multiple_modes"),
            "an assertion carrying two modes must fail closed"
        );
    }

    #[test]
    fn conditional_assertion_must_subset_of_rejects_a_nonstring_value() {
        // A non-string element in the subject array must be rejected, not dropped by
        // `filter_map(as_str)` (which would let `999` escape the subset check).
        let slice = json!({
            "slice_id": "subsetnonstr",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "rows",
                "member_key": "id",
                "members": ["A"],
                "conditional_assertions": [
                    { "when_member": "A", "field": "tiers", "must_subset_of": ["x", "y"] }
                ]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{ "id": "A", "tiers": ["x", 999] }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_conditional_field_not_subset"),
            "a non-string subject element must fail the subset check"
        );
    }

    #[test]
    fn required_markers_malformed_empty_or_missing_field_is_red() {
        // Empty markers list would otherwise be a silent no-op that enforces nothing.
        let empty_markers = json!({
            "slice_id": "rm-empty",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_markers": [{ "field": "conditionals", "markers": [] }]
        });
        let mut spec = valid_slice_spec();
        spec["conditionals"] = json!([{ "if_tier": "x" }]);
        assert!(
            evaluate_configured(&policy_with(empty_markers), &corpus_with(spec.clone()))
                .violations
                .contains("contract_slice_required_markers_malformed"),
            "an empty markers list must fail closed"
        );
        // Missing field must also fail closed rather than search the empty path.
        let missing_field = json!({
            "slice_id": "rm-nofield",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_markers": [{ "markers": ["x"] }]
        });
        assert!(
            evaluate_configured(&policy_with(missing_field), &corpus_with(spec))
                .violations
                .contains("contract_slice_required_markers_malformed"),
            "a missing scope field must fail closed"
        );
    }

    #[test]
    fn field_implies_required_malformed_is_red() {
        // Missing if_field would otherwise silently skip the rule (a false green);
        // an empty then_required_fields would enforce nothing.
        let slice = json!({
            "slice_id": "fir-malformed",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_object_array_members": [{
                "field": "rows",
                "member_key": "id",
                "members": ["A"],
                "field_implies_required": [
                    { "then_required_fields": ["companion"] }
                ]
            }]
        });
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{ "id": "A" }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_field_implies_required_malformed"),
            "a rule with no if_field must fail closed"
        );
    }

    // ---- Phase 3: Group B T1 (B1 required_false_fields, B2 any_of, B3a separators) ----

    #[test]
    fn forbidden_marker_normalizes_separators_so_hyphen_trips_a_spaced_marker() {
        // The real evasion: `production-ready` (hyphen) must trip the marker
        // `production ready` (space). A plain substring scan misses it.
        let slice = json!({
            "slice_id": "sep",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "forbidden_markers": ["production ready"]
        });
        let mut spec = valid_slice_spec();
        spec["claim"] = json!("this substrate is production-ready today");
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_forbidden_marker"),
            "a separator-substituted marker must still be caught"
        );
    }

    #[test]
    fn required_false_fields_rejects_true_and_absent() {
        let slice = json!({
            "slice_id": "boolf",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_false_fields": ["non_necessary_default"]
        });
        let mut spec = valid_slice_spec();
        spec["non_necessary_default"] = json!(false);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // true must be rejected...
        let mut spec = valid_slice_spec();
        spec["non_necessary_default"] = json!(true);
        assert!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_field_not_false")
        );
        // ...and so must an absent field (fail-closed, like required_true_fields).
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(valid_slice_spec()))
                .violations
                .contains("contract_slice_field_not_false")
        );
    }

    #[test]
    fn required_markers_any_of_reds_only_when_no_marker_matches() {
        let slice = json!({
            "slice_id": "anyof",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_markers": [{
                "field": "nonclaims",
                "quantifier": "any_of",
                "markers": ["no ERP settlement authority", "no marketplace settlement authority"]
            }]
        });
        // green: exactly one of the accepted wordings is present.
        let mut spec = valid_slice_spec();
        spec["nonclaims"] = json!(["no ERP settlement authority is claimed"]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: none of the accepted wordings appear.
        let mut spec = valid_slice_spec();
        spec["nonclaims"] = json!(["something unrelated"]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_required_marker_none_present")
        );
    }

    #[test]
    fn required_markers_scope_whole_spec_searches_the_document() {
        let slice = json!({
            "slice_id": "wholemark",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_markers": [{
                "scope": "whole_spec",
                "markers": ["fixture only"]
            }]
        });
        // valid_slice_spec's non_claims already contains "fixture only; not live evidence".
        assert_eq!(
            evaluate_configured(
                &policy_with(slice.clone()),
                &corpus_with(valid_slice_spec())
            )
            .verdict,
            Verdict::Green
        );
        // red: remove the phrase entirely.
        let mut spec = valid_slice_spec();
        spec["non_claims"] = json!(["something else"]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_required_marker_missing")
        );
    }

    #[test]
    fn required_markers_bad_quantifier_is_red() {
        let slice = json!({
            "slice_id": "badq",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "required_markers": [{ "field": "x", "markers": ["m"], "quantifier": "some_of" }]
        });
        let mut spec = valid_slice_spec();
        spec["x"] = json!(["m"]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_required_markers_malformed")
        );
    }

    // ---- Phase 4: Group B T2 (B4 field_patterns, B3b/B3c forbidden, B5/B6/B7) ----

    #[test]
    fn field_patterns_enforce_regex_and_bad_pattern_fails_closed() {
        let slice = json!({
            "slice_id": "pat",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "field_patterns": [{ "field": "signature", "pattern": "^[0-9a-f]{128}$" }]
        });
        // green: exactly 128 hex chars.
        let mut spec = valid_slice_spec();
        spec["signature"] = json!("a".repeat(128));
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: wrong shape (and a missing field is a mismatch too, fail-closed).
        let mut spec = valid_slice_spec();
        spec["signature"] = json!("abcdef");
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_pattern_mismatch")
        );
        // a malformed regex fails closed rather than accepting everything.
        let bad = json!({
            "slice_id": "pat2",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "field_patterns": [{ "field": "x", "pattern": "([unclosed" }]
        });
        let mut spec = valid_slice_spec();
        spec["x"] = json!("whatever");
        assert!(
            evaluate_configured(&policy_with(bad), &corpus_with(spec))
                .violations
                .contains("contract_slice_bad_pattern")
        );
    }

    #[test]
    fn forbidden_field_markers_are_scoped_to_the_field() {
        let slice = json!({
            "slice_id": "ffm",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "forbidden_field_markers": [
                { "field": "can_claim_now", "markers": ["cannot claim yet"] }
            ]
        });
        // green: the forbidden phrase appears elsewhere, but not in the scoped field.
        let mut spec = valid_slice_spec();
        spec["can_claim_now"] = json!(["residency honored"]);
        spec["cannot_claim_yet_block"] = json!(["cannot claim yet: runtime migration"]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: the phrase leaks into the scoped field.
        let mut spec = valid_slice_spec();
        spec["can_claim_now"] = json!(["cannot-claim-yet, oops"]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_forbidden_field_marker")
        );
    }

    #[test]
    fn marker_exclude_fields_carves_a_subtree_out_of_the_scan() {
        let make = |exclude: bool| {
            let mut slice = json!({
                "slice_id": "excl",
                "spec_path": "fixtures/exemplar-slice.json",
                "required_fields": [],
                "forbidden_markers": ["cannot claim yet"]
            });
            if exclude {
                slice["marker_exclude_fields"] = json!(["claim_boundary"]);
            }
            slice
        };
        // The spec quotes the forbidden phrase only inside claim_boundary.
        let mut spec = valid_slice_spec();
        spec["claim_boundary"] = json!({ "forbids": ["cannot claim yet"] });
        // without the carve-out the spec self-trips...
        assert!(
            evaluate_configured(&policy_with(make(false)), &corpus_with(spec.clone()))
                .violations
                .contains("contract_slice_forbidden_marker")
        );
        // ...with the carve-out it is green.
        assert_eq!(
            evaluate_configured(&policy_with(make(true)), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // but the same phrase OUTSIDE the excluded subtree still REDs even with the carve-out.
        let mut spec = valid_slice_spec();
        spec["claim_boundary"] = json!({ "forbids": ["cannot claim yet"] });
        spec["headline"] = json!("we cannot claim yet, really");
        assert!(
            evaluate_configured(&policy_with(make(true)), &corpus_with(spec))
                .violations
                .contains("contract_slice_forbidden_marker")
        );
    }

    #[test]
    fn exact_projected_sequence_enforces_order_and_length() {
        let slice = json!({
            "slice_id": "seq",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "exact_projected_sequence": [
                { "field": "stages", "member_field": "name", "values": ["dev", "canary", "prod"] }
            ]
        });
        let mut spec = valid_slice_spec();
        spec["stages"] = json!([{ "name": "dev" }, { "name": "canary" }, { "name": "prod" }]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: reordered (a set/superset check would miss this).
        let mut spec = valid_slice_spec();
        spec["stages"] = json!([{ "name": "canary" }, { "name": "dev" }, { "name": "prod" }]);
        assert!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_projected_sequence_mismatch")
        );
        // red: an extra trailing stage.
        let mut spec = valid_slice_spec();
        spec["stages"] =
            json!([{ "name": "dev" }, { "name": "canary" }, { "name": "prod" }, { "name": "x" }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_projected_sequence_mismatch")
        );
    }

    #[test]
    fn array_cardinality_enforces_min_max_and_uniqueness() {
        let slice = json!({
            "slice_id": "card",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "array_cardinality": [{ "field": "entries", "min": 2, "max": 3, "unique_by": "id" }]
        });
        let mut spec = valid_slice_spec();
        spec["entries"] = json!([{ "id": "a" }, { "id": "b" }]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // below min.
        let mut spec = valid_slice_spec();
        spec["entries"] = json!([{ "id": "a" }]);
        assert!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_below_min")
        );
        // duplicate id.
        let mut spec = valid_slice_spec();
        spec["entries"] = json!([{ "id": "a" }, { "id": "a" }]);
        assert!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_not_unique")
        );
        // above max.
        let mut spec = valid_slice_spec();
        spec["entries"] = json!([{ "id": "a" }, { "id": "b" }, { "id": "c" }, { "id": "d" }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_above_max")
        );
    }

    #[test]
    fn array_cardinality_malformed_and_bad_field_fail_closed() {
        // No constraint declared at all -> malformed.
        let no_constraint = json!({
            "slice_id": "card2",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "array_cardinality": [{ "field": "entries" }]
        });
        let mut spec = valid_slice_spec();
        spec["entries"] = json!([{ "id": "a" }]);
        assert!(
            evaluate_configured(&policy_with(no_constraint), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_cardinality_malformed")
        );
        // Field is not an array -> bad_field.
        let not_array = json!({
            "slice_id": "card3",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "array_cardinality": [{ "field": "entries", "min": 1 }]
        });
        let mut spec = valid_slice_spec();
        spec["entries"] = json!("not an array");
        assert!(
            evaluate_configured(&policy_with(not_array), &corpus_with(spec))
                .violations
                .contains("contract_slice_array_cardinality_bad_field")
        );
    }

    #[test]
    fn projected_value_sets_enforce_exact_coverage() {
        let slice = json!({
            "slice_id": "vset",
            "spec_path": "fixtures/exemplar-slice.json",
            "required_fields": [],
            "projected_value_sets": [
                { "field": "rows", "member_field": "class", "exact_values": ["A", "B", "C"] }
            ]
        });
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{ "class": "A" }, { "class": "B" }, { "class": "C" }]);
        assert_eq!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec)).verdict,
            Verdict::Green
        );
        // red: a copy-pasted row duplicated B and dropped C.
        let mut spec = valid_slice_spec();
        spec["rows"] = json!([{ "class": "A" }, { "class": "B" }, { "class": "B" }]);
        assert!(
            evaluate_configured(&policy_with(slice.clone()), &corpus_with(spec))
                .violations
                .contains("contract_slice_projected_set_missing")
        );
        // red: an unexpected extra class.
        let mut spec = valid_slice_spec();
        spec["rows"] =
            json!([{ "class": "A" }, { "class": "B" }, { "class": "C" }, { "class": "D" }]);
        assert!(
            evaluate_configured(&policy_with(slice), &corpus_with(spec))
                .violations
                .contains("contract_slice_projected_set_unexpected")
        );
    }
}
