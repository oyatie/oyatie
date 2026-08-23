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
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-data-class
enforcement_status:
  governance-data-class: F-PENDING-DATA-CLASS (crate missing; tracked in registry/stub-audit/2026-05-17/missing-fitness-crates.json)
  governance-dsr-cascade: F-PENDING-DSR-CASCADE (crate missing)
  governance-audit-emission: existing
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

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Every kernel struct field MUST carry a `data_class` annotation. The
annotation gates cross-pillar flow, audit-chain emission, DSR cascade
behavior, and observability redaction. This standard names the classes,
the transition matrix, and the cascade integration.

The program-level privacy scope (consent tiers, regulator mappings, DSR
SLAs) lives in [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md). This
standard supplies the per-field rules.

## 1. The class taxonomy

| Class | Examples | Cross-pillar flow | Logged in plain form? |
|---|---|---|---|
| `public` | published doc IDs, capability slugs, public marketplace metadata | freely | YES |
| `internal` | request IDs, capability latencies, queue lengths | freely | YES |
| `tenant-config` | tenant settings, feature-flag state, autonomy-tier bindings | within tenant scope only | YES (redact tenant ID in cross-tenant aggregations) |
| `tenant-data` | user-authored content, documents, search corpora | within tenant scope only | YES (with tenant ID; redact body for audit) |
| `pii` | name, email, phone, address, IP, device ID | tenant-scoped + consent-gated | NO (hashed in logs) |
| `phi` | medical, diagnostic, lab, prescriptions | tenant-scoped + consent-gated + jurisdiction-gated | NO (hashed in logs; vaulted at rest) |
| `financial` | account numbers, transaction IDs, balances | tenant-scoped + KYC-gated | NO (hashed in logs) |
| `secret` | tokens, API keys, encryption keys | NEVER cross-pillar; vault-only | NEVER logged |
| `regulated-jurisdiction` | KR-residency, EU-residency, US-state-residency markers | governs which pillars are allowed | YES (the marker only) |

Lane: `governance-data-class` checks every new kernel struct
field declaration for an annotation.

