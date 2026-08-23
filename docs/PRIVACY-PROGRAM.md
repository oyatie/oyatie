---
purpose: Oyatie — Privacy Program
doc_status: published
---

# Oyatie — Privacy Program

> **Status:** Accepted boundary text — ADR-0008 ratified 2026-05-14; operating model remains monthly-maintained.
> **Companion docs:** [PRD.md](PRD.md), [DESIGN.md](DESIGN.md), [security-program/security-program.json](security-program/security-program.json), [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md).
> **Owners:** Privacy & Legal council; Architecture council co-signs cross-axis flows; Founder is final arbiter on scope changes.

---

## 1. Why this program exists

Oyatie is the only EaaS provider that simultaneously hosts:

- regulated tenant data (PHI, PII, PCI, KR-신용정보, 영상정보),
- a search index that *could* index that data,
- an ad-targeting auction that *could* monetize behavioral signal,
- an AI agent runtime that operates on tenant data on behalf of users,
- a cloud control plane that provisions resources holding all of the above.

No competitor faces this exact intersection. Google has search and ads but no tenant-regulated workloads. Salesforce has tenant workloads but no search/ads. AWS has cloud but no search/ads/agents. Naver has search and ads but no tenant SaaS at this depth. **Oyatie's privacy program is therefore stricter than any single competitor's program — it has to absorb all of theirs.**

Failure modes are unrecoverable: a single PHI leak into the search index or as an ad-targeting feature is a regulator-visible event that cannot be unwound. Hence the program prioritizes **structural enforcement over policy text** — the boundary is a compile-time/CI-enforceable contract, not a memo.

---

## 2. Data Use Boundary (ADR-0008 Accepted)

> **Title:** ADR-0008 — Data Use Boundary across Application, Search, Ads, Analytics, Cloud, and Agent Runtime
> **Status:** Accepted via ADR-0008 (2026-05-14); blocks substantive data-plane work that lacks data-class and purpose-permission coverage.
> **Related ADRs:** ADR-0001, ADR-0002, ADR-0003, ADR-0006, ADR-0007, ADR-0010, ADR-0011, ADR-0034, ADR-0038, ADR-0049.

### 2.1 Context

Oyatie is repositioning as a single cohesive ecosystem-as-a-service across 7 axes. Several existing ADRs were drafted under a narrower scope:
- ADR-0008 says cross-pillar data flow is forbidden.
- ADR-0008 hard-no's personal-email mining.
- ADR-0050 says tenant data lives only in tenant collections.

These ADRs assumed an internal SaaS scope. With search and ads becoming first-class axes, a clean structural contract is needed for what *can* and *cannot* flow across pillars under what *consent* and to what *purpose*. Without the contract, every PR in the cloud/search/ads axes is a gamble against the privacy posture.

### 2.2 Decision

#### 2.2.1 Data class taxonomy (exactly 12 classes)

Every record in Oyatie's stores carries exactly one `data_class` annotation. The annotation is propagated via schema (proto/SQL/event-schema) and enforced at the boundary by lint and runtime check.

| # | Class | Examples | Default ad rule | Default analytics rule |
|---|---|---|---|---|
| 1 | `INTERNAL_ONLY` | crypto keys, audit-chain entries, capability registry internals | DENY | DENY |
| 2 | `PHI` | clinical record, lab result, prescription, diagnosis | **HARD DENY** (no consent override) | DP-only after 5+ aggregation |
| 3 | `PII_IDENTIFYING` | name + phone + RRN + address + face image | DENY (consent required for analytics; never for ads) | DP-only |
| 4 | `PII_QUASI_IDENTIFIER` | birthdate + ZIP + gender (k-anonymity threshold) | DENY without aggregation | k-anonymous (k≥10) |
| 5 | `PCI` | card PAN, CVV, account number | **HARD DENY** | **HARD DENY** |
| 6 | `FINANCIAL_KR` | KR credit score, loan history (`신용정보`) | **HARD DENY** | DP-only with FSC consent flow |
| 7 | `BEHAVIORAL_TENANT_PRODUCT` | which workflow tenant ran, which feature | DENY for ads outside tenant; per-tenant analytics OK | per-tenant + cross-tenant DP |
| 8 | `BEHAVIORAL_ADS` | impression / click / conversion (ads-axis-internal) | OK for first-party attribution; cross-tenant retargeting requires separate consent | OK |
| 9 | `DECLARED_PREFERENCE` | tenant-declared interest categories, opted-in segments | OK for ad targeting | OK |
| 10 | `SEARCH_QUERY` | what a user searched | DP-aggregated only; never per-user ads tied | DP only |
| 11 | `PUBLIC` | open-web crawled pages, public legal corpus | OK for both | OK |
| 12 | `SENSITIVE_PIPA_ART23` (KR Art 23 — health, sex life, race, religion, political views, biometric, criminal record) | as KR-PIPA Article 23 enumerates | **HARD DENY** (KR-PIPA forbids ad use) | DP-only with explicit purpose-bound consent |

