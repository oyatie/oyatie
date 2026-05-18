---
doc_class: DPIA
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: council-privacy + axis-foundry-evidence
reviewers: [council-privacy, dpo-eu, dpo-kr, ops-security]
related_adrs: [ADR-0024, ADR-0028, ADR-0117, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/foundry-evidence/PRD.md
  - microservices/foundry-evidence/policy/data-residency.md
  - microservices/foundry-evidence/policy/regulator-export-scope.cedar
  - microservices/foundry-evidence/threat-model.md
  - microservices/foundry-evidence/compliance.md
doc_status: published
---

# DPIA: foundry-evidence

## Section 1 — Identification

- **Controller** — oyatie (operator of foundry-evidence µservice on tenant's behalf for the duration of the tenant's contract).
- **Joint controllership** — none; oyatie is processor for tenant-originated personal data and controller only for internal operational metadata (`source_microservice`, `principal_spiffe_id`, SLI timings).
- **Sub-processors** — Oracle Cloud Infrastructure (compute + Postgres + S3 + Cloud-HSM, all delegated through audit-chain substrate); no per-pack-EU data leaves EU pack region per ADR-0117.
- **DPO contact** — `dpo-eu@oyatie.com` (pack-eu, pack-us-healthcare); `dpo-kr@oyatie.com` (pack-kr); per-pack DPO routing in `tenancy` µservice ROPA.

## Section 2 — Nature of processing

The foundry-evidence µservice processes per-capability-invocation evidence packs for Foundry agent runtime. Each pack typically contains:

| Data element | Source | Data class | Required for | Lawful basis |
|---|---|---|---|---|
| `invocation_id`, `agent_id`, `capability_id` | foundry-runtime | INTERNAL_ONLY | traceability | legitimate interest (controller's audit obligation) |
| `model_version`, `provider`, `request_token_count`, `response_token_count` | foundry-runtime | INTERNAL_ONLY | EU AI Act Art. 12 traceability | legal obligation (EU AI Act Art. 12) |
| `prompt_payload_sha`, `output_payload_sha` | foundry-runtime | INTERNAL_ONLY (hashes) | tamper detection | legitimate interest |
| Raw prompt + output text (content-addressed in WORM) | foundry-runtime | variable (declared by source); may be PII / PHI / PIPA Art. 23 / SECRET | regulator-grade forensic evidence on dispute | legal obligation (HIPAA §164.312(b); KR PIPA Art. 29; GDPR Art. 30) + tenant contract |
| `subject_hash` (if available from tenant-side hashing) | foundry-runtime | quasi-identifier | linking to tenant-side records during DSR | legitimate interest |
| `autonomy_tier_decision` | foundry-supervisor | INTERNAL_ONLY | EU AI Act Art. 26 + ADR-0024 | legal obligation |
| `guardrail_decisions[]` | foundry-guardrails | INTERNAL_ONLY | safety evidence | legitimate interest |
| `eval_verdict_at_invocation` | foundry-eval | INTERNAL_ONLY | ADR-0024 eval-evidence integration | legitimate interest |
| `tenant_id` | SPIFFE binding | INTERNAL_ONLY | tenant scope | contract |
| `principal_spiffe_id` | SPIFFE | INTERNAL_ONLY | accountability | legitimate interest |
| timestamps | system | INTERNAL_ONLY | ordering + retention | legitimate interest |

## Section 3 — Necessity and proportionality

### Necessity

- Without per-invocation evidence packs, oyatie cannot satisfy EU AI Act Art. 12 (logs of high-risk AI system operation), HIPAA §164.312(b) (audit controls), GDPR Art. 30 (records of processing), KR PIPA Art. 29 (safety measures), or SOC 2 CC4.x (system activity monitoring).
- Per ADR-0024 the eval-evidence join is mandatory at invocation time, not reconstructable later; without aggregator capture the evidence is irretrievable.
- Without `prompt_payload_sha` + `output_payload_sha`, tampered logs cannot be detected.

### Proportionality

- Raw prompt + output text is the most-sensitive surface. To minimise plaintext exposure:
  - Plaintext lives only in the audit-chain WORM blob (substrate-owned), addressed by `payload_sha`.
  - `foundry-evidence` index stores only the hash + per-class metadata.
  - Reads of plaintext are Cedar-gated on `payload_data_class ∈ principal.sensitive_data_entitlements`.
  - Tenant operators without sensitive-data entitlement see hash-only views.
- Subject-level identifiers in payloads are minimised to `subject_hash` (where the tenant supplies a hashing salt); cleartext PII / PHI in payloads is the tenant's responsibility and is gated identically.
- Retention is per-pack legal minimum (no excess) per `compliance.md`; cold archival cascades reduce hot-tier exposure.

### Alternatives considered

| Alternative | Rejected because |
|---|---|
| Do not retain prompt/output text | Defeats forensic forward-chain; cannot meet HIPAA §164.312(b) or EU AI Act Art. 12 evidence depth |
| Retain plaintext in foundry-evidence Postgres | Defeats minimisation; expands attack surface; audit-chain WORM is the appropriate substrate |
| Skip eval-verdict join | Defeats ADR-0024; cannot answer "was this invocation gated by a passing eval?" |
| Per-tenant local pack storage (tenant-controlled) | Defeats cryptographic chain continuity and substrate uniformity; available as tenant-controlled export instead |

## Section 4 — Data subject rights

| Right | How foundry-evidence honours it |
|---|---|
| Access (GDPR Art. 15 / KR PIPA Art. 35) | Via `evidence_query` API + tenant portal; tenant operator with subject-mapping entitlement queries `subject_hash`-scoped packs; cleartext mediated by tenant's own DSR workflow on tenant-controlled plaintext copies |
| Rectification (Art. 16) | Evidence packs are factual records of "what the agent did"; rectification of *facts* is not applicable; for incorrect tenant-controlled context, see `tenancy` µservice DSR cascade |
| Erasure (Art. 17 / KR PIPA Art. 36) | DSR cascade from `tenancy` → audit-chain substrate retention-cascade; foundry-evidence Postgres index rows redacted; payload-blob soft-deleted at substrate per audit-chain `RetentionApplied`; Merkle proof of redaction emitted |
| Restriction (Art. 18) | Tenant DSR sets `restricted=true` flag on subject_hash; queries filter accordingly; export endpoints refuse restricted packs |
| Portability (Art. 20) | Tenant-controlled export via `regulator-export` with `framework=portability`; bundle Ed25519-signed; receiving-side verifiable |
| Object (Art. 21) | Tenant-level objection raised via `tenancy`; legitimate-interest-based processing reviewed; controller may refuse where legal obligation overrides |
| Automated-decision (Art. 22) | Evidence packs CARRY the agent's decision rationale + autonomy-tier; tenant can review and request human-intervention via `governance` workflow; foundry-evidence emits an `Art22Reviewed` event back to chain |
| KR PIPA Art. 23 sensitive-info-handling | Per `policy/data-residency.md` §DR-04; sensitive-info packs gated by explicit consent entitlement at tenancy onboarding |

## Section 5 — Risks to data subjects

| Risk | Likelihood | Severity | Mitigation | Residual |
|---|---|---|---|---|
| Plaintext PII leak via cross-tenant read | L | High | Cedar tenant-scope + payload_data_class gate; LEAN `cross-tenant-leak-prevention` | L |
| Plaintext PHI leak via insufficiently-gated read | L | Critical | Sensitive-data entitlement issued at BAA signature only (pack-us-healthcare); Cedar enforces | L |
| Cross-pack residency violation | L | High | Pack-pinning in `policy/data-residency.md`; LEAN `cross-pack-replication-forbidden` | L |
| Re-identification via subject_hash + auxiliary data | L | Medium | salt rotation per-tenant; `subject_hash` not exposed in public-read transparency | L |
| Retention overrun (data kept past legal max) | L | Medium | per-pack retention cascade (audit-chain substrate); LEAN `retention-cascade-on-cadence` | L |
| Long-lived pre-signed URL leak | L | High | URLs ≤ 5 min TTL + IP-bound; audit-emitted on issuance | L |
| Inadvertent disclosure of T3 escalation rationale | L | Medium | autonomy_tier rationale is INTERNAL_ONLY; entitlement-gated | L |
| Regulator-export delivered to wrong tenant | L | Critical | 2-person rule; receiving-bucket bound to receiving-tenant SCC; audit-emitted | L |
| Plaintext exposure via OS-level log scraping | L | Critical | Plaintext never written to stdout/stderr or log files; LEAN `no-payload-in-logs` blocks debug-level logging of payload fields | L |
| Audit-of-audits creates unbounded recursion | M | Low | recursive emits bounded depth + flagged via `event_class=foundry.evidence.read.v1`; substrate emits without re-emitting | L |

## Section 6 — Cross-border transfer

- Pack-EU data: processed exclusively in EU region per ADR-0117; OCI EU regions only (Frankfurt, Madrid).
- Pack-KR data: KR region only; KR PIPA Art. 28 cross-border restriction honoured.
- Pack-US-healthcare data: US healthcare-region only; BAA-bound; no cross-region replication.
- Regulator-export bundles to non-pack regulator endpoints: TLS 1.3 + mTLS where supported; receiving-side SCC + DPA-recorded transfer mechanism on file before issuance; if regulator endpoint cannot accept SCC, export delivered via tenant-mediated bridge.

## Section 7 — Records of Processing Activities (ROPA) join

This µservice contributes to the tenant's ROPA per GDPR Art. 30:

- Processing activity: "Operate AI agent runtime; record per-invocation evidence; honour regulator audit requests."
- Categories of data: see Section 2.
- Categories of recipients: tenant operators (within DPA scope); regulators (per audit engagement); internal forensic teams (Cedar-gated).
- Retention: per pack legal minimum (see Section 5 + `compliance.md`).
- Security: cryptographic seal + WORM + Cedar default-deny + SPIFFE attestation; see `threat-model.md`.

## Section 8 — Review cadence

- Annual full review.
- Out-of-cycle review on: any change to data classes consumed; any new pack onboarded; any change to retention defaults; any change to lawful-basis posture.
- Sign-off: council-privacy chair + DPO of affected packs.

## Section 9 — Honesty annotation per ADR-0133

Per ADR-0133, every claim of mitigation effectiveness is tied to a CI gate or an explicit substrate runbook. The DPIA is itself audit-emitted on publication via `audit-chain` event `audit.dpia.published.v1`.