## 2. Annotation shape

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatientRecord {
    /// Stable subject identifier.
    #[oyatie(data_class = "pii")]
    pub subject_id: SubjectId,

    /// Diagnostic finding.
    #[oyatie(data_class = "phi", regulator = "MFDS", retention = "10y")]
    pub diagnosis: Diagnosis,

    /// Last-modified timestamp.
    #[oyatie(data_class = "internal")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

The `oyatie` attribute is a derive macro (or attribute macro) provided by
`kernel-data-class`. It emits compile-time metadata and runtime
descriptors consumed by:

- The audit-chain emission processor.
- The OTel redaction filter (per [`observability.md`](observability.md) §8).
- The DSR cascade walker (§5).
- The cross-pillar flow checker (§3).

## 3. Cross-pillar transition matrix

The seven axes (SaaS, Workspace, Vertical, Foundry, Cloud, Search,
Ads + Analytics) are pillars. Flows are governed by:

| From → To | `public` | `internal` | `tenant-config` | `tenant-data` | `pii` | `phi` | `financial` | `secret` |
|---|---|---|---|---|---|---|---|---|
| SaaS → Workspace | ✓ | ✓ | ✓ (same tenant) | ✓ (same tenant + consent) | gated | DENY | gated | DENY |
| SaaS → Foundry (Foundry agents act ON tenant data) | ✓ | ✓ | ✓ | ✓ + audit | ✓ + audit | ✓ + audit | ✓ + audit | DENY (Foundry uses secrets via `SecretProvider`, never receives them) |
| Vertical → Vertical (cross-vertical) | ✓ | ✓ | DENY | DENY | DENY | DENY | DENY | DENY |
| any → Search index | ✓ | ✓ (sanitized) | DENY | redacted | DENY | DENY | DENY | DENY |
| any → Ads/Analytics | aggregate only | aggregate only | DENY | DENY | DENY | DENY | DENY | DENY |
| any → Cloud (storage/runtime) | ✓ | ✓ | ✓ | ✓ (encrypted-at-rest) | ✓ + KMS | ✓ + dedicated KMS + jurisdiction | ✓ + KMS | ✓ (vault-only) |

"DENY" means: no plumbing, no opt-in, no flag. A flow that violates a row
fails the `governance-cohesion` lane and the
`governance-data-class` lane at the field-level.

Mathematical rule: the class of a derived value is the **lexicographic
maximum** of its inputs (e.g., `name + diagnosis = phi`).

## 4. Audit-chain emission requirements

Per [`observability.md`](observability.md) §4, every cross-pillar flow
of class `tenant-config`, `tenant-data`, `pii`, `phi`, `financial`, or
`regulated-jurisdiction` MUST emit an `EVT-DATA-EGRESS` (or class-specific
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
| `jurisdiction` | when `regulated-jurisdiction` |
| `redacted_payload_hash` | YES (proves emission without storing the body) |

Lane: `governance-audit-emission` validates the emission point
exists for every `data_class`-gated transition.

## 5. DSR cascade integration

Per [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md), every DSR (Data
Subject Request — access / rectification / erasure / portability) walks
**every** field in **every** axis whose `data_class` is `pii`, `phi`, or
`financial` and emits a proof-of-action receipt.

Mechanics:

1. The `platform-dsr-kernel` walks the catalog of kernel structs and
   their `data_class` annotations.
2. For each (`subject_id`, `class`) pair, the kernel queries each pillar
   adapter implementing `DsrCapable`.
3. Each adapter returns a list of records + a recommended action (delete,
   redact, export).
4. The kernel persists the action plan, executes per regulatory deadline
   (e.g., 30 days for KR PIPC), and emits `EVT-DSR-EXECUTED` per record
   touched.
5. Proof-of-erasure is hash-chained and signed.

Lane: `governance-dsr-cascade` validates that every `pii`/`phi`/
`financial` field has a registered `DsrCapable` adapter.

## 6. Schema migration discipline

Adding a new field to a kernel struct:

1. Declare the `data_class` in the same PR (compile-fail without it).
2. If `pii`/`phi`/`financial`/`secret`, update
   [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) class taxonomy in
   the same PR.
3. If `phi` or `regulated-jurisdiction`, update
   [`docs/COMPLIANCE-MATRIX.md`](../COMPLIANCE-MATRIX.md) regulator
   mapping.
4. Provide DSR cascade adapter coverage (§5) before the field reaches
   production.
5. Run `governance-schema-migration` (per AGENTS.md D14).

## 7. Field-naming conventions

- PII/PHI fields SHOULD be wrapped in a newtype (`SubjectId`,
  `Diagnosis`) so the data class is visible at every call site.
- Boolean PII flags ("is_minor") MUST carry `data_class = "pii"` even
  though boolean — the inference (about the subject) is the PII.
- Aggregated counts (k-anonymous, k ≥ 50) MAY be `internal`; k < 50 keeps
  the underlying class.

## 8. Cross-tenant isolation

Per the fitness lane `cross-tenant-access-fuzz` (DOC-CATALOG.md §4),
deterministic isolation probes prove cross-tenant access fails closed.
This standard adds the `data_class` participation:

- Every kernel function that takes a `tenant_id` MUST carry a
  `#[tenant_scoped]` attribute that emits a tenant-mismatch check at
  the trait boundary.
- Cross-tenant aggregations (e.g., billing rollups) MUST go through
  `platform-tenancy-aggregator` which strips `tenant-data`,
  hashes `tenant_id`, and emits the aggregate via `internal` class.

## 9. Observability redaction binding

The OTel redaction filter (per [`observability.md`](observability.md) §8)
reads the same `data_class` metadata at runtime. Rules:

- `secret` fields are NEVER emitted — the filter raises a panic in debug
  builds and silently strips in release builds (with an `EVT-REDACTION`
  event).
- `pii` / `phi` / `financial` fields are hashed (per-tenant salt).
- `tenant-data` fields are body-redacted; only IDs and lengths emit.
- `internal` / `public` fields emit verbatim.

## 10. Anti-patterns

1. **`String` field with no `data_class` annotation in a kernel struct.**
   Pre-commit blocks.
2. **`#[oyatie(data_class = "public")]` on a name-like field** to dodge
   audit emission. Reviewer agent catches.
3. **Cross-pillar flow that bypasses an adapter** (direct DB peek across
   axis boundaries). Refused by cohesion lane.
4. **Derived value whose class is lower than its inputs.** The
   lexicographic-max rule is enforced at compile time via the macro.
5. **DSR-capable field with no adapter coverage** in production.

## 11. Sources scanned

- [`decision-principles.json`](../../specs/decision-principles.json) DP-08 + [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-05.
- [`docs/PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) (program scope).
- [`docs/DOC-CATALOG.md`](../DOC-CATALOG.md) §4 lanes:
  `privacy-class-taxonomy-coverage`, `privacy-consent-flow-completeness`,
  `cross-tenant-access-fuzz`, `audit-chain-replay`.
- ADR-0003 (audit chain), ADR-0008 (Data Use Boundary).
- KR PIPC (Personal Information Protection Act),
  EU GDPR, HIPAA (US) — referenced via COMPLIANCE-MATRIX.
