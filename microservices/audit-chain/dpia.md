---
doc_class: DPIA
template_id: TPL-DPIA
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-audit-chain
deciders: council-privacy, ops-security, axis-audit-chain, council-architecture
methodology: ICO DPIA + CNIL PIA + GDPR Art. 35 + KR PIPA Art. 33 (개인정보영향평가)
related_adrs: [ADR-0028, ADR-0003, ADR-0117, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/audit-chain-merkle-ed25519.json]
related_artifacts:
  - microservices/audit-chain/threat-model.md
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/policy/data-residency.md
  - microservices/audit-chain/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, retention defaults, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — PARTIAL (chain is monitoring not profiling, but covers every state change across every µservice for every tenant)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (pack-us-healthcare PHI possible; KR PIPA Art. 23 sensitive personal info)"
  - "Art. 35(3)(c): publicly accessible monitoring — NO"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (PII protection), A.5.31 (legal)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3/15/17/22-2/23/28/29/29-2/33/34/36", "PIPC Notice 2020-7 (DPIA methodology)", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) (risk analysis)", "§164.312(b)+(c)(1) (audit + integrity)", "§164.316(b)(2) (retention)"]
  pack-eu: ["GDPR Arts. 25/30/32/35", "EDPB Guidelines 4/2019", "EDPB Recommendations 01/2020 (post-Schrems II)"]
  pack-jp: ["APPI Arts. 17/20/24/26-2"]
  pack-sg: ["PDPA Part III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1+8+11", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §8-11"]
  pack-br: ["LGPD Arts. 6/7/11/33/38"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 23"]
  pack-ksa: ["KSA PDPL Royal Decree M/19/2021 Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: audit-chain µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) is engaged because:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a) systematic + extensive evaluation | **PARTIAL** | The chain is monitoring of state-changes, not profiling per se. However, it captures *every* event across *every* µservice across *every* tenant; the cumulative footprint is systematic-and-extensive enough that EDPB guidance counts this as DPIA-triggering. |
| Art. 35(3)(b) large-scale special-category processing | **YES (conditional on pack)** | pack-us-healthcare carries PHI; pack-kr carries KR PIPA Art. 23 sensitive data. |
| Art. 35(3)(c) systematic public monitoring | **NO** | n/a |

Korean PIPC Notice 2020-7 mandates DPIA when system handles sensitive PIPA-Art-23 data at scale — engaged.

DPIA is therefore mandatory pre-deployment for pack-eu + pack-kr + pack-us-healthcare; voluntary best-practice for other packs (and authored anyway).

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** audit-chain accepts AuditEvent submissions from every workload µservice; durable-stores each event in a per-pack append-only WAL; periodically batches events per `(tenant, period)` into a Merkle tree; signs the tree's root with a pack-resident HSM-backed Ed25519 key; publishes the signed root to S3 WORM + Mimir + GitHub-pinned manifest; serves verification + query reads to tenants + auditors; honours per-pack retention windows; cascades DSR redactions while preserving Merkle proofs of redaction.

**How:** Each event is durable-written via emission-rest → Postgres WAL + S3 raw-blob; sealing-worker reads the WAL, builds Merkle trees, calls HSM to sign roots, publishes to three channels; verification-rest serves per-event proofs against published roots; query-rest serves tenant- and auditor-scoped forensic queries; retention-cascade-worker enforces per-pack retention.

**Where:** Per-pack region-pinned (pack-kr → KR OCI ap-seoul-1; pack-eu → eu-frankfurt-1; pack-us-healthcare → us-ashburn-1; etc.). One audit-chain instance per pack; chains never cross packs.

**When:** Continuous; emission cadence per workload µservice; sealing cadence 1s default; retention-cascade daily; key rotation 90d.

**Who:** Workload µservices (every oyatie µservice); tenant operators (read-only via query-rest); auditors (JIT-token-scoped); operators (axis-audit-chain + ops-security + council-privacy); HSM operator (Oracle, with no read access to oyatie's key material — partition-isolated).

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `AUDIT` | Every state-changing event metadata (event_id, source µservice, principal, tenant_id, emitted_at) | Art. 6(1)(c) legal obligation (records of processing per Art. 30); Art. 6(1)(f) legitimate interest (operational integrity) | ~10⁶ events/day per medium tenant; varies |
| Event payload (variable class) | The source-µservice-supplied payload; class declared by source per Bominal ADR-0003 | inherited from source µservice's lawful basis | ~1 KB average |
| `PII_IDENTIFYING` (when leaked) | If a source µservice fails redaction and emits PII in payload | Art. 6(1)(c) for the audit purpose; source µservice's lawful basis for the underlying record | target = 0; DSR remediation if detected |
| `PHI` (pack-us-healthcare only) | Same — if source µservice fails redaction | HIPAA §164.502(a) TPO; Art. 9(2)(h) | target = 0 |
| `SENSITIVE_PIPA_ART23` | Principal identity (SPIFFE → tenant_id mapping); subject_hash with auxiliary | KR PIPA Art. 15 + 23; explicit consent at tenant onboarding | 1 per event |
| `SECRET` | HSM signing keys (HSM-bound; never leave) | not personal data | per-pack rotation |

**Geographical scope:** Pack-pinned per `policy/data-residency.md`; no cross-pack movement.

**Cross-border transfer:** Forbidden by default. Tenant-controlled export bundle to a receiving-bucket attested by tenant is the only path; SCC-required for EU-resident tenants per Arts. 44–46.

### 2.3 Context

- **Data subjects:** Tenants' end-users (subject_hash carries through from source events when applicable); tenant operators; oyatie operators. Joint controllership per Art. 26 with the tenant.
- **Relationship to subjects:** Subject is upstream of audit-chain; oyatie holds the audit-of-the-action, not the action's payload primary. Tenant DPA carries the joint-controllership cascade.
- **Reasonable expectations:** Every modern SaaS has an audit log; tenants expect this. End-users (tenants' users) expect operational record-keeping per tenant's privacy notice.
- **Previous experience:** Bominal audit-chain (predecessor) operated 24mo without DPA complaint.
- **Industry codes:** None directly applicable; voluntary alignment with RFC 6962 (Certificate Transparency Merkle shape) + eIDAS AdES.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Maintain SOC 2 / ISO 27001 / HIPAA / GDPR audit-control evidence | Mandatory | Art. 6(1)(c) legal obligation |
| Non-repudiation of state changes for forensic + regulatory inquiry | Necessary for contracted SLA + incident response | Art. 6(1)(b) contract + 6(1)(c) legal |
| Tenant self-service forensic queries | Tenant-contracted | Art. 6(1)(b) contract |
| Auditor evidence export | Mandatory for SOC 2 + ISO 27001 + HIPAA OCR audits | Art. 6(1)(c) |
| DSR cascade execution + receipt | GDPR Art. 17 / KR PIPA Art. 36 / equivalent | Art. 6(1)(c) |
| Tamper-detection alarms | Operational integrity | Art. 6(1)(c) + 6(1)(f) |
| Marketing / non-audit secondary use | NOT a purpose | n/a |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | sign-off pending |
| Tenant representative (pre-GA sample) | Scheduled | DPA + retention defaults review |
| Data subjects (tenants' end-users) | Indirect via tenant onboarding notice | upstream-disclosure obligation |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Prior consultation (Art. 36) NOT triggered after mitigations | residual ≤ Medium |
| ops-security | YES | co-author of threat-model |
| Engineering (axis-audit-chain + every emitting µservice owner) | YES | caller-redaction contract enforced per Bominal ADR-0003 |
| External auditor | At first audit cycle | cross-references this DPIA |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary for purpose? | YES — non-repudiation cannot be achieved without durable + tamper-evident + signed record. |
| Less intrusive alternative? | Considered: hash-only audit (no payload). Rejected: tenants need payload to perform forensic queries; hash-only is insufficient. Mitigation: caller-redaction at emission; payload stays opaque to audit-chain; class declared at emit time. |
| Proportionate? | YES — payload size capped (1 MB default); caller-redaction; data-class declared; retention bounded. |
| Public / private interest? | YES — operational integrity + regulatory obligation. |
| Anonymised alternative? | PARTIAL — subject_hash (salted) is the pseudonymisation primitive; full anonymisation would defeat tenant forensic-query purpose. |
| Lawful basis | per §2.4. |
| Special-category basis | HIPAA TPO via §164.502(a); KR PIPA Art. 23(2) tenant consent. |
| Transfer basis | SCC-only; default residency. |
| Retention | Per-pack matrix in `policy/data-residency.md` + retention-matrix.yaml. |
| Rights | DSR cascade with chain-preservation; Art. 15/16/17/18/20/21 honoured. |

## Step 5 — Identify and assess risks

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | PII / PHI leakage via source µservice redaction failure (audit-chain receives unredacted) | M-H | H | **H** |
| R-02 | Cross-tenant query leak (Cedar misconfiguration) | L-M | H | **H** |
| R-03 | Long retention amplifies surveillance pattern of the chain | M | M | **M** |
| R-04 | Forensic query result inferred subject behaviour (timing + frequency patterns) | M | M | **M** |
| R-05 | Auditor exfiltration beyond engagement scope | L | H | **M** |
| R-06 | Cross-border misroute (audit data flows to wrong pack) | L | H | **M** |
| R-07 | DSR-cascade redaction loses non-redacted data still in proof artefacts | L | M | **L-M** |
| R-08 | HSM key compromise enables retroactive seal forgery | L | Critical | **M-H** |
| R-09 | Source µservice impersonation (T-S-01) → false audit attribution | M | H | **H** |
| R-10 | Subject re-identification via subject_hash + auxiliary tenant-side data (small tenant) | L | M | **L-M** |
| R-11 | Tenant operator misconfigures DPA → tenant's end-users uninformed | M | M | **M** |
| R-12 | Children's data (DPDPA 2023 §9; pack-in) inadvertently captured without parental-consent record | L | H | **M-H** |
| R-13 | Cryptography library supply-chain compromise affects verification correctness | L | H | **M** |
| R-14 | DSR not actionable because the data is also in a regulator-mandated retention window (e.g., HIPAA 6y locks erasure for 6y) | M | M | **M** |
| R-15 | Internal-malicious insider attempts mass-redaction to destroy evidence of prior action | L | Critical | **M-H** |

## Step 6 — Measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 PII/PHI leakage | Caller-redaction contract per Bominal ADR-0003; payload-class mandatory at emit; synthetic-PII detector scans pre-prod; DSR cascade for post-detection remediation; long-retention amplifies harm — mitigation is *prevention at source* not *cleanup downstream*. | M (engineering-discipline floor) | each emitting µservice owner |
| R-02 cross-tenant leak | Cedar default-deny; SPIFFE binding; server-side scope enforcement; LEAN lane; pen-test; threat-hunt. | L | ops-security |
| R-03 long-retention surveillance | Bounded retention per legal minima; DSR honoured; payload-class declares purpose; minimum-necessary applied within minima. | M (legal minima dominate) | council-privacy |
| R-04 inference via timing | Cardinality + frequency caps on cross-tenant queries; DP-noise on cross-tenant aggregates if ever offered. | L | axis-audit-chain |
| R-05 auditor exfiltration | Cedar `auditor-scope.cedar`; export bundles instead of raw access; TTL ≤ 4h; mTLS pin; every read audit-emitted. | L | ops-security + council-privacy |
| R-06 cross-border misroute | Pack-pinned at emission level; LEAN `cross-pack-replication-forbidden`; integration test catches misconfig. | L | axis-audit-chain |
| R-07 DSR proof-preservation | Soft-delete model: payload redacted but the leaf hash + Merkle proof of "this redacted at <ts> on dsr_id" preserved; cf. Bominal ADR-0028 §"Retention proof". GDPR Art. 17 + recital 65. | L | council-privacy + axis-audit-chain |
| R-08 HSM key compromise | HSM partition isolation; PKCS#11 access SPIFFE-bound; key never leaves HSM; rotation 90d with 24h overlap; KeyResolver maps period → public key so retroactive re-sign is rejected; quarterly rotation drill. | L (defence-in-depth across 4 controls) | ops-security + cloud-secrets |
| R-09 source impersonation | SPIFFE identity authoritative for source µservice; Cedar tenant-binding cross-check; per Bominal ADR-0003 §"Emission attribution". | L | ops-security + axis-audit-chain |
| R-10 subject re-identification | Salted hash + per-pack salt rotation 12mo; small-tenant cardinality limits. | L | ops-security |
| R-11 tenant DPA misconfig | DPA template mandates upstream disclosure; tenant onboarding checklist verifies. | L-M | council-privacy + gtm |
| R-12 children's data | Tenant DPA child-data clause; tenant DPA affirms age-gating; audit-chain doesn't gate by age — relies on tenant. | L (residual depends on tenant) | council-privacy |
| R-13 supply-chain | `cargo deny` + Cosign signed builds + reproducible builds + weekly RustSec advisory check; HSM mediates load-bearing crypto so a compromised pure-Rust crate cannot directly leak keys. | L | ops-security |
| R-14 retention-locked DSR | DSR cascade marks for redaction-at-retention-expiry; tenant + subject informed of the bounded delay; this is per HIPAA + GDPR-recital-65 lawful retention. | M (legal floor — irreducible) | council-privacy |
| R-15 insider mass-redaction | retention-cascade enforces declared policy; emergency redaction = 2-person rule + OpenBao JIT + audit-emit; redaction itself sealed in chain so attempts leave gaps. | L | ops-security + council-privacy |

## Step 7 — Sign-off

| Sign-off | Status | Signatory |
|---|---|---|
| DPO (council-privacy chair) | pending | TBA |
| ISO (ops-security chair) | pending | TBA |
| µservice owner (axis-audit-chain lead) | pending | TBA |
| Council-architecture chair | pending | TBA |

**DPO advice:** Residual risks after mitigations are L or M (no H or M-H remain after mitigations). Art. 36 prior consultation NOT triggered. Proceed to first-tenant onboarding subject to:
- Quarterly review of R-01 (caller-redaction discipline) and R-08 (HSM key-rotation drill).
- Annual review of this DPIA.
- Re-trigger DPIA on pack activation, HSM hardware change, or cryptography library upgrade.

## Per-Pack Overlay Sections

### pack-kr (Korea PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 require 개인정보영향평가 for sensitive-data-at-scale; this document fulfils. Additional KR considerations:
- PIPA Art. 23-2 sensitive cross-border forbidden — pack-pinning satisfies.
- PIPA Art. 28 storage period — retention matrix bounded.
- PIPA Art. 29 + 29-2 — technical safeguards + encryption; AES-256 + Ed25519 + HSM.
- KR 전자문서법 Arts. 5–7 — electronic document integrity, storage, verification — load-bearing; the chain is the implementation.
- PIPA Art. 36 — right to erasure → DSR cascade; recital-65-equivalent retention permitted.

### pack-us-healthcare (HIPAA)

- §164.308(a)(1)(ii)(A) — risk analysis; this document + threat-model.
- §164.312(b) — audit controls; this entire µservice IS the implementation.
- §164.312(c)(1) — integrity; Ed25519 + Merkle.
- §164.316(b)(2) — 6y retention; retention-cascade enforces.
- §164.404 — breach notification; integrated with `incident-response.md`.
- BAA per tenant — `legal/baa-template.md`.

### pack-eu (GDPR + EDPB + eIDAS + NIS2)

- EDPB Guidelines 4/2019 (Art. 25 by design): pseudonymisation + multi-tenancy + redaction-with-proof default.
- EDPB Recommendations 01/2020 (post-Schrems II): pseudonymisation + EU-pack KMS keys for SSE; supplementary measures documented.
- eIDAS 910/2014 Art. 26 (AdES): HSM-Ed25519 satisfies; load-bearing for cross-border transaction records.
- NIS2 (2022/2555): incident reporting timelines integrated.
- GDPR Art. 30 (records of processing): this µservice IS the platform-wide register backbone.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/audit-chain-dpia-overlay.md`. Each pack's local-law citations map to the same 7-step structure with substituted articles.

## Re-review Triggers

- Annually (Q2).
- New pack activation.
- HSM hardware change.
- Cryptography library upgrade (any of `ring`, `ed25519-dalek`, `sha2`).
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change (`legal/sub-processors.md`).
- Breach triggered.

## References

- Bominal ADR-0028 + ADR-0003 (inherited).
- ADR-0117 + ADR-0131 + ADR-0140.
- `microservices/audit-chain/threat-model.md` (paired).
- `microservices/audit-chain/policy/{seal-integrity, data-residency, tenant-scope, ci-scope, auditor-scope, public-read}.{md,cedar}`.
- `microservices/audit-chain/compliance.md`.
- ICO + CNIL DPIA methodology; PIPC Notice 2020-7.
- GDPR + EDPB; HIPAA; LGPD; DPDPA 2023.
