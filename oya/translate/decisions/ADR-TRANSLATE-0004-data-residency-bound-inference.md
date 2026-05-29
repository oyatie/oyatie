---
doc_class: AdrSpec
template_id: TPL-ADR
adr_id: ADR-TRANSLATE-0004
title: Data-residency-bound inference
status: Accepted
deciders: council-architecture, council-privacy, ops-security, axis-translate, ops-compliance
date: 2026-05-17
microservice: translate
supersedes: []
superseded_by: []
related_adrs: [ADR-0117, ADR-0135, ADR-0131, ADR-TRANSLATE-0001]
related_artifacts:
  - microservices/translate/PRD.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/threat-model.md
  - microservices/translate/dpia.md
  - microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md
doc_status: published
---

# ADR-TRANSLATE-0004 — Data-residency-bound inference

## Context

The single most-critical contractual + legal commitment translate makes to sovereign tenants — Korean (PIPA Art. 28), EU (GDPR Arts. 44–50), Indian (DPDPA 2023 §16), Chinese (PIPL Arts. 38–43), Japanese (APPI Art. 24), Brazilian (LGPD Art. 33), UAE (PDPL), KSA (PDPL + SAMA), etc. — is that **tenant content does not cross the tenant's pack boundary during inference**.

This is **not** a soft preference; it is a HARD legal invariant. A breach is:

- Sev-1 (P0) per `failure-modes.md` FM-70.
- Regulator-notifiable per `compliance.md` §"Breach Notification" within 72 h (most packs).
- Contract-terminating per tenant DPA standard clauses.

Industry references:

- **GDPR Arts. 44–50** (chapter V, transfers to third countries).
- **EDPB Recommendations 01/2020** (post-Schrems-II supplementary measures).
- **KR PIPA Art. 28** + Art. 23-2 (sensitive data cross-border).
- **CN PIPL Arts. 38–43** (cross-border data transfer).
- **CN Cybersecurity Law + Data Security Law** (CSL + DSL).
- **IN DPDPA 2023 §16** (cross-border transfer).
- **HIPAA 45 CFR §164.502(e)** + §164.530 (BAA + records).
- **APPI Art. 24** (Japan cross-border).
- **LGPD Art. 33** (Brazil cross-border).
- **PDPA SG cross-border transfer guidance**.
- **AU Privacy Act APP 8**.
- **UAE PDPL** + **KSA PDPL** + **SAMA**.

This ADR formalizes the technical posture that makes cross-region inference **default-deny at the router decide layer**, not merely policy-layer.

## Decision

### 1. Default-deny at router decide

The `oya-translate-router-domain::residency_filter()` (per `IP-003-translate-router-domain.md`) is the first and **HARD** filter applied to any candidate engine. It implements:

```text
permit candidate iff
    candidate.vendor   ∈ tenant.policy.residency.permitted_vendors AND
    candidate.region   ∈ tenant.policy.residency.permitted_regions
```

`tenant.policy.residency.default_deny == true` is a **kernel-level invariant** (`ResidencyConstraint::default_deny: bool`) — set true by every code path that constructs the struct; verified by a property test `proptest_residency_constraint_default_deny`.

If no candidate passes residency filter → `RouterError::NoResidencyCompliantEngine`; downstream returns 422 to caller; alert fired (`oya_translate_residency_no_compliant_engine_total`).

### 2. Per-pack engine whitelist (canonical source: `policy/data-residency.md`)

| Pack | Permitted engines (vendor × region) |
|---|---|
| pack-kr | in-house (KR-region), Anthropic (KR-region via SCC + ZDR), Google Cloud Translation (KR-region via SCC), DeepL (DE-EU; conditional on tenant PIPA Art. 28 consent) |
| pack-eu | in-house (EU-region), Anthropic (EU-region), OpenAI (EU-region post-SCC), Google (EU-region), DeepL (DE-EU native) |
| pack-us | any (per tenant DPA) |
| pack-us-healthcare | in-house (HIPAA region), Anthropic (BAA + ZDR); others per-tenant BAA |
| pack-jp | in-house (JP-region), Anthropic (JP-region), DeepL (JP-region), Google (JP-region) |
| pack-cn-stub | **in-house ONLY (CN-region); ALL external vendors FORBIDDEN per PIPL Arts. 38–43** |
| pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | per pack overlay matrix |