**HARD DENY** = no consent override is honored; the path is structurally impossible. Implementation forces this via compile-time class annotation + tenant-ads-gate refusal + ad-side schema rejection.

> **Subject and legal-capacity attributes are orthogonal to data class** (Codex review 2026-05-09). Child status, jurisdiction, residency, lawful basis, etc. are attributes attached to the **subject** of a record, not the **content** of a record. Every record carries:
>
> ```rust
> pub struct RecordAttributes {
>     pub data_class: DataClass,           // one of the 12 above
>     pub subject_class: SubjectClass,     // adult, minor (<14 KR / <16 GDPR-K / <13 COPPA), elderly, vulnerable, ...
>     pub minor_status: Option<MinorStatus>, // age band + guardian-consent receipt
>     pub jurisdiction: JurisdictionCode,
>     pub residency: ResidencyClass,
>     pub lawful_basis: LawfulBasis,       // consent, contract, legal-obligation, vital-interest, public-task, legitimate-interest
>     pub purpose: PurposeId,              // bound to the consent receipt's declared purpose
>     pub derivation_lineage: Vec<DerivationStep>,  // for the inference-boundary check (§2.2.5)
>     pub consent_receipt_id: Option<ConsentReceiptId>,
> }
> ```
>
> A record about a minor with `data_class: PHI` is hard-denied for ads by class; a record about a minor with `data_class: DECLARED_PREFERENCE` is hard-denied for ads by `subject_class.minor_status` regardless of its data class. The two checks compose; both must pass.

#### 2.2.2 Purpose-permission matrix (replaces the linear ladder per Codex review 2026-05-09)

A linear ladder is unsafe: cross-device identity should not imply ad-targeting; cross-tenant aggregate use should not imply cross-device profiling; "essential" processing is often contract/legal necessity, not consent. Permissions are **independently grantable, revocable, and scoped** by `(purpose × data class × tenant class × geography × subject class)`. Tier labels exist only as UI shorthand bundles, never as authoritative grants.

##### Purposes (closed enumeration)

| Purpose ID | Description | Lawful basis examples |
|---|---|---|
| `service_operation` | Necessary to run the service the tenant subscribed to | contract, legitimate interest |
| `security_fraud_prevention` | Detect / prevent abuse, IVT, account takeover | legitimate interest, legal obligation |
| `regulatory_compliance` | Required by an applicable regulator | legal obligation |
| `tenant_analytics_first_party` | Tenant-scoped analytics on tenant-owned data | contract |
| `cross_tenant_aggregate_anonymous` | Cross-tenant aggregate analytics, k-anonymous (k≥10) | consent |
| `personalization_in_saas` | Per-user personalization within Oyatie SaaS surfaces | consent |
| `ad_targeting_declared` | Ad targeting using DECLARED_PREFERENCE only | consent |
| `ad_targeting_behavioral` | Ad targeting using BEHAVIORAL_ADS first-party signal | consent |
| `cross_device_linking` | Linking a user across devices | consent |
| `cross_tenant_individual` | Linking a user identity across tenants | consent (rare; escalation gated) |
| `model_training_oya` | Training Oyatie-internal models on tenant data (de-identified) | consent (escalation gated) |
| `model_training_third_party` | Forwarding tenant data to external model providers | consent (escalation gated) |
| `data_export_to_subject` | DSR fulfillment (export to data subject) | legal obligation |
| `data_deletion_cascade` | DSR fulfillment (delete cascade) | legal obligation |

