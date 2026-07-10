//! # cloud-ci-contract-slice-conformance
//!
//! Paved-road Rust/Buck2 gate that replaces the fleet-wide
//! `scripts/tests/*_check.py` "contract slice" validators with a single owned,
//! declarative, owned-Rust gate.
//!
//! A worker declares a slice as one entry in `contract-slice-policy.json`
//! (the committed spec path, its required fields, enum constraints, forbidden
//! content markers, and — for a migration — the retired Python source) and
//! ships the slice's committed spec JSON. No new Python, no shell, no CLI, no
//! new crate: the gate reads the declared slices and validates the live
//! committed specs.
//!
//! The surface is API/config shaped: callers pass the policy plus the typed
//! JSON corpus to [`evaluate_configured`]. The gate is pure — it never shells
//! out, spawns an interpreter, mutates files, or reads ambient repository
//! state. Repository-specific paths and per-slice rules live in
//! `contract-slice-policy.json`.
//!
//! Mirrors the `resource-contract-conformance` gate (ADR-0515 WS-D pure gate
//! shape; the `source_migration_slice` Python→Rust retirement pattern).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

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
        "enum_constraints",
        "required_array_members",
        "exact_array_fields",
        "required_object_array_members",
        "forbidden_markers",
        "required_markers",
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
    let skip_universal_markers =
        slice.get("skip_universal_markers").and_then(Value::as_bool) == Some(true);
    if !skip_universal_markers {
        for marker in FORBIDDEN_SPEC_MARKERS {
            if recursively_contains(spec, marker) {
                findings.insert(Finding::new(
                    "contract_slice_forbidden_marker",
                    format!("{slice_id}:{marker}"),
                ));
            }
        }
    }
    for marker in string_array(slice, "forbidden_markers") {
        if recursively_contains(spec, &marker) {
            findings.insert(Finding::new(
                "contract_slice_forbidden_marker",
                format!("{slice_id}:{marker}"),
            ));
        }
    }

    // 1b. Required content markers: the inverse of forbidden_markers, scoped to a
    //     dotted field. Proves a conditional sub-tree (e.g. a JSON Schema `allOf`
    //     tier-conditional block) actually names the values it claims to gate on.
    //     Scoping to `field` (rather than the whole spec) matters: an unscoped
    //     search would trivially pass on a marker that merely appears elsewhere
    //     in the document (e.g. a tier name already required by an enum
    //     constraint), silently disarming the check it exists to make.
    if let Some(requirements) = slice.get("required_markers").and_then(Value::as_array) {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "markers"],
                slice_id,
                "required_markers",
                findings,
            );
            let target_field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let scope = get_dotted(spec, target_field);
            for marker in requirement
                .get("markers")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let found = scope.is_some_and(|value| recursively_contains(value, marker));
                if !found {
                    findings.insert(Finding::new(
                        "contract_slice_required_marker_missing",
                        format!("{slice_id}:{target_field}:{marker}"),
                    ));
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

    // 3. Enum constraints: the dotted field's scalar value must be allowed.
    //    `actual` is canonicalized via `scalar_str` so a JSON number/bool leaf
    //    (e.g. a pinned `14.4` threshold or `90`-day lifetime) compares equal to
    //    the policy's string-authored `allowed` literal, not just string leaves.
    if let Some(constraints) = slice.get("enum_constraints").and_then(Value::as_array) {
        for constraint in constraints {
            check_keys(
                constraint,
                &["field", "allowed"],
                slice_id,
                "enum_constraints",
                findings,
            );
            let field = constraint
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let allowed: Vec<&str> = constraint
                .get("allowed")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let actual = get_dotted(spec, field).and_then(scalar_str);
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
    //    in Rust — they stay data in the policy. Array elements are canonicalized
    //    via `scalar_str` so a JSON number array (e.g. pinned rollout-stage
    //    percentages) is checked the same as a string array.
    if let Some(requirements) = slice
        .get("required_array_members")
        .and_then(Value::as_array)
    {
        for requirement in requirements {
            check_keys(
                requirement,
                &["field", "members"],
                slice_id,
                "required_array_members",
                findings,
            );
            let field = requirement
                .get("field")
                .and_then(Value::as_str)
                .unwrap_or("");
            let present: Vec<String> = get_dotted(spec, field)
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(scalar_str).collect())
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
    if let Some(requirements) = slice.get("exact_array_fields").and_then(Value::as_array) {
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
            let expected: Vec<&str> = requirement
                .get("values")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let actual: Option<Vec<&str>> = get_dotted(spec, field)
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect());
            if actual.as_deref() != Some(expected.as_slice()) {
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

    // 4c. Required object-array members: an array-of-objects field must contain an
    //     object per declared member (matched on `member_key`, default "id"), each
    //     object carrying `member_required_fields` and satisfying
    //     `member_enum_constraints`. Expresses "the six-input promotion gate must
    //     enumerate exactly these inputs, each fail-closed" as policy DATA.
    if let Some(requirements) = slice
        .get("required_object_array_members")
        .and_then(Value::as_array)
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
                    let if_field = rule.get("if_field").and_then(Value::as_str).unwrap_or("");
                    if get_dotted(object, if_field).and_then(Value::as_bool) != Some(true) {
                        continue;
                    }
                    for then_field in rule
                        .get("then_required_fields")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
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

    let applies = match (
        assertion.get("when_member").and_then(Value::as_str),
        assertion.get("when_member_in").and_then(Value::as_array),
    ) {
        (Some(one), _) => member_id == one,
        (None, Some(many)) => many
            .iter()
            .filter_map(Value::as_str)
            .any(|m| m == member_id),
        (None, None) => true,
    };
    if !applies {
        return;
    }

    let target_field = assertion.get("field").and_then(Value::as_str).unwrap_or("");
    let key = format!("{slice_id}:{field}:{member_id}:{target_field}");

    // Dotted (not a flat `object.get`) so a rule can pin a nested member field
    // (e.g. `capability_overrides.enforcement.lane_id`), not only a top-level one.
    if let Some(expected) = assertion.get("must_equal").and_then(Value::as_str) {
        if get_dotted(object, target_field).and_then(Value::as_str) != Some(expected) {
            findings.insert(Finding::new(
                "contract_slice_conditional_field_not_equal",
                key,
            ));
        }
        return;
    }
    if assertion.get("must_be_true").and_then(Value::as_bool) == Some(true) {
        if get_dotted(object, target_field).and_then(Value::as_bool) != Some(true) {
            findings.insert(Finding::new(
                "contract_slice_conditional_field_not_true",
                key,
            ));
        }
        return;
    }
    if let Some(members) = assertion.get("must_contain").and_then(Value::as_array) {
        let present: BTreeSet<&str> = get_dotted(object, target_field)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
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
    if let Some(allowed) = assertion.get("must_subset_of").and_then(Value::as_array) {
        let allowed: BTreeSet<&str> = allowed.iter().filter_map(Value::as_str).collect();
        let actual: Vec<&str> = get_dotted(object, target_field)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        for value in actual {
            if !allowed.contains(value) {
                findings.insert(Finding::new(
                    "contract_slice_conditional_field_not_subset",
                    format!("{key}:{value}"),
                ));
            }
        }
        return;
    }
    findings.insert(Finding::new(
        "contract_slice_conditional_assertion_no_mode",
        key,
    ));
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
                { "field": "threshold", "allowed": ["14.4"] },
                { "field": "enabled", "allowed": ["true"] }
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
                { "field": "canary_stages_percent", "members": ["1", "10", "50", "100"] }
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
}