The whitelist is canonicalized in `policy/data-residency.md` and rendered into per-pack Helm overlays via `iac/kustomize/overlays/<pack>/`. Drift is caught by the `oya-translate-data-residency-correctness` BLOCKER lane.

### 3. SLI + alarm: cross-region inference event count = 0 invariant

`oya_translate_residency_violation_total` is a canary metric: any value > 0 fires a Page-1 alert (per `microservices/translate/runbooks/sovereign-tenant-cross-region-leak-incident-p0.md`). The metric counts attempted-but-rejected at the router (logged as info) AND completed-but-detected (Sev-1 incident). Steady-state value = 0.

### 4. Defense-in-depth

Five orthogonal checkpoints prevent cross-region inference:

1. **Kernel invariant** — `ResidencyConstraint::default_deny: bool = true`.
2. **Domain filter** — `residency_filter()` first-pass.
3. **Cedar policy** — `policy/translate-tenant-scope.cedar` per-pack deny rules.
4. **Engine adapter check** — adapter refuses if (vendor, region) ≠ (decision.vendor, decision.region).
5. **Network-level enforcement** — `cell` µservice egress proxy allowlists only per-pack endpoints; any cross-pack egress is dropped at the network layer.

### 5. Per-vendor regional endpoint pinning

- Anthropic — pinned region matched against pack (e.g., pack-kr → KR-region endpoint).
- Google Cloud Translation — `regional` endpoint forced via API URL parameter.
- DeepL — pinned to DE-EU; cross-border permitted only with explicit tenant PIPA Art. 28 consent for pack-kr.
- OpenAI — pinned to EU-region for pack-eu post-SCC; not used in pack-kr / pack-jp / pack-in / pack-br / pack-ae / pack-ksa M01.
- In-house — always in-pack (foundry-runtime co-located).

### 6. TM + termbase respect pack boundary

Per ADR-TRANSLATE-0002: per-tenant Meilisearch index lives in pack-pinned cluster; cross-pack TM read forbidden by both Cedar + index-isolation + RLS. FM-72 covers cross-pack TM lookup attempts.

### 7. Bulk-translate + document-translate honor pack boundary

S3 buckets per pack; bulk-worker pool per pack; gVisor sandbox pods per pack. Cross-pack S3 read forbidden by IAM policy.

### 8. Real-time stream honors pack boundary

Per-session pod schedule constrained to pack region; foundry-runtime endpoint per pack.

### 9. pack-cn-stub strict mode

Scaffolding for `pack-cn-stub` ships in M01 IaC but **production activation is NOT permitted in M01**. Activation requires:

- KR-PIPA-equivalent CN regulatory mapping documented.
- China-resident KMS keys (Aliyun KMS or equivalent).
- China-resident vLLM cluster (no Anthropic / OpenAI / Google / DeepL endpoints reachable).
- All external adapter replicas set to 0 (enforced by overlay + OPA gatekeeper).
- Tenant DPA per PIPL Arts. 38–43.

## Alternatives Considered

### Alternative A — Best-effort residency (no default-deny; let tenant configure)

- **Pros**: simpler; flexible.
- **Cons**: misconfiguration → P0 incident; regulator-notifiable; contract-terminating; legal exposure.
- **Verdict**: rejected. Default-deny is the only legally + commercially defensible posture.

### Alternative B — Per-tenant residency declaration (no per-pack matrix)

- **Pros**: tenant-flexible.
- **Cons**: no policy aggregation; every tenant duplicates the legal analysis; cross-tenant comparison impossible; auditor burden.
- **Verdict**: rejected. Per-pack matrix is the canonical source; tenant pack assignment is the lever.

### Alternative C — Cross-region with SCC fallback (auto-attach SCC on cross-border)

- **Pros**: maximal engine selection latitude; auto-compliance.
- **Cons**: SCC ≠ permission for sensitive data (PIPA Art. 23 + GDPR Art. 9 require explicit consent); auto-attach risks regulatory non-compliance; tenant DPA may not authorize.
- **Verdict**: rejected. SCC is a transfer mechanism, not a permission slip; per-pack matrix is canonical.