##### Permission matrix

A permission is `(purpose, data_class, tenant_class, geography, subject_class) → ALLOW / DENY / REQUIRES_CONSENT_RECEIPT`. The matrix is the authoritative source; consent receipts cite the specific (purpose, data_class) pair.

Each permission grant is:
- **purpose-bound** — consent declares purposes; using data for a different purpose is a breach.
- **independently grantable** — granting `personalization_in_saas` does NOT grant `ad_targeting_*`.
- **independently revocable** — revoking one permission does not affect others; revocation cascades to all derived data within 30 days (PIPA Art 39-7 / GDPR Art 17).
- **subject-scoped** — minor subjects auto-deny `ad_targeting_*` regardless of guardian consent on `service_operation`.
- **geography-scoped** — KR subjects honor PIPA defaults; EU subjects honor GDPR; US California subjects honor CCPA/CPRA; per regional pack.
- **receipt-stamped** — every grant emits a consent receipt to the audit chain (ADR-0028 / ADR-0003) with hash-anchored timestamp.

##### Tier labels (UI shorthand only — NOT authoritative grants)

Tier labels exist for UI ergonomics (e.g. tenant onboarding shows "Essential / Analytics / Personalization / Ads / Cross-tenant" buttons). Each button corresponds to a *bundle* of (purpose, data_class) permissions. The button is a convenience; the **stored grants** are the per-permission rows. A user MAY grant ANALYTICS but not PERSONALIZATION; a user MAY grant CROSS_DEVICE but not AD_TARGETING; the matrix is independently navigable.

##### Consent-tier mapping (UI bundle projection)

These tiers are UI presets only. The consent service expands each preset into explicit `(purpose, data_class)` grant rows, stamps every row into the audit chain, and evaluates the resulting rows against tenant class, geography, and subject class before use.

| UI tier | Purpose rows emitted | Default data-class ceiling | Hard-deny floor | Revocation cascade |
|---|---|---|---|---|
| Essential | `service_operation`, `security_fraud_prevention`, `regulatory_compliance` | Minimum classes needed for subscribed service | PHI/PCI/PIPA-Art23/CHILDREN remain purpose-limited; never ads | Service stops or degrades to legal minimum; derived caches purged within 30 days |
| Tenant analytics | `tenant_analytics_first_party` | Tenant-owned classes excluding HARD_DENY ad floors | PCI and raw HARD_DENY classes remain blocked from non-compliance analytics | Analytics aggregates invalidated; DP budgets closed |
| Cross-tenant aggregate | `cross_tenant_aggregate_anonymous` | k-anonymous / DP aggregates only (`k≥10`, `k≥25` for PIPA Art 23) | No row-level export, no individual linking | Aggregate cohorts re-keyed or removed within 30 days |
| Personalization | `personalization_in_saas` | Per-user personalization inside Oyatie SaaS surfaces | Does not grant ads, model training, cross-device, or cross-tenant individual use | Profiles and derived recommendations purged within 30 days |
| Declared ads | `ad_targeting_declared` | `DECLARED_PREFERENCE` only | PHI, PII identifying, PCI, FINANCIAL_KR, SEARCH_QUERY, PIPA Art 23, and minors always blocked | Audiences removed and active campaigns re-evaluated immediately |
| First-party ads attribution | `ad_targeting_behavioral` | `BEHAVIORAL_ADS` first-party signal only | No tenant product behavior, PHI/PII/PCI/financial/sensitive classes | Attribution windows closed; future decisions denied |
| Cross-device linking | `cross_device_linking` | Identity-link metadata needed for the declared service | Does not grant cross-tenant individual linking or ads | Link graph edge deleted; dependent caches purged |
| Model training — Oyatie | `model_training_oya` | De-identified, lineage-tracked records allowed by class and tenant policy | No HARD_DENY classes; no minors; no raw regulated identifiers | Training corpus exclusion attestation emitted; derived artifacts quarantined if needed |
| Model training — third party | `model_training_third_party` | Escalation-gated de-identified export only | Default DENY unless council-approved; no HARD_DENY classes | Provider deletion proof required; trust-portal evidence published |
| DSR export/delete | `data_export_to_subject`, `data_deletion_cascade` | Subject-owned records required by law | N/A — legal-obligation path | Export or proof-of-erasure completed within SLA |


