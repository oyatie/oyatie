---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-002-form-field-section-response-domain-kernel
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test, cargo-fmt, cargo-clippy, oya-governance-canonical-form-byte-equality, oya-governance-form-schema-conformance]
---

# IP-002: Form / Field / Section / Response domain kernel

## Intent

Author the typed domain kernel for forms in Rust: `Form`, `Field`, `Section`, `Page`, `Response`, `Submission`, `FormSpecV1`. Per ADR-FORMS-0001 form.v1 canonicalisation; per ADR-0131 per-microservice flat layout with `src/` as canonical code root. Kernel is pure (no I/O, no Tokio); adapters wrap.

## ChangeSet boundary

One ChangeSet: `microservices/forms/src/domain/{form, field, section, response, submission, version}.rs` + serde-rs canonical encoder + form.v1 JSON Schema validator wrapper + comprehensive unit tests.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/domain/form.rs` | create |
| `microservices/forms/src/domain/field.rs` | create |
| `microservices/forms/src/domain/section.rs` | create |
| `microservices/forms/src/domain/page.rs` | create |
| `microservices/forms/src/domain/response.rs` | create |
| `microservices/forms/src/domain/submission.rs` | create |
| `microservices/forms/src/domain/version.rs` | create |
| `microservices/forms/src/domain/canonical.rs` | create — RFC 8785 JCS encoder |
| `microservices/forms/src/domain/mod.rs` | create — re-exports |
| `microservices/forms/tests/domain_round_trip.rs` | create |

## Code Shape

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormSpecV1 {
    pub purpose: String,
    pub consent_notice: Option<String>,
    pub pages: Vec<Page>,
    pub branching: Vec<BranchPredicate>,
    pub cross_field_rules: Vec<CrossFieldRule>,
}

impl FormSpecV1 {
    /// RFC 8785 canonical bytes. Round-trip invariant: load(emit(x)) == x.
    pub fn to_canonical_bytes(&self) -> Vec<u8> { /* … */ }
    pub fn from_canonical_bytes(b: &[u8]) -> Result<Self, FormSpecError> { /* … */ }
    pub fn schema_hash(&self) -> ShaHash { sha256(&self.to_canonical_bytes()) }
    pub fn validate_x_data_class_declared(&self) -> Result<(), FormSpecError> { /* … */ }
    pub fn validate_x_i18n_label_present(&self) -> Result<(), FormSpecError> { /* … */ }
}
```

## Acceptance Gates

```bash
cargo test -p oya-forms-domain
cargo fmt -p oya-forms-domain -- --check
cargo clippy -p oya-forms-domain -- -D warnings
cargo run -p oya-dev-cli -- gate validate canonical-form-byte-equality --microservice forms
cargo run -p oya-dev-cli -- gate validate form-schema-conformance --microservice forms
```

## Test Plan

- Property tests: `proptest` over field-type / data-class combinations.
- Byte-equality: 1000-case `load(emit(x)) == x`.
- Schema-hash determinism: same FormSpecV1 → same SHA-256 across 100 process restarts.
- ≥ 90% line coverage; 100% on PII paths (data_class + i18n_label + consent_notice required-field paths).

## Halt Conditions

- Any round-trip byte-equality test fails.
- Coverage threshold not met.

## Next IP

[`IP-003-conditional-logic-engine-cel.md`](IP-003-conditional-logic-engine-cel.md)

## References

- ADR-FORMS-0001 form-definition schema.
- ADR-0131 per-microservice flat layout.
- RFC 8785 JCS.
- JSON Schema draft 2020-12.
