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
- PRD FR-03 and AC-04.
- `microservices/forms/contracts/openapi/forms.openapi.yaml` request/response validation payloads.
- `microservices/forms/contracts/proto/forms.proto` internal validation result messages.
- `microservices/forms/slos/field-validate-latency.openslo.yaml`.
- `microservices/forms/runbooks/response-store-corruption.md` for bad accepted-response remediation.
- `microservices/forms/decisions/ADR-FORMS-0001-form-definition-schema.md`.

## Foundation A-G Substance

- A. Product scope: validation is the authority boundary for every response, import, pre-fill, and workflow-triggered submit.
- B. Domain model: `ValidationRule`, `CrossFieldRule`, `ValidationDiagnostic`, and `ValidationDecisionTrace` live outside adapters.
- C. Contracts: diagnostics use JSON Pointer and stable codes so UI, SDK, and webhooks can render the same failure.
- D. Policy: high-risk and special-category fields require data-class-aware validation before any response-store write.
- E. Operations: invalid schema publication fails fast; invalid submission returns a typed 422 and sealed audit failure.
- F. Observability: track p50/p95/p99 validation latency, diagnostic-code cardinality, and client/server disagreement count.
- G. Promotion: server/client parity, OpenAPI examples, SLO budget, and WCAG diagnostic rendering all gate completion.

## Counterpart Benchmark

- Counterpart: HubSpot Forms field validation, Salesforce Web-to-Lead required-field enforcement, and ServiceNow catalog item variable validation.
- Defensible parity claim: Oyatie must match required, regex, range, email, date ordering, and cross-field constraints.
- Differentiator: the server is authoritative and every rejection has an audit-visible diagnostic code.
- Grep counterpart names: HubSpot Forms; Salesforce Web-to-Lead; ServiceNow catalog item forms.

## Remediation Notes

- Expanded validation beyond file targets into artifact-bound foundation criteria.
- Added A-G substance for domain, contracts, policy, runtime, telemetry, and promotion gates.
- Added counterpart names that are easy to grep during competitive and parity audits.

## Next IP

[`IP-005-versioning-and-changeset-binding.md`](IP-005-versioning-and-changeset-binding.md)