##### Four-pillar matrix (per Codex review 2026-05-09)

> *(Required matrix per recon P0-DUB. Models the cross-pillar contract from ADR-0008 + ADR-0008.)*

The four pillars: **Org-owned**, **Person-owned**, **Public-corpus**, **Opt-in-Consumer**. Cross-pillar flow is the most contradiction-prone path.

| From → To | Org | Person | Public | Opt-in-Consumer |
|---|---|---|---|---|
| **Org →** | ALLOW (intra-org) | DENY (org cannot push into person pillar without explicit subject consent) | ALLOW (org publishes to public corpus only with org consent + subject re-consent if PII) | REQUIRES_CONSENT_RECEIPT (org may share with consumer pillar only via opted consumer choice) |
| **Person →** | DENY (person data does not auto-flow to org pillar) | ALLOW (intra-person) | DENY (person data not published to public corpus without explicit consent + per-class check) | REQUIRES_CONSENT_RECEIPT |
| **Public →** | ALLOW (public ingestion into org analytics) | DENY (public corpus does not auto-flow to person — would create unsolicited profile) | ALLOW (intra-public) | DENY |
| **Opt-in-Consumer →** | REQUIRES_CONSENT_RECEIPT | DENY | ALLOW (consumer publishes to public via own intent) | ALLOW (intra-consumer) |

Any cross-pillar flow that does not match this matrix is a CI failure on the producing axis.

#### 2.2.3 Tenant-class overrides (vertical defaults)

Some tenant verticals get tighter defaults that **cannot be raised** even by tenant admin:

| Tenant class | Forced data-class blocklist for ads | Rationale |
|---|---|---|
| Healthcare (any FHIR-touching tenant) | classes 2, 3, 4, 12 always blocked from ads | HIPAA + KR-MFDS + 의료법 |
| Fintech (any payment or open-banking tenant) | classes 5, 6 always blocked from ads | KR-FSC + PCI-DSS + 신용정보법 |
| Defense / Public safety | classes 1, 2, 3, 4, 6, 12 always blocked from ads | KR-NIS + ITAR (if exposure) |
| Children-product (e.g. education tenant) | `subject_class = CHILDREN_UNDER_14` always hard deny (orthogonal to data_class taxonomy per ADR-0008 §2) | KR-청소년보호법 + GDPR-K |
| Public sector tenant | classes 2-7, 12 always blocked from ads | 공공정보법 + 정보공개법 |
| Corporate / SME (default) | classes 2, 5, 6, 12 always blocked; rest opt-in | baseline |

#### 2.2.4 Structural enforcement (compile-time first)

The boundary is enforced at six layers, each independently:

1. **Schema annotation.** Every `.proto`, every SQL DDL, every event schema carries `oyatie.data_class = "...";` per field. Missing annotation = CI failure.
2. **Lint-time check.** `governance-data-class` walks every cross-axis call site and verifies the source class is allowed at the destination.
3. **Source crate singleton.** The *only* crates allowed to source tenant data into ads/analytics are `platform-ads-gate` and `platform-analytics-router`. All other crates attempting to publish to ads/analytics topics are rejected at the eventing-backbone layer.
4. **Architecture fitness gate.** `governance-flat-crates` rejects any new flat crate whose dep graph imports an ads/analytics adapter from outside the approved gate crates.
5. **Audit-chain emission per decision.** Every ad-targeting decision emits an evidence record with consenting tenant, consenting user (if applicable), data classes used, audience id, ad id, decision rationale (which rules fired). Missing emission = capability-invocation reject.
6. **Runtime guard.** Even after ingestion, a final guard at the auction boundary re-validates consent tier vs declared purpose and blocks if any class drifted.

#### 2.2.5 Inference boundary (closes the laundering loophole)

