---
doc_class: PipelineSpec
shape: pipeline
length_cap: 150
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Every kernel struct field carries an `oyatie.data_class` doc-comment annotation
  (e.g. `// data_class: INTERNAL_ONLY`). This pipeline parses those annotations
  plus the field's `///` doc-comment and emits the canonical data-class
  catalogue doc. The doc is the system-of-record for which fields hold which
  privacy class.
planned_enforcement_ref: oya-governance-data-class
extends_crates:
  - oya-governance-data-class-fitness-kernel
  - oya-governance-glossary-coverage-kernel
companion_docs:
  - INDEX.md
  - ../../docs/PRIVACY-PROGRAM.md
  - ../../docs/decisions/ADR-0709-general-live-apex.md
doc_status: published
---

# Pipeline: schema-doc / data-class catalogue auto-generation

> **ADRs:** ADR-0052, ADR-0053, ADR-0054.

## 1. Purpose

ADR-0008 (Data Use Boundary) defines five data classes: `PUBLIC`, `INTERNAL_ONLY`, `CUSTOMER_CONFIDENTIAL`, `PII_CUSTOMER`, `PII_REGULATED`. Today fields are annotated in source (visible in `oya-governance-runbook-freshness-kernel/src/lib.rs`: `pub path: String, // data_class: INTERNAL_ONLY`). The existing `oya-governance-data-class-fitness-kernel` validates coverage. This pipeline emits the catalogue doc that humans + agents consult.

## 2. Inputs

- Every `crates/**/src/**/*.rs` field with a trailing `// data_class: <CLASS>` comment (the field-level annotation pattern already in production).
- Every `crates/**/src/**/*.rs` struct preceded by `/// data_class_default: <CLASS>` doc-comment (the struct-level default).
- The field-level `///` doc-comment (purpose + privacy notes).

## 3. Outputs

- `docs/data-class-catalogue.md` — single-file catalogue grouped by data class, then by crate, then by struct, then by field. Each row: `crate :: struct :: field | type | data_class | description (from doc-comment)`.
- `docs/machine-readable/data-classes.json` — same data, machine-readable, consumed by `oya-governance-privacy-class-taxonomy-coverage`.
- Per-data-class mdbook chapter `docs/site/src/data/<data-class>.md`.

## 4. Annotation grammar (the source-of-truth syntax)

```rust
/// data_class_default: INTERNAL_ONLY
/// purpose: Per-tenant audit-chain event with full event-source provenance.
pub struct AuditEvent {
    /// Stable tenant identifier; never logged outside audit chain.
    pub tenant_id: String,       // data_class: PII_CUSTOMER
    /// Wall-clock event time at emission.
    pub emitted_at: DateTime,    // data_class: INTERNAL_ONLY
    /// Free-text actor description (may include human names).
    pub actor: String,           // data_class: PII_REGULATED
}
```

Allowed classes: `PUBLIC | INTERNAL_ONLY | CUSTOMER_CONFIDENTIAL | PII_CUSTOMER | PII_REGULATED`. Unknown class → BLOCKER from `oya-governance-data-class`.

## 5. Trigger matrix

| Event | Action |
|---|---|
| Per-PR touching any `pub` field in `crates/**` | Regenerate catalogue; diff posted to PR; PR fails if catalogue not regenerated. |
| Nightly | Full sweep; cross-link verification (every class referenced exists in ADR-0008 taxonomy). |
| On `docs/adr-archive/ADR-0008-data-use-boundary.md change | Re-validate every annotation against the new taxonomy. |

## 6. Validation gates (extending `oya-governance-data-class`)

1. **Field coverage.** Every `pub` field in every kernel crate has a data-class annotation OR inherits one via `data_class_default` (BLOCKER on omission).
2. **Class validity.** Annotation value ∈ the five allowed classes (BLOCKER).
3. **Privacy escalation rule.** A field re-classified from a more-restrictive class to a less-restrictive one (e.g. `PII_CUSTOMER → INTERNAL_ONLY`) requires an ADR-citation in the same PR (BLOCKER).
4. **Doc-comment presence.** Every annotated field has a non-empty `///` doc-comment (HIGH).
5. **Catalogue drift.** Generated catalogue differs from committed `docs/data-class-catalogue.md` (BLOCKER).

## 7. Glossary integration

Every data-class label is a glossary term; `oya-governance-glossary-coverage-kernel` cross-validates that `docs/GLOSSARY.md` defines each class with link back to ADR-0008. See also `glossary-pipeline.md`.

## 8. Out-of-scope

- Runtime PII redaction enforcement (covered by `oya-intelligence-policy-kernel` Cedar layer).
- Egress filtering (covered by the data-use-boundary runtime; ADR-0008 §Implementation).
- Synthetic-data class for test fixtures (separate `synthetic-data-pipeline.md`, not in this batch).
