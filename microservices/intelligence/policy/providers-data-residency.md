---
doc_class: PolicyDocument
title: Data-Residency Policy
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry + ops-security
deciders: council-privacy, council-architecture, ops-security, axis-foundry
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence/threat-model.md (T-08 cross-pack data leakage)
  - microservices/intelligence/dpia.md
  - microservices/intelligence/compliance.md
review_cadence: quarterly + on every pack activation + on every new vendor
doc_status: published
---

# Data-Residency Policy (foundry-providers µservice)

## Purpose

The provider-router must enforce per-pack data residency at decision time. This policy defines the canonical (pack × vendor × region × transport) matrix; the router refuses any tuple not present in the `permitted` table; default-deny.

## Per-Pack Permitted-Vendor Matrix (M01 launch)

### pack-kr (KR jurisdiction)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | KR (via SCC + ZDR) | KR PIPA Art. 17 + 28 + 29 cross-border | ZDR negotiated; sub-processor DPA executed |
| Anthropic | subscription | KR (via SCC + ZDR) | as above | subscription is FRAGILE; prefer API |
| Gemini | api | KR (Google Cloud KR region) | KR PIPA Art. 17 + 28 + 29 | sub-processor DPA executed |
| Gemini | subscription | KR | as above | subscription FRAGILE |
| in-house | n/a | KR (oyatie-owned vLLM/TGI in OCI ap-seoul-1) | self-served; no cross-border | preferred when capability fit |
| OpenAI | api | (FORBIDDEN) | no KR region + no SCC arrangement at M01 | conditional post-SCC |
| OpenAI | subscription | (FORBIDDEN) | as above | conditional post-SCC |

### pack-eu (EU jurisdiction)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | EU (via SCC 2021/914 + supplementary measures) | GDPR Art. 46 SCC | EDPB Schrems II TIA executed |
| Anthropic | subscription | EU | as above | FRAGILE |
| OpenAI | api | EU (data-residency option enabled; SCC) | GDPR Art. 46 SCC | post-OpenAI-EU-residency-launch (2024) |
| OpenAI | subscription | EU | as above | FRAGILE |
| Gemini | api | EU | GDPR Art. 46 SCC | sub-processor DPA |
| Gemini | subscription | EU | as above | FRAGILE |
| in-house | n/a | EU (OCI eu-frankfurt-1 + eu-amsterdam-1 DR pair) | self-served | preferred |

### pack-us-healthcare (HIPAA + state-PHI laws)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | US (HIPAA-eligible) | HIPAA BAA + ZDR | BAA executed per tenant |
| OpenAI | api | US (HIPAA-eligible region post-BAA) | HIPAA BAA | conditional per tenant BAA |
| OpenAI | subscription | (FORBIDDEN) | subscription channel does not support BAA | hard deny |
| Gemini | api | US (conditional BAA) | HIPAA BAA | conditional per tenant BAA |
| Gemini | subscription | (FORBIDDEN) | as above | hard deny |
| in-house | n/a | US HIPAA-eligible region | self-served + HIPAA controls | preferred |
| Anthropic | subscription | (FORBIDDEN for PHI tenants) | subscription does not support BAA | hard deny |

### pack-us (US non-HC)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | US | sub-processor DPA | default permitted |
| Anthropic | subscription | US | sub-processor DPA | FRAGILE |
| OpenAI | api | US | sub-processor DPA | default permitted |
| OpenAI | subscription | US | sub-processor DPA | FRAGILE |
| Gemini | api | US | sub-processor DPA | default permitted |
| Gemini | subscription | US | sub-processor DPA | FRAGILE |
| in-house | n/a | US (OCI us-ashburn-1 + us-phoenix-1) | self-served | preferred |

### pack-jp (Japan)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | JP (or US with APPI Art. 24 adequacy) | APPI Art. 24 cross-border + sub-processor DPA | adequacy declared |
| Gemini | api | JP | APPI Art. 24 | sub-processor DPA |
| OpenAI | api | JP (or US with APPI Art. 24) | APPI Art. 24 | conditional |
| in-house | n/a | JP (OCI ap-tokyo-1) | self-served | preferred |
| (subscription transports) | n/a | per same constraints | FRAGILE — case-by-case |

### pack-sg (Singapore)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | SG | PDPA §26 + MAS-TRM | DPA |
| Gemini | api | SG | PDPA §26 | DPA |
| OpenAI | api | SG | PDPA §26 | DPA |
| in-house | n/a | SG (OCI ap-singapore-1) | self-served | preferred |

