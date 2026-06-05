---
doc_class: DPIA
template_id: TPL-DPIA
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry + ops-security
deciders: council-privacy, ops-security, axis-foundry, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 + EU AI Act Art. 27 (FRIA — fundamental-rights impact assessment)
related_adrs: [ADR-0025, ADR-0026, ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/threat-model.md
  - microservices/intelligence-providers/policy/data-residency.md
  - microservices/intelligence-providers/policy/credential-isolation.md
  - microservices/intelligence-providers/compliance.md
review_cadence: annually + on every vendor change + on every in-house-model rollout
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (capability-routed provider selection uses tenant-request profile)"
  - "Art. 35(3)(b): large-scale processing of special-category data — CONDITIONAL (PHI in pack-us-healthcare requests; sensitive data under PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
  - "EU AI Act Art. 27: high-risk AI system FRIA — CONDITIONAL on tenant workload classification"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46, 47, 49"
  - "EU AI Act Reg. (EU) 2024/1689 Arts. 13, 14, 27, 50"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII)"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 17/18/22-2/23/24/28/29/29-2/33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b) minimum necessary"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EDPB Guidelines 4/2019", "EDPB Guidelines 7/2022 on transfers"]
  pack-jp: ["APPI Arts. 17 + 18 + 23 + 24 + 27"]
  pack-sg: ["PDPA Part III + IV + V", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1/5/6/8/11/12"]
  pack-in: ["DPDPA 2023 §10/11/16"]
  pack-br: ["LGPD Arts. 6/7/11/33/38"]
  pack-ae: ["UAE PDPL Art. 22 + 23"]
  pack-ksa: ["PDPL Art. 9 + 29"]
doc_status: published
---

# Data Protection Impact Assessment: foundry-providers µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The foundry-providers µservice triggers two of the three Art. 35(3) automatic triggers + the EU AI Act Art. 27 FRIA trigger (conditional).

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | The router profiles every incoming request against capability requirements; the provider selection is a quasi-automated decision affecting where tenant data is processed (which vendor edge sees the prompt). |
| Art. 35(3)(b): Large-scale special-category data | **CONDITIONAL** | Pack-us-healthcare carries PHI in prompts when tenants emit it; pack-kr requests may include KR-PIPA Art. 23 sensitive data. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | Not applicable. |
| EU AI Act Art. 27 FRIA: deployer of a high-risk AI system | **CONDITIONAL** | When a tenant's workload classification is "high-risk" per EU AI Act Annex III (employment-decisions, credit-scoring, etc.), the provider-routing decision is part of the high-risk pipeline. |

In addition, the cross-border-transfer dimension (provider call sends tenant prompts off-host to vendor) engages GDPR Arts. 44–50 across packs.

Therefore: a DPIA is mandatory pre-deployment.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What.** Workload µservices submit prompts (request payload) to the router. The router selects a vendor + transport + region. The selected adapter sends the prompt to the vendor edge, receives the response, hashes and signs it, and returns it. Personal data may be present in the prompt and may be persisted by the vendor depending on vendor terms; oyatie's adapters negotiate Zero-Data-Retention (ZDR) where vendors support it.

**How.** In-process router decision → in-process adapter dispatch → OpenBao credential resolution (in-memory) → mTLS HTTPS to vendor edge → response capture → BLAKE3+Ed25519 envelope → `ProviderInvoked` event to audit-chain.

**Where.** Per-pack region pinning: pack-kr requests use vendor edges in or close to KR (Anthropic via KR-region SCC + ZDR; Gemini KR region; in-house KR region). Pack-eu uses EU vendor edges. Pack-us-healthcare uses HIPAA-eligible vendor regions + BAA. Off-pack vendor edges are default-deny.

**When.** Per request, real-time; provider-router decision is in-process and synchronous.

**Who.** Workload µservices (the principals); their tenants (the data subjects whose data may appear in prompts); the vendors (Anthropic / OpenAI / Google / in-house); the audit-chain consumer.

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PII_IDENTIFYING` | User names, emails, document subjects in prompt text | Art. 6(1)(b) contract; Art. 6(1)(f) legitimate interest with safeguards | varies by tenant emission |
| `PII_QUASI_IDENTIFIER` | URLs, IPs, file paths in prompt context | Art. 6(1)(f) legitimate interest | per-request |
| `SENSITIVE_GDPR_ART9` | Health / racial / political / religious data in prompts | Art. 9(2)(a) explicit consent OR Art. 9(2)(h) healthcare (pack-us-healthcare under BAA) | per pack policy |
| `SENSITIVE_PIPA_ART23` | KR sensitive data | KR PIPA Art. 23 explicit consent at tenant onboarding | per pack-kr |
| `AUTH_CREDENTIAL` | Vendor API key / subscription cookie (never persisted in plaintext) | Art. 6(1)(b) contract — necessity for operation | per (tenant, vendor) |
| `BEHAVIORAL_AI_USAGE` | Request size, response size, latency, cost | Art. 6(1)(b) + Art. 6(1)(f) | per call |
| `AUDIT` | `ProviderInvoked` event records | Art. 6(1)(c) legal obligation (audit) + Art. 6(1)(f) | per call |

### 2.3 Categories of data subjects

| Category | Description |
|---|---|
| Tenant end-users | Whose prompts pass through the adapter |
| Tenant operators | Whose credentials are stored in OpenBao |
| Audit recipients | Whose `ProviderInvoked` events are read for compliance |

### 2.4 Cross-border transfers

This is the **central DPIA concern** for foundry-providers: provider calls send prompts off-host to the vendor's edge. Per pack:

| Pack | Permitted vendors | Transfer mechanism | Notes |
|---|---|---|---|
| pack-kr | Anthropic KR (SCC + ZDR), Gemini KR, in-house KR | KR PIPA Art. 17 + 28 + 29 cross-border lawful basis | KR PIPC notification at tenant onboarding |
| pack-eu | Anthropic EU, OpenAI EU (post-SCC), Gemini EU, in-house EU | GDPR Art. 46 SCC 2021/914 + supplementary measures (Schrems II) | EDPB transfer-impact assessment |
| pack-us-healthcare | Anthropic (BAA + ZDR), in-house HIPAA region | HIPAA §164.502(e) BAA | Per tenant BAA execution |
| pack-jp | Anthropic JP, Gemini JP, in-house JP | APPI Art. 24 cross-border consent + adequacy | — |
| pack-sg | Anthropic SG, Gemini SG, in-house SG | PDPA §26 transfer limitation + adequacy | — |
| pack-au | Anthropic AU, Gemini AU, in-house AU | APP 8 cross-border accountability | — |
| pack-in | in-house IN (primary), Anthropic IN | DPDPA §16 cross-border (per government notification) | — |
| pack-br | in-house BR, Anthropic BR (post-SCC) | LGPD Art. 33 international transfer | — |
| pack-ae | in-house AE, Anthropic AE | UAE PDPL Art. 22 cross-border | — |
| pack-ksa | in-house KSA, Anthropic KSA | PDPL Art. 29 cross-border + SAMA compliance | — |

## Step 3 — Consult relevant stakeholders

| Stakeholder | Consulted | Outcome |
|---|---|---|
| Tenant operators | At onboarding | Per-pack permitted-vendor matrix accepted; tenant signs DPA |
| DPO (per pack) | Pre-launch | DPIA approved per pack |
| EU AI Act assessor | When tenant workload is high-risk | FRIA appended; tenant notified |
| KR PIPC | First pack-kr tenant | DPIA filing per Notice 2020-7 |
| Sub-processors (Anthropic / OpenAI / Google) | DPA / BAA execution | Vendor DPA per pack |

## Step 4 — Necessity and proportionality

| Principle | Treatment |
|---|---|
| Lawfulness | Art. 6 lawful basis per data class above |
| Purpose limitation | Provider invocation only; no secondary processing inside foundry-providers |
| Data minimisation | Adapters do NOT log prompt/response bytes; only hashes + metadata reach audit-chain |
| Accuracy | Vendor responses are signed (Ed25519); response shape conformance checked |
| Storage limitation | foundry-providers stores no prompt/response payloads; OpenBao leases auto-expire (15-min default) |
| Integrity + confidentiality | mTLS + Ed25519 + Cedar + OpenBao isolation |
| Accountability | `ProviderInvoked` events + Ed25519 + Mimir SLI |

## Step 5 — Identify risks to data subjects

| Risk | Likelihood | Severity | Initial rating | Treatment |
|---|---|---|---|---|
| Prompt exposure to vendor beyond contractual scope | Medium | Medium | Medium | ZDR negotiation + per-pack vendor whitelist + EU AI Act Art. 50 disclosure |
| Vendor data-breach exposes prompt content | Low | High | Medium | Vendor sub-processor DPA + breach-notification chain |
| Cross-pack data exfil via mis-routing | Low | High | Medium | Residency-aware router (T-08 mitigations) |
| Credential leak in chat / logs / git | Low | Critical | High | `oya-foundry-providers-credential-isolation` lane (T-01 mitigations) |
| In-house-model regression causes incorrect tool/decision output | Medium | Medium | Medium | Baseline-set parity + burn-rate auto-rollback |
| Tool-call exfil attempt | Low | High | Medium | Adapter never executes tools; `cell` Cedar gate |
| EU AI Act non-disclosure (Art. 50) | Medium | Medium | Medium | Per-call disclosure record schema |

## Step 6 — Identify mitigations

(See `threat-model.md` + `policy/credential-isolation.md` + `policy/data-residency.md` + `compliance.md` for the full register; key mitigations summarised here.)

| Risk | Mitigation | Owner | Evidence |
|---|---|---|---|
| Prompt exposure | ZDR + sub-processor DPA + EU AI Act Art. 50 disclosure | council-privacy | sub-processor list + per-call disclosure record |
| Vendor breach | Rotation runbook + breach-notification chain | ops-security | runbooks/credential-rotation.md |
| Cross-pack exfil | Residency-aware router + per-pack policy | axis-foundry | policy/data-residency.md + `residency-conformance` lane |
| Credential leak | OpenBao isolation + LEAN lane + `ResolvedCredential` opaque type | ops-security | credential-isolation.md + lean lane |
| In-house regression | Burn-rate auto-rollback + canary cohort + baseline set | axis-foundry | observability/PHASE-01 + IP-012 |
| Tool exfil | Adapter no-execute + Cedar gate at `cell` | council-privacy | adapter-impl + cell policy |
| EU AI Act non-disclosure | Per-call disclosure event | council-privacy | contracts/asyncapi/provider-events.yaml |

## Step 7 — Approve and record outcomes

| Outcome | Sign-off |
|---|---|
| DPIA approved per pack | council-privacy chair |
| FRIA appended for EU AI Act high-risk tenants | per tenant |
| Risk register published | published |
| Reviewed annually | next review 2027-05-17 |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=dpia --microservice foundry-providers` exits 0.
- Per-pack DPIA review committed to `evidence/dpia/<pack>/` at first-tenant onboarding.

## References

- ICO DPIA template (UK) — `ico.org.uk`.
- CNIL DPIA methodology (FR).
- KR PIPC Notice 2020-7.
- EDPB Guidelines 4/2019 on Art. 25.
- EDPB Guidelines 7/2022 on transfers.
- EU AI Act Reg. (EU) 2024/1689 — Arts. 13, 14, 27, 50.
- HIPAA 45 CFR §164.502(e) Business Associate Agreements.
- ADR-0117 — pack residency model.
- `microservices/intelligence-providers/threat-model.md`.