### Alternative D — Allow cross-region within "adequate" jurisdictions only (e.g., EU → adequate decisions list)

- **Pros**: GDPR-compliant cross-EU-adequate.
- **Cons**: adequacy decisions can be revoked (Schrems I + II precedent); engineering hard-deps on adequacy is brittle.
- **Verdict**: rejected for default; explicit per-pack overlay permits adequate-jurisdiction cross-border on tenant DPA opt-in.

### Alternative E — Network-level enforcement only (cell egress proxy)

- **Pros**: simpler app layer; defense at L3/L7.
- **Cons**: app-layer routing decision can still emit logs / metrics / audit events with cross-region intent; defense-in-depth requires all five layers.
- **Verdict**: rejected as sole control; included as Layer 5 of defense-in-depth.

### Alternative F — Pack-cn full production in M01

- **Pros**: addressable market.
- **Cons**: CN regulatory regime (CSL + DSL + PIPL) requires CN-licensed entity + in-country LLM compliance review + Aliyun KMS + significant additional engineering; not feasible M01.
- **Verdict**: rejected M01; scaffolding only; tracked M03-onward.

## Consequences

### positive

1. **Cross-region inference is structurally impossible** in steady state — five-layer defense-in-depth; CI-validated; HARD BLOCKER lane.
2. **Tenant trust** — published ADR + tenant-readable policy/data-residency.md + per-call audit event covering decision.region.
3. **Regulatory defensibility** — auditor walkthrough is straightforward; per-pack matrix is canonical; legal citations on file.
4. **Per-pack overlays bake regional legal posture into deployment** — operational + legal posture co-evolve.

### negative

1. **Reduced engine selection latitude in restrictive packs** (pack-kr, pack-jp, pack-in, pack-br, pack-ae, pack-ksa, pack-cn-stub) — in-house parity bar must close gap; ADR-0026 + ADR-TRANSLATE-0001 fallback constraints.
2. **Per-pack engineering + ops cost** — 11 packs × per-pack overlays + 11 Postgres clusters + 11 Meilisearch + 11 Valkey; folded into capacity-model.md.
3. **Per-vendor regional-endpoint operational variance** — each vendor's regional endpoint matrix changes; per-vendor adapter-pin runbook tracks.

### neutral

1. **DR pair limited to intra-region or intra-pack** — RTO unchanged (≤ 35 min per multi-region.md); residency-compliant.
2. **TM accumulation pack-bound** — tenant's pack-kr TM never serves pack-eu queries (correct posture; tenants accept on contract).
3. **pack-cn-stub scaffolding without production** is a known M01 limitation; documented + tracked.

## Validation

- `oya-translate-data-residency-correctness` BLOCKER lane: zero residency violations in last release window.
- `tests/integration/router_residency_filter_default_deny.rs` — Alternative A path verified.
- `tests/integration/cn_stub_external_vendor_blocked.rs` — pack-cn-stub external adapter denied.
- `tests/e2e/full_e2e_per_pack_engine_whitelist.rs` — end-to-end per pack.
- Quarterly chaos drill: simulate cross-region misroute; verify rejection at all 5 layers.
- Annual privacy-counsel audit: per-pack regulatory mapping refresh.

## References

- ADR-0117 — pack residency model (canonical).
- ADR-0135 — connect super-app expansion (parent ADR).
- ADR-0131 — per-microservice flat layout.
- ADR-TRANSLATE-0001 — engine routing (residency-bound).
- GDPR Arts. 5/6/9/22/25/32/44–50.
- EDPB Recommendations 01/2020.
- KR PIPA Arts. 17/22-2/23/28; PIPC Notice 2020-7; PIPC Notice on Cross-Border Transfer.
- CN Cybersecurity Law + Data Security Law (DSL) + PIPL Arts. 38–43.
- HIPAA 45 CFR §164.502(e) + §164.530.
- APPI Art. 24.
- LGPD Art. 33.
- PDPA SG cross-border guidance.
- AU Privacy Act APP 8.
- DPDPA 2023 §16.
- UAE PDPL + KSA PDPL + SAMA.
- ICO Adequacy + transfer guidance.
- Schrems I + II rulings.
- `policy/data-residency.md` (sibling artifact; canonical matrix).
- `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` (sibling artifact).