### pack-au (Australia)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| Anthropic | api | AU | APP 8 + sub-processor DPA | DPA |
| Gemini | api | AU | APP 8 | DPA |
| OpenAI | api | AU | APP 8 | DPA |
| in-house | n/a | AU (OCI ap-sydney-1 + ap-melbourne-1) | self-served | preferred |

### pack-in (India)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| in-house | n/a | IN (OCI ap-hyderabad-1 + ap-mumbai-1) | self-served | **strongly preferred** (DPDPA §16) |
| Anthropic | api | IN (post-DPDPA-readiness; per government notification) | DPDPA §16 | conditional |
| Gemini | api | IN | DPDPA §16 | conditional |
| OpenAI | api | IN | DPDPA §16 | conditional |

### pack-br (Brazil)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| in-house | n/a | BR (OCI sa-saopaulo-1 + sa-vinhedo-1) | self-served | preferred |
| Anthropic | api | BR (post-SCC) | LGPD Art. 33 international transfer | conditional |
| Gemini | api | BR | LGPD Art. 33 | conditional |
| OpenAI | api | BR | LGPD Art. 33 | conditional |

### pack-ae (UAE)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| in-house | n/a | AE (OCI me-abudhabi-1 + me-dubai-1) | self-served | preferred |
| Anthropic | api | AE | UAE PDPL Art. 22 + DPA | conditional |
| Gemini | api | AE | UAE PDPL Art. 22 | conditional |
| OpenAI | api | AE | UAE PDPL Art. 22 | conditional |

### pack-ksa (Saudi Arabia)

| Vendor | Transport | Region | Mechanism | Notes |
|---|---|---|---|---|
| in-house | n/a | KSA (OCI me-jeddah-1 + me-riyadh-1) | self-served | preferred |
| Anthropic | api | KSA (post-DPA + SAMA review) | PDPL Art. 29 + SAMA Cybersecurity Framework | conditional |
| Gemini | api | KSA | as above | conditional |
| OpenAI | api | KSA | as above | conditional |

## Default-Deny Posture

Any (pack × vendor × transport × region) tuple not present in a `permitted` row is **denied by default** at router decision time. The router returns a `NoCompliantProvider` error with a structured diagnostic.

Explicit-forbid rows are duplicated as Cedar `forbid` rules in `policy/openbao-credential.cedar` and `policy/provider-router-tenant-scope.cedar` for defence-in-depth.

## Activation Status

| Pack | Activation status |
|---|---|
| pack-kr | M01 launch (live) |
| pack-eu | conditional (first EU tenant SCC; expected M01+1) |
| pack-us | conditional |
| pack-us-healthcare | conditional (per BAA) |
| pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | conditional (per first tenant) |

## SCC Requirements (EU-specific)

Per GDPR Arts. 44–50 and the EDPB Schrems II framework:
- Pre-transfer: tenant signs DPA referencing SCC 2021/914.
- Transfer Impact Assessment (TIA) executed for each vendor edge (Anthropic EU, OpenAI EU, Gemini EU).
- Supplementary measures: vendor offers technical (encryption + access control) and contractual (sub-processor list + audit rights) safeguards.
- Per-tenant transfer register (Art. 30) emitted as a `ProviderInvoked` event subset.

## EU AI Act Disclosure (Art. 50)

When the tenant's `jurisdiction_code` is `EU` and the workload classification is not exempt:
- Every `ProviderInvoked` event includes the structured Art. 50 disclosure: `{provider_name, model_id, jurisdiction, model_version_or_date, system_prompt_hash, response_hash, request_id}`.
- Tenants are informed at onboarding that requests will carry this disclosure for transparency obligations.

## Verification

- `cargo run -p oya-dev-cli -- gate validate residency-conformance --microservice foundry-providers` exits 0.
- Per-pack negative-test: `tests/integration/residency_deny_<pack>.rs` validates that a forbidden tuple deterministically denies.
- Quarterly review: this matrix is refreshed with vendor-region updates.

## References

- ADR-0117 — pack residency model.
- GDPR Arts. 44–50; SCC 2021/914; EDPB Schrems II framework.
- KR PIPA Arts. 17 + 28 + 29; PIPC notification framework.
- HIPAA 45 CFR §164.502(e) BAA.
- APPI Art. 24 (Japan).
- PDPA §26 (Singapore); MAS-TRM v2021.
- APP 8 (Australia).
- DPDPA 2023 §16 (India).
- LGPD Art. 33 (Brazil).
- UAE PDPL Art. 22.
- KSA PDPL Art. 29 + SAMA Cybersecurity Framework 2017.
- EU AI Act Reg. (EU) 2024/1689 Art. 50.
