---
purpose: "Cross-cutting data-class standard. Mandates `oyatie.data_class` annotations on every kernel struct field, codifies the cross-pillar flow rules (which classes may cross which axis boundaries)."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting data-class standard. Mandates `oyatie.data_class` annotations on
  every kernel struct field, codifies the cross-pillar flow rules (which classes
  may cross which axis boundaries), and binds DSR (data subject request) cascade
  hooks to each class. Implements `forbidden-operations.json` Item 5
  ("No new struct fields in kernel crates without `data_class`") and §Do Item 8
  (audit-chain emission on every cross-axis flow).
canonical_authority: docs/decisions/ADR-0008-data-use-boundary.md + docs/PRIVACY-PROGRAM.md + libs/oya-data-boundary-kernel/src/lib.rs
planned_enforcement_ref: oya-governance-data-class
enforcement_status:
  oya-governance-data-class: F-PENDING-DATA-CLASS (crate missing; tracked in registry/stub-audit/2026-05-17/missing-fitness-crates.json)
  oya-governance-dsr-cascade: F-PENDING-DSR-CASCADE (crate missing)
  oya-governance-audit-emission: existing
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
companion_docs:
  - docs/PRIVACY-PROGRAM.md
  - docs/standards/security-review.md
  - docs/standards/observability.md
  - docs/standards/autonomy-ceiling.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Data Class

## Doctrinal authority — ADR-0008 + Privacy Program + data-boundary kernel

Every kernel struct field MUST carry a `data_class` annotation. The annotation gates cross-pillar flow, audit-chain emission, DSR cascade behavior, and observability redaction. This standard binds field-level discipline to [`ADR-0008`](../decisions/ADR-0008-data-use-boundary.md), [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md), and [`libs/oya-data-boundary-kernel/src/lib.rs`](../../libs/oya-data-boundary-kernel/src/lib.rs).

Program policy still has exactly twelve privacy-program classes, an orthogonal `subject_class`, purpose-bound grants, four-pillar flows, and DSR/withdrawal cascade duties. The Rust kernel currently exposes thirteen privacy labels because the regulated-financial program family is split into `FINANCIAL` and `FINANCIAL_REGULATED_CREDIT` wire labels; that split is an implementation projection, not a new program class.

## 1. Current taxonomy and compatibility boundary

Authoritative privacy-program classes are: `INTERNAL_ONLY`, `PHI`, `PII_IDENTIFYING`, `PII_QUASI_IDENTIFIER`, `PCI`, the regulated-financial program family, `BEHAVIORAL_TENANT_PRODUCT`, `BEHAVIORAL_ADS`, `DECLARED_PREFERENCE`, `SEARCH_QUERY`, `PUBLIC`, and `SENSITIVE_PIPA_ART23`.

| Current Rust surface | Treatment | Notes |
|---|---|---|
| `PRIVACY_PROGRAM_DATA_CLASS_LABELS` | 13 executable privacy labels | Includes both `FINANCIAL` and `FINANCIAL_REGULATED_CREDIT` for the regulated-financial program family. |
| `PiiSensitive` / `PII_SENSITIVE` | compatibility alias | Maps to `PII_QUASI_IDENTIFIER`; not a separate privacy class. |
| `PipaArticle23` / `PIPA_ARTICLE_23` | compatibility alias | Maps to `SENSITIVE_PIPA_ART23`; not a separate privacy class. |
| `Usage` / `USAGE` | compatibility alias | Maps to `BEHAVIORAL_TENANT_PRODUCT`; not a separate privacy class. |
| `Audit` / `AUDIT` | operational marker | Use `OperationalDataClass` / `DataClassification`, not `PrivacyDataClass`. |
| `Secret` / `SECRET` | operational marker | Vault-only operational metadata; never public privacy taxonomy. |
| `Children` / `CHILDREN` | subject marker | Use `SubjectClass` / `SubjectDataMarker`; ADR-0008 rejected CHILDREN as a 13th privacy class. |

Historical lower-case labels (`public`, `internal`, `tenant-config`, `tenant-data`, `pii`, `phi`, `financial`, `secret`, and `regulated-jurisdiction`) are retired as implementation authority. They may appear only in legacy docs, import adapters, or coarse inventory prose that explicitly maps to the uppercase program/kernel vocabulary.

Lane: `oya-governance-data-class` checks every new kernel struct
field declaration for an annotation.

