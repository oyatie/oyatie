---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-004-validation-engine
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test, oya-forms-field-validate-latency, oya-forms-cross-field-validation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: Validation engine (per-field + cross-field)

## Intent

Implement field validation engine: per-field via JSON Schema draft 2020-12 subset declared in form.v1; cross-field via CEL rules per ADR-FORMS-0004. Validation runs client-side for UX + server-side for authority; results must agree.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/domain/validation/field.rs` | create |
| `microservices/forms/src/domain/validation/cross_field.rs` | create |
| `microservices/forms/src/domain/validation/diagnostic.rs` | create — JSON-pointer-precise diagnostics |
| `microservices/forms/tests/validation_field.rs` | create |
| `microservices/forms/tests/validation_cross.rs` | create |

## Code Shape

```rust
pub fn validate_submission(
    spec: &FormSpecV1,
    submission: &FieldValues,
) -> Result<(), Vec<ValidationDiagnostic>> { /* … */ }

pub struct ValidationDiagnostic {
    pub json_pointer: String,  // /fields/email
    pub code: ValidationCode,
    pub message_i18n: HashMap<Locale, String>,
}
```

## Acceptance Gates

- Per-field validation p99 ≤ 50ms over 10k-call benchmark (PRD performance).
- Cross-field rules evaluate identically client + server.
- ValidationDiagnostic always carries precise JSON pointer (AC-04).

## References

- ADR-FORMS-0001 + ADR-FORMS-0004.
- JSON Schema draft 2020-12.
- IETF RFC 6901 (JSON Pointer).

## Next IP

[`IP-005-versioning-and-changeset-binding.md`](IP-005-versioning-and-changeset-binding.md)