A derived attribute inherits the **most-restrictive** class of any input feature. Example: a "purchase propensity" score derived from PHI + behavioral data inherits `PHI` and is therefore blocked from ads regardless of how it was named.

The fitness-function check walks the model-feature lineage (model registry → feature store → source columns) and rejects any model whose feature ancestry includes a class that the model's deployment context isn't authorized for.

#### 2.2.6 DP + k-anonymity wrappers

Every tenant/user/public boundary applies a DP wrapper. Per-tenant per-class ε-budget is tracked in a central ledger; budget exhaustion blocks further queries until the next refresh window. ADR-0008 governs the gateway shape.

Aggregate exports (analytics dashboards, advertiser reporting) use k-anonymity ≥ 10 by default (≥ 25 for PIPA Art 23). Re-identification attacks are tested quarterly.

#### 2.2.7 Korea-specific obligations (non-bypassable)

| Obligation | Implementation |
|---|---|
| KR data residency (PIPA Art 17) | Default tenancy `region: KR-Seoul1`, `residency: strict_kr`; a tenant must explicitly opt out for any cross-region replication. |
| PIPA banner mandatory | Every UI surface in KR locale shows the consent banner first, with purpose-bound granular controls. |
| 청소년 보호법 (minors) | Any tenant marked as serving < 19 forces minors-protect mode; 13-year cutoff for hard deny absent guardian consent. |
| 의료광고 review | Healthcare-vertical ads flow through 의료광고심의위원회 review queue. |
| 금융광고 review | Fintech-vertical ads flow through 금융감독원 review pattern (FSC). |
| 정치광고 transparency | Political-category ads are public-archived on a transparency surface; spend disclosed. |
| Data export to outside KR | Per PIPA Art 28-8, require explicit user-or-tenant consent + purpose declaration; emit audit record per export. |

#### 2.2.8 Agent-runtime specifics

Agents (Foundry, axis 3) run **under a tenant's autonomy ceiling**:
- An agent **cannot** acquire a higher data-class permission than the tenant has granted.
- An agent acting on a user's behalf inherits the user's consent, never the tenant's broader consent.
- Agentic ad-buying defaults to "recommend-only" (human approves the bid). Auto-execution of ad-buys requires explicit per-tenant uplift to autonomy tier T3 or above (per ADR-0022 persona tier).
- Every agent step emits an audit record with capability + data classes touched + autonomy tier + consent context.
- One-click revoke pulls all derived agent context (RAG cache, tool-call traces, anything class-tagged by the user) within 24 hours.

#### 2.2.9 DSR + withdrawal cascade

Per GDPR + KR-PIPA, a DSR (export, delete, restrict) or consent withdrawal triggers a cascade across all cross-axis stores:

1. **Authoritative tenant store** — record marked `pending_dsr` immediately; cascade deadline = 30 days (PIPA) or 30 days (GDPR Art 12).
2. **Search index** — delete all per-tenant private and cross-tenant entries derived from the affected records; emit deletion-evidence record.
3. **Ads attribution store** — purge impression/click/conversion records keyed by the user; preserve aggregate-only counters per k-anonymity rule.
4. **Analytics warehouse** — purge per-user rows; preserve aggregated facts only after re-aggregation removes the affected user.
5. **Agent runtime context** — purge RAG caches, tool-call traces, agent memory keyed to the user.
6. **Audit chain** — emits a "dsr-fulfilled" record with cascade proof; the prior records are NOT deleted (chain is append-only) but pointers are annotated with the deletion-evidence record's hash.
7. **Cloud storage** — block-storage shred (cryptographic erasure of the DEK) for any per-record encrypted blobs.

The proof-of-erasure record is published back to the user / tenant via the trust portal.

#### 2.2.10 Class transitions

A record can only **weaken** its data-class via explicit human approval (e.g., a tenant decides a previously `PII_QUASI` field is now `PUBLIC`). **Tightening** (consent withdrawal, escalation discovery, regulatory shift) is automatic and cascades.

### 2.3 Consequences