## 2. Annotation shape

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatientRecord {
    /// Stable subject identifier.
    #[oyatie(data_class = "PII_IDENTIFYING")]
    pub subject_id: SubjectId,

    /// Diagnostic finding.
    #[oyatie(data_class = "PHI", regulator = "MFDS", retention = "10y")]
    pub diagnosis: Diagnosis,

    /// Last-modified timestamp.
    #[oyatie(data_class = "INTERNAL_ONLY")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

The `oyatie` attribute is a derive macro (or attribute macro) provided by
`oya-kernel-data-class`. It emits compile-time metadata and runtime
descriptors consumed by:

- The audit-chain emission processor.
- The OTel redaction filter (per [`observability.md`](observability.md) §8).
- The DSR cascade walker (§5).
- The cross-pillar flow checker (§3).

## 3. Cross-pillar transition matrix

The seven axes (SaaS, Workspace, Vertical, Foundry, Cloud, Search,
Ads + Analytics) are pillars. Flows are governed by:

| From → To | `PUBLIC` | `INTERNAL_ONLY` | tenant/product behavioral classes | identifying/quasi PII | PHI/PCI/PIPA/regulated-financial | `AUDIT`/`SECRET` operational markers |
|---|---|---|---|---|---|---|
| SaaS → Workspace | ✓ | ✓ | same tenant + consent | gated | DENY except service/regulatory purpose | operational seam only; `SECRET` via vault |
| SaaS → Foundry | ✓ | ✓ + audit | tenant/user consent + audit | consent + audit | purpose-bound only; never ads | agent receives references, not vault secrets |
| Vertical → Vertical | ✓ | ✓ | DENY unless explicit cross-vertical grant | DENY | DENY | DENY |
| any → Search index | ✓ | sanitized | redacted/private-index only | DENY | DENY | operational metadata only |
| any → Ads/Analytics | aggregate only | aggregate only | tenant analytics / first-party ads only by purpose | DENY for ads | DENY / HARD DENY floors | never ad features |
| any → Cloud | ✓ | ✓ | encrypted-at-rest | KMS + residency | KMS + residency + retention policy | `SECRET` vault-only; `AUDIT` append-only |

"DENY" means: no plumbing, no opt-in, no flag. A flow that violates a row
fails the `oya-governance-cohesion` lane and the
`oya-governance-data-class` lane at the field-level.

Mathematical rule: the class of a derived value is the most restrictive
privacy-program class of its inputs (e.g., name + diagnosis inherits `PHI`).
Subject markers such as minor status remain orthogonal `subject_class`
inputs and compose with the data-class check.

## 4. Audit-chain emission requirements

Per [`observability.md`](observability.md) §4, every cross-pillar flow
of a non-public privacy-program class, operational marker, or residency /
jurisdiction marker MUST emit an `EVT-DATA-EGRESS` (or class-specific
event) with:

| Field | Required |
|---|---|
| `evt_id` | YES |
| `pillar_from` / `pillar_to` | YES |
| `data_class` | YES |
| `tenant_id` | YES (hashed if cross-tenant aggregation) |
| `actor_id` | YES |
| `capability_id` | when invoked via Foundry |
| `autonomy_tier` | when invoked via Foundry |
| `consent_token` | when consent-gated |
| `jurisdiction` | when residency / jurisdiction marker participates |
| `redacted_payload_hash` | YES (proves emission without storing the body) |

Lane: `oya-governance-audit-emission` validates the emission point
exists for every `data_class`-gated transition.

## 5. DSR cascade integration

Per [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md), every DSR (Data Subject Request — access / rectification / erasure / portability) walks **every** field in **every** axis whose `data_class` is a subject-owned privacy class (`PII_IDENTIFYING`, `PII_QUASI_IDENTIFIER`, `PHI`, `PCI`, `FINANCIAL`, `FINANCIAL_REGULATED_CREDIT`, `SENSITIVE_PIPA_ART23`, and purpose-specific behavioral/search classes when tied to a subject) and emits a proof-of-action receipt.

Mechanics:

1. The `oya-platform-dsr-kernel` walks the catalog of kernel structs and
   their `data_class` annotations.
2. For each (`subject_id`, `class`) pair, the kernel queries each pillar
   adapter implementing `DsrCapable`.
3. Each adapter returns a list of records + a recommended action (delete,
   redact, export).
4. The kernel persists the action plan, executes per regulatory deadline
   (e.g., 30 days for KR PIPC), and emits `EVT-DSR-EXECUTED` per record
   touched.
5. Proof-of-erasure is hash-chained and signed.

Lane: `oya-governance-dsr-cascade` validates that every subject-owned
privacy-program field has a registered `DsrCapable` adapter.

## 6. Schema migration discipline

Adding a new field to a kernel struct:

1. Declare the `data_class` in the same PR (compile-fail without it).
2. If the field is subject-owned, regulated, financial, or operationally secret-bearing, update [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) class taxonomy in the same PR.
3. If `PHI`, `SENSITIVE_PIPA_ART23`, regulated-financial, or a residency / jurisdiction marker participates, update [`docs/COMPLIANCE-MATRIX.md`](../COMPLIANCE-MATRIX.md) regulator mapping.
4. Provide DSR cascade adapter coverage (§5) before the field reaches
   production.
5. Run `oya-governance-schema-migration` (per AGENTS.md D14).

## 7. Field-naming conventions

- PII/PHI/regulated-financial fields SHOULD be wrapped in a newtype (`SubjectId`,
  `Diagnosis`) so the data class is visible at every call site.
- Boolean subject-status flags ("is_minor") MUST carry the correct
  privacy-program `data_class` for the source field plus a `subject_class`
  marker; child status is not a privacy class.
- Aggregated counts (k-anonymous, k ≥ 50) MAY be `INTERNAL_ONLY`; k < 50 keeps
  the underlying class.

## 8. Cross-tenant isolation

Per the fitness lane `cross-tenant-access-fuzz` (DOC-CATALOG.md §4),
deterministic isolation probes prove cross-tenant access fails closed.
This standard adds the `data_class` participation:

- Every kernel function that takes a `tenant_id` MUST carry a
  `#[tenant_scoped]` attribute that emits a tenant-mismatch check at
  the trait boundary.
- Cross-tenant aggregations (e.g., billing rollups) MUST go through
  `oya-platform-tenancy-aggregator` which strips tenant/product bodies,
  hashes `tenant_id`, and emits the aggregate via `INTERNAL_ONLY` class.

## 9. Observability redaction binding

The OTel redaction filter (per [`observability.md`](observability.md) §8)
reads the same `data_class` metadata at runtime. Rules:

- `SECRET` fields are NEVER emitted — the filter raises a panic in debug
  builds and silently strips in release builds (with an `EVT-REDACTION`
  event).
- `PII_IDENTIFYING`, `PII_QUASI_IDENTIFIER`, `PHI`, `PCI`, `FINANCIAL`,
  `FINANCIAL_REGULATED_CREDIT`, and `SENSITIVE_PIPA_ART23` fields are
  hashed or vaulted according to the owning regulator/pack.
- Tenant/product behavioral fields are body-redacted; only IDs and lengths
  emit unless a purpose-bound aggregate explicitly permits more.
- `INTERNAL_ONLY` / `PUBLIC` fields emit verbatim only when no subject,
  residency, or secret marker composes into the field.

## 10. Anti-patterns

1. **`String` field with no `data_class` annotation in a kernel struct.**
   Pre-commit blocks.
2. **`#[oyatie(data_class = "PUBLIC")]` on a name-like field** to dodge
   audit emission. Reviewer agent catches.
3. **Cross-pillar flow that bypasses an adapter** (direct DB peek across
   axis boundaries). Refused by cohesion lane.
4. **Derived value whose class is lower than its inputs.** The
   most-restrictive-input rule is enforced at compile time via the macro.
5. **DSR-capable field with no adapter coverage** in production.

## 11. Sources scanned

- [`decision-principles.json`](../../specs/decision-principles.json) DP-08 + [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-05.
- [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) (program scope).
- [`libs/oya-data-boundary-kernel/src/lib.rs`](../../libs/oya-data-boundary-kernel/src/lib.rs) and [`retention_policy.rs`](../../libs/oya-data-boundary-kernel/src/retention_policy.rs) (`DataClass`, `PrivacyDataClass`, aliases, operational/subject markers, `ClassificationLevel`, `RetentionPolicy`, `PurgeAction`).
- [`specs/cloud-production-quality-kits-target.json`](../../specs/cloud-production-quality-kits-target.json) QK-03 and [`specs/capability-tier-schema.json`](../../specs/capability-tier-schema.json) compliance/export/sunset/audit-retention shape (target/spec only; not runtime proof).
- [`docs/DOC-CATALOG.md`](../DOC-CATALOG.md) §4 lanes:
  `privacy-class-taxonomy-coverage`, `privacy-consent-flow-completeness`,
  `cross-tenant-access-fuzz`, `audit-chain-replay`.
- ADR-0003 (audit chain), ADR-0008 (Data Use Boundary).
- KR PIPC (Personal Information Protection Act),
  EU GDPR, HIPAA (US) — referenced via COMPLIANCE-MATRIX.
