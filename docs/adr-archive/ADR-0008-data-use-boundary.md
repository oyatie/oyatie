---
id: ADR-0008
status: Superseded
superseded_by: [ADR-709]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0008: Data Use Boundary — twelve data classes with HARD_DENY for PHI/PCI/PIPA-Art23/CHILDREN, orthogonal subject_class, purpose-permission matrix, and four-pillar flow matrix

> **Status:** Accepted
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-privacy`
> **Date:** 2026-05-09 (accepted 2026-05-14)
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0006, ADR-0007, ADR-0010

---

## Context

Oyatie simultaneously hosts regulated tenant data (PHI, PII, PCI, KR `신용정보`, KR `영상정보`), a search index that *could* index that data, an ad-targeting auction that *could* monetize behavioral signal, an AI agent runtime that operates on tenant data, and a cloud control plane that provisions resources holding all of the above. No competitor faces this exact intersection. A single PHI leak into the search index or as an ad-targeting feature is a regulator-visible event that cannot be unwound — KR PIPA Art 34, GDPR Art 33, HIPAA Breach Notification Rule, and PCI-DSS all assume the leak is permanent and require notification within 24–72 hours.

The cohesion thesis (ADR-0001) compounds the risk. Without a structurally enforceable contract for what data may flow across axes, every cross-microservice PR becomes a gamble against the privacy posture. Policy-only enforcement has historically failed (a contractor adds a topic without reading the memo). The contradiction ledger LEDG-001, LEDG-003, LEDG-005 record the prior failures; the resolution path adopted in PRIVACY-PROGRAM.md §2 is the structural-enforcement-first design that this ADR formalizes.

---

## Decision

We adopt the **Data Use Boundary** as the contract that governs which tenant data may flow across axes under what consent, for what purpose, to what subject class, in what jurisdiction. The boundary is enforced at six structural layers (compile-time first), uses an *orthogonal* subject-class attribute (not a 13th data class), uses a *purpose-permission matrix* (not a linear ladder), and uses a *four-pillar flow matrix* (Org / Person / Public / Opt-in-Consumer).

Ratification binds the companion operating text in [`PRIVACY-PROGRAM.md`](../PRIVACY-PROGRAM.md) §2.2.2 as the published consent-tier UI projection. Tier labels are non-authoritative bundles; only persisted purpose-permission rows are grants.

### 1. Twelve data classes (closed enumeration)

| # | Class | Examples | Default ad rule | Default analytics rule |
|---|---|---|---|---|
| 1 | `INTERNAL_ONLY` | crypto keys, audit-chain entries, capability registry internals | DENY | DENY |
| 2 | `PHI` | clinical record, lab result, prescription, diagnosis | **HARD_DENY** (no consent override) | DP-only after 5+ aggregation |
| 3 | `PII_IDENTIFYING` | name + phone + RRN + address + face image | DENY (consent for analytics; never for ads) | DP-only |
| 4 | `PII_QUASI_IDENTIFIER` | birthdate + ZIP + gender (k-anonymity threshold) | DENY without aggregation | k-anonymous (k≥10) |
| 5 | `PCI` | card PAN, CVV, account number | **HARD_DENY** | **HARD_DENY** |
| 6 | `FINANCIAL_KR` (신용정보) | KR credit score, loan history | **HARD_DENY** | DP-only with FSC consent flow |
| 7 | `BEHAVIORAL_TENANT_PRODUCT` | which workflow tenant ran, which Workspace feature | DENY for ads outside tenant | per-tenant + cross-tenant DP |
| 8 | `BEHAVIORAL_ADS` | impression / click / conversion (ads-axis-internal) | OK first-party; cross-tenant requires separate consent | OK |
| 9 | `DECLARED_PREFERENCE` | tenant-declared interest categories, opted-in segments | OK for ad targeting | OK |
| 10 | `SEARCH_QUERY` | what a user searched | DP-aggregated only | DP only |
| 11 | `PUBLIC` | open-web crawled pages, public legal corpus | OK | OK |
| 12 | `SENSITIVE_PIPA_ART23` (health, sex life, race, religion, political, biometric, criminal record) | as KR-PIPA Art 23 enumerates | **HARD_DENY** | DP-only with explicit purpose-bound consent |

**HARD_DENY** = no consent override is honored; the path is structurally impossible. CHILDREN status is **NOT** a 13th class; it is a `subject_class` attribute. A child + DECLARED_PREFERENCE is hard-denied for ads by `subject_class`, not by `data_class`. Both checks compose; both must pass.

### 2. Orthogonal subject_class attribute

```rust
// crates/oya-data-boundary-policy-kernel
pub struct RecordAttributes {
    pub data_class: DataClass,                 // one of the 12 above
    pub subject_class: SubjectClass,           // adult | minor | elderly | vulnerable | ...
    pub minor_status: Option<MinorStatus>,     // age band + guardian-consent receipt
    pub jurisdiction: JurisdictionCode,
    pub residency: ResidencyClass,
    pub lawful_basis: LawfulBasis,             // consent | contract | legal_obligation | vital | public_task | legit_interest
    pub purpose: PurposeId,                    // bound to the consent receipt's declared purpose
    pub derivation_lineage: Vec<DerivationStep>, // for the inference-boundary check
    pub consent_receipt_id: Option<ConsentReceiptId>,
}