**Positive:**
- Structural enforcement makes the privacy posture a compile-time invariant, not a policy memo.
- Every cross-axis data flow has an audit record; regulator queries are answerable in minutes.
- Tenants can opt into more usage incrementally without trusting a long policy document.
- Verticals get tighter defaults that cannot be misconfigured.
- Foundry agents inherit the contract automatically — no special-case agent privacy logic.

**Negative:**
- Initial implementation requires schema annotation across all proto / SQL / events (large lift; one-time cost).
- Slows feature velocity in the search/ads axes until the boundary is operational.
- DP / k-anonymity adds noise to advertiser reporting; some advertisers will object; mitigation is to publish the aggregation parameters so advertisers can plan for it.
- Tenant-class overrides require tenant onboarding to declare vertical correctly; mis-declaration is a hard failure mode.

**Operational:**
- Per-tenant per-class ε-budget tracking adds ~1ms to every query; acceptable.
- DSR cascade SLA = 30 days (matches PIPA + GDPR); cascading proof-of-erasure stress-tested quarterly.
- Vertical onboarding includes a "regulatory pack" auto-applied based on declared class.

### 2.4 Alternatives considered

1. **Policy-only enforcement.** Rejected — leaks the moment a contractor adds a topic without reading the policy.
2. **Per-axis privacy frameworks.** Rejected — every axis re-implements 90% of the same logic; drift between axes is the failure mode.
3. **Trust-the-data-team black-box review.** Rejected — does not scale and creates bottleneck on every cross-axis PR.

### 2.5 Open questions (must close before ratification)

| # | Question | Owner |
|---|---|---|
| Q1 | Is `BEHAVIORAL_TENANT_PRODUCT` ever allowed to flow to cross-tenant aggregate analytics if the source tenant opted in but the user did not? Default proposed: NO; user consent required. | Privacy + Product |
| Q2 | Personal: keep "no ads, ever" inviolable, or carve a P-tier that allows opt-in ads? Default proposed: keep inviolable (per existing brand). | Founder + GTM |
| Q3 | Healthcare PHI exclusion: are any non-FHIR healthcare records exempt from the hard-deny? Default proposed: NO; healthcare-vertical = hard deny. | Healthcare + Privacy |
| Q4 | Do agent-runtime tool-call traces inherit the tenant's data class, the user's data class, or the most-restrictive? Default proposed: most-restrictive (§2.2.5 inference boundary). | Foundry + Privacy |
| Q5 | Cross-tenant search index — per-record opt-in by tenant, or per-collection? Default proposed: per-record (more work, cleaner). | Search + Privacy |
| Q6 | Whether `DECLARED_PREFERENCE` collected via tenant-side surveys can flow to ads-axis without re-consent. Default proposed: yes if survey explicitly stated ad use. | Ads + Privacy |
| Q7 | Per-record cryptographic shred (envelope encryption with per-record DEK) — required for all classes or just PHI/PCI/PIPA Art 23? Default proposed: required for all *_HARD_DENY classes; opt-in for others. | Cloud + Privacy |

### 2.6 References

- KR-PIPA: 개인정보보호법 Articles 15, 17, 22, 22-2, 23, 28-8, 39-7
- KR-정보통신망법
- KR-청소년보호법
- KR-의료법, 의료광고심의위원회 가이드라인
- KR-신용정보법, 금융감독원 광고 가이드라인
- KR-정치자금법 (political ad transparency)
- GDPR Articles 6, 7, 9, 17, 22, 25
- HIPAA Privacy Rule, Security Rule
- PCI-DSS v4.0
- IAB IPA, Apple Privacy Sandbox aggregation, SKAdNetwork
- ADR-0011 (audit event), ADR-0028 (audit-chain), ADR-0006 (engine-enforced isolation), ADR-0022-0140 (persona tier, data ownership pillars), ADR-0003 (audit chain immutability)

---

## 3. Privacy operating model

The ADR is the contract; this section is how we run the program day-to-day.

### 3.1 Roles

