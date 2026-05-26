//! M02-P02-IP-005 — Upstream-API drift detection kernel.
//!
//! Pure-Rust kernel that diffs two contract fingerprints (current adapter
//! contract vs upstream reference) and classifies findings as BREAKING,
//! NON_BREAKING, or ADDITIVE. The workspace constraint forbids OpenAPI YAML
//! scaffolds (validators reject incomplete schemas), so the kernel diffs
//! JSON contract fingerprints: deterministic JSON summaries of the route
//! surface (operation id, method, path, request-field schema, response-field
//! schema, response status set).
//!
//! Linus good-taste: providers are rows in a static `UpstreamRegistry` table;
//! `detect_drift` is one function that iterates the table. Adding Gemini,
//! Mistral, Cohere, or a future provider is a row addition, not a code
//! addition.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_intelligence_account_kernel::ProviderFamily;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DriftSeverity {
    Breaking,
    NonBreaking,
    Additive,
}

impl DriftSeverity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Breaking => "breaking",
            Self::NonBreaking => "non_breaking",
            Self::Additive => "additive",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriftEntry {
    // data_class: INTERNAL_ONLY
    pub operation_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub kind: DriftKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub severity: DriftSeverity, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriftKind {
    OperationRemoved,
    OperationAdded,
    MethodChanged {
        from: String,
        to: String,
    },
    PathChanged {
        from: String,
        to: String,
    },
    RequestFieldRemoved(String),
    RequestFieldAdded(String),
    RequestFieldTypeChanged {
        field: String,
        from: String,
        to: String,
    },
    ResponseFieldRemoved(String),
    ResponseFieldAdded(String),
    ResponseFieldTypeChanged {
        field: String,
        from: String,
        to: String,
    },
    EnumValueRemoved {
        field: String,
        value: String,
    },
    EnumValueAdded {
        field: String,
        value: String,
    },
    StatusCodeRemoved(u16),
    StatusCodeAdded(u16),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriftReport {
    // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub contract_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub entries: Vec<DriftEntry>, // data_class: INTERNAL_ONLY
}

impl DriftReport {
    pub fn highest_severity(&self) -> Option<DriftSeverity> {
        self.entries.iter().map(|e| e.severity).min()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Convenience predicate — true if any entry is BREAKING.
    pub fn has_breaking(&self) -> bool {
        self.entries
            .iter()
            .any(|e| e.severity == DriftSeverity::Breaking)
    }
}

/// JSON contract fingerprint — deterministic operation set + per-operation
/// field/status surface. Pure value type with no JSON dependency (kernel layer
/// stays std-only; adapters can deserialize from JSON via `serde_json` in
/// outer rings).
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ContractFingerprint {
    // data_class: INTERNAL_ONLY
    pub provider: Option<ProviderFamily>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub contract_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub operations: BTreeMap<String, OperationFingerprint>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct OperationFingerprint {
    // data_class: INTERNAL_ONLY
    pub method: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub path: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub request_fields: BTreeMap<String, FieldDescriptor>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub response_fields: BTreeMap<String, FieldDescriptor>, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub status_codes: BTreeSet<u16>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct FieldDescriptor {
    /// Logical type — `string`, `integer`, `array<string>`, `enum<a|b|c>`, etc.
    // data_class: INTERNAL_ONLY
    pub type_marker: String, // data_class: INTERNAL_ONLY
    /// Enum values when `type_marker` starts with `enum<…>`. Parallel to
    /// `type_marker` so the diff can detect added/removed enum values without
    /// re-parsing.
    // data_class: INTERNAL_ONLY
    pub enum_values: BTreeSet<String>, // data_class: INTERNAL_ONLY
}

/// Provider × adapter-contract row. Adding a provider = adding a row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpstreamSpec {
    // data_class: INTERNAL_ONLY
    pub provider: ProviderFamily, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub canonical_source_url: &'static str, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub pinned_version: &'static str, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub adapter_contract_id: &'static str, // data_class: INTERNAL_ONLY
}

pub fn upstream_registry() -> Vec<UpstreamSpec> {
    vec![
        UpstreamSpec {
            provider: ProviderFamily::Claude,
            canonical_source_url: "https://github.com/anthropics/anthropic-sdk-python/blob/main/openapi.json",
            pinned_version: "2023-06-01",
            adapter_contract_id: "foundry-compat-anthropic-v1",
        },
        UpstreamSpec {
            provider: ProviderFamily::OpenAiOrCodex,
            canonical_source_url: "https://github.com/openai/openai-openapi/blob/main/openapi.yaml",
            pinned_version: "2024-08-06",
            adapter_contract_id: "foundry-compat-openai-v1",
        },
        UpstreamSpec {
            provider: ProviderFamily::Gemini,
            canonical_source_url: "https://ai.google.dev/api/rest/v1beta",
            pinned_version: "v1beta",
            adapter_contract_id: "foundry-compat-gemini-v1",
        },
    ]
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriftError {
    UnknownProvider,
    ContractIdMismatch { left: String, right: String },
}

impl fmt::Display for DriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownProvider => write!(f, "unknown provider"),
            Self::ContractIdMismatch { left, right } => {
                write!(f, "contract_id mismatch: {left} != {right}")
            }
        }
    }
}

/// Pure decision function. Diffs `current` (the locally pinned adapter
/// contract fingerprint) against `upstream` (the freshly fetched canonical
/// fingerprint), producing a `DriftReport`.
pub fn detect_drift(
    spec: &UpstreamSpec,
    current: &ContractFingerprint,
    upstream: &ContractFingerprint,
) -> Result<DriftReport, DriftError> {
    if current.contract_id != upstream.contract_id {
        return Err(DriftError::ContractIdMismatch {
            left: current.contract_id.clone(),
            right: upstream.contract_id.clone(),
        });
    }
    let mut entries = Vec::new();
    let cur_ops: BTreeSet<&String> = current.operations.keys().collect();
    let up_ops: BTreeSet<&String> = upstream.operations.keys().collect();

    // Removed operations (upstream no longer publishes a route we depend on).
    for op in cur_ops.difference(&up_ops) {
        entries.push(DriftEntry {
            operation_id: (*op).clone(),
            kind: DriftKind::OperationRemoved,
            severity: DriftSeverity::Breaking,
        });
    }
    // Added operations (upstream added a route — additive for our adapter).
    for op in up_ops.difference(&cur_ops) {
        entries.push(DriftEntry {
            operation_id: (*op).clone(),
            kind: DriftKind::OperationAdded,
            severity: DriftSeverity::Additive,
        });
    }
    // Intersect: per-operation diff
    for op in cur_ops.intersection(&up_ops) {
        let cur_op = &current.operations[*op];
        let up_op = &upstream.operations[*op];
        if cur_op.method != up_op.method {
            entries.push(DriftEntry {
                operation_id: (*op).clone(),
                kind: DriftKind::MethodChanged {
                    from: cur_op.method.clone(),
                    to: up_op.method.clone(),
                },
                severity: DriftSeverity::Breaking,
            });
        }
        if cur_op.path != up_op.path {
            entries.push(DriftEntry {
                operation_id: (*op).clone(),
                kind: DriftKind::PathChanged {
                    from: cur_op.path.clone(),
                    to: up_op.path.clone(),
                },
                severity: DriftSeverity::Breaking,
            });
        }
        // Request fields
        diff_fields(
            op,
            &cur_op.request_fields,
            &up_op.request_fields,
            FieldSide::Request,
            &mut entries,
        );
        // Response fields
        diff_fields(
            op,
            &cur_op.response_fields,
            &up_op.response_fields,
            FieldSide::Response,
            &mut entries,
        );
        // Status codes
        for st in cur_op.status_codes.difference(&up_op.status_codes) {
            entries.push(DriftEntry {
                operation_id: (*op).clone(),
                kind: DriftKind::StatusCodeRemoved(*st),
                severity: DriftSeverity::Breaking,
            });
        }
        for st in up_op.status_codes.difference(&cur_op.status_codes) {
            entries.push(DriftEntry {
                operation_id: (*op).clone(),
                kind: DriftKind::StatusCodeAdded(*st),
                severity: DriftSeverity::Additive,
            });
        }
    }

    Ok(DriftReport {
        provider: spec.provider,
        contract_id: spec.adapter_contract_id.to_owned(),
        entries,
    })
}

#[derive(Clone, Copy)]
enum FieldSide {
    Request,
    Response,
}

fn diff_fields(
    op: &str,
    cur: &BTreeMap<String, FieldDescriptor>,
    up: &BTreeMap<String, FieldDescriptor>,
    side: FieldSide,
    entries: &mut Vec<DriftEntry>,
) {
    let cur_set: BTreeSet<&String> = cur.keys().collect();
    let up_set: BTreeSet<&String> = up.keys().collect();
    // Removed:
    //   - request-side: upstream removed an input we send → BREAKING (server will reject extras eventually).
    //     We currently classify as NON_BREAKING (server is more permissive) for request-removal because
    //     a server that no longer expects a field will typically ignore it. But for our adapter, that means
    //     we still produce a valid-but-ignored field. Treat as NonBreaking.
    //   - response-side: BREAKING (we read this field; upstream stopped emitting it).
    for f in cur_set.difference(&up_set) {
        let kind = match side {
            FieldSide::Request => DriftKind::RequestFieldRemoved((*f).clone()),
            FieldSide::Response => DriftKind::ResponseFieldRemoved((*f).clone()),
        };
        let severity = match side {
            FieldSide::Request => DriftSeverity::NonBreaking,
            FieldSide::Response => DriftSeverity::Breaking,
        };
        entries.push(DriftEntry {
            operation_id: op.to_owned(),
            kind,
            severity,
        });
    }
    // Added:
    //   - request-side: upstream wants a new field → could be required (BREAKING) or optional (ADDITIVE).
    //     Without explicit required/optional markers in the fingerprint, default to ADDITIVE.
    //   - response-side: upstream emits a new field → ADDITIVE (we just don't read it).
    for f in up_set.difference(&cur_set) {
        let kind = match side {
            FieldSide::Request => DriftKind::RequestFieldAdded((*f).clone()),
            FieldSide::Response => DriftKind::ResponseFieldAdded((*f).clone()),
        };
        entries.push(DriftEntry {
            operation_id: op.to_owned(),
            kind,
            severity: DriftSeverity::Additive,
        });
    }
    // Intersect:
    for f in cur_set.intersection(&up_set) {
        let c = &cur[*f];
        let u = &up[*f];
        if c.type_marker != u.type_marker {
            let kind = match side {
                FieldSide::Request => DriftKind::RequestFieldTypeChanged {
                    field: (*f).clone(),
                    from: c.type_marker.clone(),
                    to: u.type_marker.clone(),
                },
                FieldSide::Response => DriftKind::ResponseFieldTypeChanged {
                    field: (*f).clone(),
                    from: c.type_marker.clone(),
                    to: u.type_marker.clone(),
                },
            };
            entries.push(DriftEntry {
                operation_id: op.to_owned(),
                kind,
                severity: DriftSeverity::Breaking,
            });
        }
        // Enum-value diffs (renamed value is removed+added).
        for v in c.enum_values.difference(&u.enum_values) {
            entries.push(DriftEntry {
                operation_id: op.to_owned(),
                kind: DriftKind::EnumValueRemoved {
                    field: (*f).clone(),
                    value: v.clone(),
                },
                severity: DriftSeverity::Breaking,
            });
        }
        for v in u.enum_values.difference(&c.enum_values) {
            entries.push(DriftEntry {
                operation_id: op.to_owned(),
                kind: DriftKind::EnumValueAdded {
                    field: (*f).clone(),
                    value: v.clone(),
                },
                severity: DriftSeverity::Additive,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(method: &str, path: &str) -> OperationFingerprint {
        OperationFingerprint {
            method: method.to_owned(),
            path: path.to_owned(),
            request_fields: BTreeMap::new(),
            response_fields: BTreeMap::new(),
            status_codes: [200].into_iter().collect(),
        }
    }

    fn fingerprint() -> ContractFingerprint {
        let mut ops = BTreeMap::new();
        ops.insert("messagesCreate".into(), op("POST", "/v1/messages"));
        ContractFingerprint {
            provider: Some(ProviderFamily::Claude),
            contract_id: "foundry-compat-anthropic-v1".into(),
            operations: ops,
        }
    }

    fn spec() -> UpstreamSpec {
        upstream_registry()
            .into_iter()
            .find(|s| s.provider == ProviderFamily::Claude)
            .unwrap()
    }

    #[test]
    fn identical_fingerprints_yield_empty_report() {
        let f = fingerprint();
        let r = detect_drift(&spec(), &f, &f).unwrap();
        assert!(r.is_empty());
        assert!(!r.has_breaking());
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn operation_removed_is_breaking() {
        let cur = fingerprint();
        let mut up = fingerprint();
        up.operations.clear();
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert_eq!(r.count(), 1);
        assert!(r.has_breaking());
        assert!(matches!(r.entries[0].kind, DriftKind::OperationRemoved));
    }

    #[test]
    fn operation_added_is_additive() {
        let cur = fingerprint();
        let mut up = fingerprint();
        up.operations
            .insert("countTokens".into(), op("GET", "/v1/messages/count_tokens"));
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert_eq!(r.count(), 1);
        assert!(!r.has_breaking());
        assert_eq!(r.entries[0].severity, DriftSeverity::Additive);
    }

    #[test]
    fn method_change_is_breaking() {
        let cur = fingerprint();
        let mut up = fingerprint();
        up.operations
            .insert("messagesCreate".into(), op("PUT", "/v1/messages"));
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(r.has_breaking());
        let kinds: Vec<_> = r.entries.iter().map(|e| &e.kind).collect();
        assert!(
            kinds
                .iter()
                .any(|k| matches!(k, DriftKind::MethodChanged { .. }))
        );
    }

    #[test]
    fn path_change_is_breaking() {
        let cur = fingerprint();
        let mut up = fingerprint();
        up.operations
            .insert("messagesCreate".into(), op("POST", "/v2/messages"));
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(r.has_breaking());
    }

    #[test]
    fn response_field_removed_is_breaking() {
        let mut cur = fingerprint();
        let mut op_ = op("POST", "/v1/messages");
        op_.response_fields.insert(
            "stop_reason".into(),
            FieldDescriptor {
                type_marker: "string".into(),
                enum_values: Default::default(),
            },
        );
        cur.operations.insert("messagesCreate".into(), op_);
        let mut up = fingerprint();
        up.operations
            .insert("messagesCreate".into(), op("POST", "/v1/messages"));
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(r.has_breaking());
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::ResponseFieldRemoved(_)))
        );
    }

    #[test]
    fn response_field_added_is_additive() {
        let cur = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op.response_fields.insert(
            "container".into(),
            FieldDescriptor {
                type_marker: "object".into(),
                enum_values: Default::default(),
            },
        );
        let mut up = fingerprint();
        up.operations.insert("messagesCreate".into(), up_op);
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(!r.has_breaking());
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::ResponseFieldAdded(_)))
        );
    }

    #[test]
    fn request_field_removed_is_non_breaking() {
        let mut cur = fingerprint();
        let mut op_ = op("POST", "/v1/messages");
        op_.request_fields
            .insert("metadata".into(), FieldDescriptor::default());
        cur.operations.insert("messagesCreate".into(), op_);
        let up = fingerprint();
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        let removed = r
            .entries
            .iter()
            .find(|e| matches!(e.kind, DriftKind::RequestFieldRemoved(_)))
            .unwrap();
        assert_eq!(removed.severity, DriftSeverity::NonBreaking);
    }

    #[test]
    fn request_field_added_is_additive() {
        let cur = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op
            .request_fields
            .insert("metadata".into(), FieldDescriptor::default());
        let mut up = fingerprint();
        up.operations.insert("messagesCreate".into(), up_op);
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::RequestFieldAdded(_)))
        );
        assert!(!r.has_breaking());
    }

    #[test]
    fn type_change_is_breaking() {
        let mut cur = fingerprint();
        let mut op_ = op("POST", "/v1/messages");
        op_.request_fields.insert(
            "max_tokens".into(),
            FieldDescriptor {
                type_marker: "integer".into(),
                enum_values: Default::default(),
            },
        );
        cur.operations.insert("messagesCreate".into(), op_);

        let mut up = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op.request_fields.insert(
            "max_tokens".into(),
            FieldDescriptor {
                type_marker: "string".into(),
                enum_values: Default::default(),
            },
        );
        up.operations.insert("messagesCreate".into(), up_op);

        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(r.has_breaking());
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::RequestFieldTypeChanged { .. }))
        );
    }

    #[test]
    fn enum_value_removed_is_breaking() {
        let mut cur = fingerprint();
        let mut op_ = op("POST", "/v1/messages");
        op_.response_fields.insert(
            "stop_reason".into(),
            FieldDescriptor {
                type_marker: "enum<end_turn|tool_use>".into(),
                enum_values: ["end_turn", "tool_use"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
        );
        cur.operations.insert("messagesCreate".into(), op_);

        let mut up = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op.response_fields.insert(
            "stop_reason".into(),
            FieldDescriptor {
                type_marker: "enum<end_turn|tool_use>".into(),
                enum_values: ["end_turn"].iter().map(|s| s.to_string()).collect(),
            },
        );
        up.operations.insert("messagesCreate".into(), up_op);

        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(r.has_breaking());
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::EnumValueRemoved { .. }))
        );
    }

    #[test]
    fn enum_value_added_is_additive() {
        let mut cur = fingerprint();
        let mut op_ = op("POST", "/v1/messages");
        op_.response_fields.insert(
            "stop_reason".into(),
            FieldDescriptor {
                type_marker: "enum".into(),
                enum_values: ["end_turn"].iter().map(|s| s.to_string()).collect(),
            },
        );
        cur.operations.insert("messagesCreate".into(), op_);

        let mut up = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op.response_fields.insert(
            "stop_reason".into(),
            FieldDescriptor {
                type_marker: "enum".into(),
                enum_values: ["end_turn", "refusal"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            },
        );
        up.operations.insert("messagesCreate".into(), up_op);

        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::EnumValueAdded { .. }))
        );
    }

    #[test]
    fn status_code_removed_is_breaking() {
        let cur = fingerprint();
        let mut up = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op.status_codes = BTreeSet::new(); // dropped 200
        up.operations.insert("messagesCreate".into(), up_op);
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(r.has_breaking());
    }

    #[test]
    fn status_code_added_is_additive() {
        let cur = fingerprint();
        let mut up = fingerprint();
        let mut up_op = op("POST", "/v1/messages");
        up_op.status_codes = [200, 429].into_iter().collect();
        up.operations.insert("messagesCreate".into(), up_op);
        let r = detect_drift(&spec(), &cur, &up).unwrap();
        assert!(!r.has_breaking());
        assert!(
            r.entries
                .iter()
                .any(|e| matches!(e.kind, DriftKind::StatusCodeAdded(429)))
        );
    }

    #[test]
    fn contract_id_mismatch_errors() {
        let cur = fingerprint();
        let mut up = fingerprint();
        up.contract_id = "different-id".into();
        let r = detect_drift(&spec(), &cur, &up);
        assert!(matches!(r, Err(DriftError::ContractIdMismatch { .. })));
    }

    #[test]
    fn registry_holds_three_initial_providers() {
        let r = upstream_registry();
        assert_eq!(r.len(), 3);
        let providers: BTreeSet<_> = r.iter().map(|s| s.provider as u8).collect();
        assert_eq!(providers.len(), 3);
    }

    #[test]
    fn drift_severity_names_distinct() {
        let s: std::collections::HashSet<&str> = [
            DriftSeverity::Breaking,
            DriftSeverity::NonBreaking,
            DriftSeverity::Additive,
        ]
        .iter()
        .map(|d| d.name())
        .collect();
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn drift_error_display_distinct() {
        let m: Vec<String> = vec![
            format!("{}", DriftError::UnknownProvider),
            format!(
                "{}",
                DriftError::ContractIdMismatch {
                    left: "a".into(),
                    right: "b".into(),
                }
            ),
        ];
        let uniq: std::collections::HashSet<_> = m.iter().collect();
        assert_eq!(uniq.len(), m.len());
    }

    #[test]
    fn highest_severity_is_lowest_enum_value() {
        // DriftSeverity ordered: Breaking < NonBreaking < Additive (Ord derived
        // via declaration order). highest_severity returns min — i.e., the
        // *most severe* finding.
        let r = DriftReport {
            provider: ProviderFamily::Claude,
            contract_id: "x".into(),
            entries: vec![
                DriftEntry {
                    operation_id: "op".into(),
                    kind: DriftKind::OperationRemoved,
                    severity: DriftSeverity::Breaking,
                },
                DriftEntry {
                    operation_id: "op".into(),
                    kind: DriftKind::OperationAdded,
                    severity: DriftSeverity::Additive,
                },
            ],
        };
        assert_eq!(r.highest_severity(), Some(DriftSeverity::Breaking));
    }

    #[test]
    fn upstream_spec_for_each_registry_provider_has_contract_id() {
        for s in upstream_registry() {
            assert!(!s.adapter_contract_id.is_empty());
            assert!(!s.canonical_source_url.is_empty());
            assert!(!s.pinned_version.is_empty());
        }
    }
}