pub enum SubjectClass {
    Adult,
    Minor { age_band: AgeBand },               // <14 KR / <16 GDPR-K / <13 COPPA
    Elderly,
    Vulnerable,
    Authority,                                  // public official, etc.
}
```

### 3. Purpose-permission matrix (NOT a linear ladder)

Permissions are independently grantable, revocable, and scoped by `(purpose × data_class × tenant_class × geography × subject_class) → ALLOW | DENY | REQUIRES_CONSENT_RECEIPT`. The closed enumeration of purposes:

| Purpose ID | Description | Lawful basis examples |
|---|---|---|
| `service_operation` | Run the service the tenant subscribed to | contract, legitimate interest |
| `security_fraud_prevention` | Detect / prevent abuse | legitimate interest, legal obligation |
| `regulatory_compliance` | Required by an applicable regulator | legal obligation |
| `tenant_analytics_first_party` | Tenant-scoped analytics | contract |
| `cross_tenant_aggregate_anonymous` | Cross-tenant aggregate (k≥10) | consent |
| `personalization_in_oya_saas` | Per-user personalization within Oyatie | consent |
| `ad_targeting_declared` | DECLARED_PREFERENCE only | consent |
| `ad_targeting_behavioral` | BEHAVIORAL_ADS first-party | consent |
| `cross_device_linking` | Linking a user across devices | consent |
| `cross_tenant_individual` | Linking a user across tenants | consent (escalation gated) |
| `model_training_oya` | Training Oyatie models on tenant data (de-identified) | consent (escalation gated) |
| `model_training_third_party` | Forwarding tenant data to external providers | consent (escalation gated) |
| `data_export_to_subject` | DSR fulfillment (export) | legal obligation |
| `data_deletion_cascade` | DSR fulfillment (delete cascade) | legal obligation |

Granting `personalization_in_oya_saas` does NOT grant `ad_targeting_*`. Granting `cross_device_linking` does NOT grant `cross_tenant_individual`. Tier labels (e.g. UI buttons "Essential / Analytics / Personalization / Ads") are *bundles* of (purpose × class) grants — never authoritative grants by themselves.

### 4. Four-pillar flow matrix (Org / Person / Public / Opt-in-Consumer)

| From → To | Org | Person | Public | Opt-in-Consumer |
|---|---|---|---|---|
| **Org →** | ALLOW (intra-org) | DENY without explicit subject consent | ALLOW with org consent + subject re-consent if PII | REQUIRES_CONSENT_RECEIPT |
| **Person →** | DENY (no auto-flow into org pillar) | ALLOW (intra-person) | DENY without explicit consent + per-class check | REQUIRES_CONSENT_RECEIPT |
| **Public →** | ALLOW (public ingestion into org analytics) | DENY (would create unsolicited profile) | ALLOW (intra-public) | DENY |
| **Opt-in-Consumer →** | REQUIRES_CONSENT_RECEIPT | DENY | ALLOW (consumer publishes to public via own intent) | ALLOW (intra-consumer) |

Any cross-pillar flow that does not match this matrix is a CI failure on the producing axis.

### 5. Tenant-class overrides (vertical defaults — not raisable by tenant admin)

| Tenant class | Always blocked from ads | Source |
|---|---|---|
| Healthcare (any FHIR-touching) | classes 2, 3, 4, 12 | HIPAA + KR `의료법` + MFDS |
| Fintech (payment / open banking) | classes 5, 6 | KR-FSC + PCI-DSS + `신용정보법` |
| Defense / Public safety | 1, 2, 3, 4, 6, 12 | KR-NIS + ITAR if exposed |
| Children-product (education) | minor `subject_class` always hard deny | KR `청소년보호법` + GDPR-K + COPPA |
| Public sector | 2-7, 12 | KR `공공정보법` + `정보공개법` |
| Corporate / SME (default) | 2, 5, 6, 12 | baseline |

### 6. Six-layer structural enforcement

1. **Schema annotation** — every `.proto`, SQL DDL, event schema carries `oyatie.data_class = "..."` per field; `oya-governance-data-class` lints.
2. **Lint-time check** — `oya-governance-data-class` walks every cross-microservice call site and verifies the source class is allowed at the destination.
3. **Source crate singleton** — only `oya-ads-gate` and `oya-analytics-router` may publish to ads/analytics topics; other crates are rejected at the eventing layer (ADR-0005).
4. **Architecture fitness gate** — `oya-governance-flat-crates` rejects any new flat crate whose dep graph imports an ads/analytics adapter from outside the approved gate crates.
5. **Audit-chain emission per decision** — every ad-targeting decision emits an evidence record (ADR-0003) with consenting tenant, user, classes used, audience, ad, and the rules that fired; missing emission = capability-invocation reject.
6. **Runtime guard** — final guard at the auction boundary re-validates consent vs purpose; blocks if any class drifted.
The first kernel policy-gate implementation surface for this boundary is `libs/oya-data-boundary-kernel/src/policy_gate.rs`, with ownership recorded through `libs/oya-data-boundary-kernel/OWNERS` under `council-privacy`.

### 7. Inference boundary (closes the laundering loophole)

A derived attribute inherits the **most-restrictive** class of any input feature. A "purchase propensity" score derived from PHI inherits PHI and is blocked from ads regardless of how it is named. The fitness function walks the model-feature lineage (model registry → feature store → source columns) and rejects any model whose feature ancestry includes a class the deployment context isn't authorized for.

### 8. DSR + withdrawal cascade

DSR (export, delete, restrict) or consent withdrawal triggers a 30-day cascade across tenant store, search index, ads attribution store, analytics warehouse, agent runtime context, audit chain (deletion-evidence + invalidation pointer per ADR-0003), and cloud KMS shred (per-record DEK destruction). Proof-of-erasure is published to the trust portal.

---

## Consequences

### Positive

- Privacy posture becomes a compile-time invariant; structural failure modes catch regressions before merge.
- Every cross-microservice data flow has an audit record (ADR-0003); regulator queries answerable in minutes.
- Verticals get tighter defaults that cannot be misconfigured.
- Foundry agents inherit the contract automatically (ADR-0007); no special-case agent privacy logic.
- Closes LEDG-001 (12-class taxonomy with orthogonal subject_class), LEDG-003 (purpose-permission matrix), LEDG-005 (four-pillar matrix).

### Negative

- Initial schema-annotation lift is large (one-time across all proto/SQL/events).
- Slows feature velocity in search/ads axes until the boundary is operational.
- DP / k-anonymity adds noise to advertiser reporting; mitigation: publish parameters so advertisers plan for it.
- Tenant onboarding must declare vertical correctly; mis-declaration is a hard failure mode.

### Operational

- Per-tenant per-class ε-budget tracking adds ~1 ms per query.
- DSR cascade SLA = 30 days (PIPA + GDPR); proof-of-erasure stress-tested quarterly.
- Vertical onboarding includes a regulatory pack auto-applied based on declared class (ADR-0010).
- On-call: any HARD_DENY-class alert pages privacy lead within 5 minutes; regulator notification within 72 hours (PIPA Art 34 / GDPR Art 33), tenants within 24 hours.

---

## Alternatives considered

### Alternative A — Policy-only enforcement (memo + review)

- **Pros:** zero tooling cost.
- **Cons:** leaks the moment a contractor adds a topic without reading the policy.
- **Rejected because:** failure mode demonstrated in legacy corpus.

### Alternative B — Per-axis privacy frameworks

- **Pros:** axis autonomy.
- **Cons:** every axis re-implements 90% of the same logic; drift between axes is the failure mode.
- **Rejected because:** ADR-0001 cohesion.

### Alternative C — 13-class taxonomy with CHILDREN as its own class

- **Pros:** simpler at the surface.
- **Cons:** child-status is orthogonal to content (a child can have PHI or DECLARED_PREFERENCE); a single class collapses the dimension.
- **Rejected because:** orthogonality is correct (Codex review 2026-05-09); resolved as LEDG-001.

### Alternative D — Linear consent ladder (Essential → Analytics → Personalization → Ads → Cross-tenant)

- **Pros:** simple UX.
- **Cons:** violates purpose-limitation; cross-device implies cross-tenant which implies ad-targeting which is wrong.
- **Rejected because:** purpose-permission matrix is correct (LEDG-003).

---

## Open questions

1. **Q1.** Can `BEHAVIORAL_TENANT_PRODUCT` flow to cross-tenant aggregate analytics if source tenant opted in but user did not? Default: NO; user consent required. → owner: `council-privacy`.
2. **Q2.** Workspace personal-use surfaces (e.g. Personal) — keep "no ads, ever" inviolable, or carve a P-tier? Default: keep inviolable. → ADR-0012 + GTM.
3. **Q3.** Healthcare PHI exclusion — any non-FHIR healthcare records exempt? Default: NO. → owner: `council-privacy` + `vertical-healthcare`.
4. **Q4.** Agent-runtime tool-call traces — whose data class? Default: most-restrictive (inference boundary §7). → ADR-0007.
5. **Q5.** Cross-tenant search index — per-record opt-in or per-collection? Default: per-record. → owner: `axis-search`.
6. **Q6.** Per-record cryptographic shred — required for all classes or just HARD_DENY? Default: required for all HARD_DENY; opt-in for others. → ADR-0009.

---

## References

- KR PIPA Art 15, 17, 22, 22-2, 23, 28-8, 39-7; KR `정보통신망법`, `청소년보호법`, `의료법`, `의료광고심의위원회` 가이드라인, `신용정보법`, 금융감독원 광고 가이드라인, `정치자금법`
- GDPR Art 6, 7, 9, 17, 22, 25
- HIPAA Privacy Rule, Security Rule
- PCI-DSS v4.0
- PCI DSS source-version planning projection: `specs/pci-dss-level-1-readiness-plan.json` (stable `PCI-DSS-L1-v4` pack id with PCI DSS v4.0.1 metadata only; no readiness, control-completeness, ROC/AOC/QSA/ASV, evidence-acceptance, or tenant CDE activation claim)
- `docs/PRIVACY-PROGRAM.md` §2 (full Data Use Boundary text)
- `docs/COMPLIANCE-MATRIX.md` §3.1, §3.2, §3.3, §3.4
- `docs/CONTRADICTION-LEDGER.md` LEDG-001, LEDG-003, LEDG-005, LEDG-006
- ADR-0001 (cohesion), ADR-0002 (Tenant kernel + consent attribute), ADR-0003 (audit emission), ADR-0006 (per-property data_class on OG), ADR-0007 (Cedar enforcement), ADR-0010 (regional pack tenant-class overrides)