| Role | Responsibility |
|---|---|
| **Privacy Council Lead** | Owns the Data Use Boundary ADR; approves class taxonomy changes; signs DPIA reports. |
| **Tenant Privacy Officer** (per tenant) | Tenant-side counterpart; receives breach notifications, manages consent surface. |
| **DSR Operator** | Runs DSR cascade; verifies proof-of-erasure; coordinates with regulator if breach. |
| **Audit-Chain Engineer** | Owns audit-chain ADR-0003 implementation; ensures emissions are complete and tamper-evident. |
| **Vertical Privacy Lead** (one per regulated vertical) | Owns vertical-specific overrides, regulator-facing compliance evidence. |

### 3.2 Cadence

| Cadence | What |
|---|---|
| Daily | Audit-chain emission integrity check (automated). |
| Weekly | DSR queue review; consent-revocation-cascade SLA dashboard. |
| Monthly | Per-vertical privacy review; DP ε-budget exhaustion report. |
| Quarterly | DPIA refresh per regulated capability; re-identification attack red team; regulatory-change watch. |
| Annually | Independent audit (KISA, SOC2, ISO 27701, KR-ISMS-P alignment). |

### 3.3 Incident response (privacy-class)

1. Auto-detect: structural enforcement layers (§2.2.4) emit alerts on any rejected cross-axis call.
2. Escalate: any alert touching `HARD_DENY` classes pages the privacy lead within 5 minutes.
3. Containment: revoke consent surface for affected tenants; freeze impression/click streams keyed to the affected records.
4. Notification: regulators within 72 hours (PIPA Art 34; GDPR Art 33). Tenants within 24 hours.
5. Postmortem: published to trust portal within 30 days.

### 3.4 Tenant onboarding privacy gates

Every new tenant onboarding flow includes:
1. Vertical declaration (auto-applies tenant-class override).
2. Region & residency selection.
3. PIPA banner shown first.
4. Granular consent surface (purpose × class).
5. DPIA acknowledgment by tenant admin.
6. Audit-chain consent receipt issued and stored.

### 3.5 Continuous compliance evidence

Per ADR-0050 governance umbrella + Issue #1577:
- Auto-collected control evidence (HIPAA, KISA, MFDS, FSC, GDPR, SOC2) emitted to evidence portal on every relevant capability invocation.
- Auditor self-serve evidence pack regeneration ≤ 4 hours.
- Annual third-party attestations published; trust portal hosts the chain-anchor proofs.

---

## 4. Cross-axis privacy obligations

| Axis | Obligation | Evidence |
|---|---|---|
| 1. SaaS | Tenant data lives in tenant boundary; cross-tenant flows only via §2.2 boundary | Audit-chain + per-tenant consent receipts |
| 2. Vertical | Vertical-specific overrides (§2.2.3) auto-applied | Vertical-pack adoption record |
| 3. Foundry | Agents inherit autonomy-ceiling-bounded data-class permissions (§2.2.8) | Per-step audit-chain record |
| 4. Foundry | Schema lint (§2.2.4 layer 1) enforced in CI | CI lane evidence |
| 5. Cloud | Per-resource KMS shred for HARD_DENY classes (§2.5 Q7) | KMS deletion evidence |
| 6. Search | Per-tier index segregation (§2.2.1 + §2.2.6) | Per-shard data-class manifest |
| 7. Ads | Singleton tenant-ads-gate is the only source (§2.2.4 layer 3) | Ads-gate audit log |

---

## 5. Sources scanned

- KR PIPA, GDPR, HIPAA, PCI-DSS, KISA guidance
- ADRs 0011, 0028, 0030, 0032, 0106, 0125, 0131, 0132, 0134, 0136, 0140, 0225 + 127-ADR full corpus
- `/Users/jasonlee/oyatie/docs/raw/greenfield-ads-analytics.md` Section Q (Data Use Boundary draft)
- `/Users/jasonlee/oyatie/docs/raw/rename-and-contradiction.md` H1, H2, H3, H17, H18, [wave name TBD per PRD §3.1]0, [wave name TBD per PRD §3.1]1 (Data Use Boundary group)
- `/Users/jasonlee/oyatie/docs/raw/greenfield-search.md` Section H (Safety + Compliance) and L (KR-Launch)
- `.omx/ultragoal/brief.md`, `goals.json`
- User directives 2026-05-08, 2026-05-09 (axes + cohesion + privacy)

*Footer regenerated whenever this doc is edited.*
