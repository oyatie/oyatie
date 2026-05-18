---
id: ADR-RECORDINGS-0002
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: ops-compliance, council-privacy, axis-recordings, council-architecture
owner: ops-compliance
supersedes: []
superseded_by: []
related: [ADR-0117, ADR-0131, ADR-0133, ADR-RECORDINGS-0005]
related_artifacts:
  - microservices/recordings/PRD.md (FR-08, FR-10, FR-11 retention + legal-hold + ediscovery)
  - microservices/recordings/policy/cedar/legal-hold.cedar
  - microservices/recordings/policy/data-residency.md
  - microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml
  - microservices/recordings/slos/retention-policy-correctness.openslo.yaml
  - microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml
  - microservices/recordings/runbooks/legal-hold-court-order-receipt.md
  - microservices/recordings/runbooks/retention-policy-rollback.md
  - microservices/recordings/runbooks/ediscovery-export.md
purpose: |
  Fix the per-pack retention floor / ceiling defaults, legal-hold engagement
  semantics (load-bearing 100 % correctness), and eDiscovery export
  workflow. Aligned with SEC Rule 17a-4(f), FINRA Rule 4511, MiFID II
  Art. 16(7), HIPAA §164.530(j), KR 전자문서법 Art. 5, GDPR Art. 5(1)(e) +
  Art. 17, FRCP Rule 26(f)/34, Sedona Conference, ISO 27037:2012.
---

# ADR-RECORDINGS-0002: Retention + legal-hold policy — per-pack defaults + load-bearing 100 % correctness invariants

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings FR-08 + FR-10 + FR-11 mandate:
- per-tenant retention with per-pack default + tenant-tier override +
  legal-hold override
- legal-hold engagement as a **load-bearing** invariant
- eDiscovery export with chain-of-custody seal per FRCP / SEC / FINRA / KR

Pack scope spans EU GDPR storage-limitation, US HIPAA 6-year clinical
retention, US SEC 17a-4(f) 36-month non-erasable retention for recorded
financial communications, KR 전자문서법 long-term integrity-attested retention,
EU MiFID II 16(7) 5-year (extensible to 7y on request) financial-comm
retention, plus right-to-erasure cascades (GDPR Art. 17 + KR PIPA Art. 21
+ HIPAA + others).

Concrete tensions to resolve:

1. **Erasure-floor conflict**: GDPR Art. 17 erasure can conflict with HIPAA
   6y / SEC 17a-4 3y / FINRA 6y floor.
2. **Legal-hold-vs-purge race**: retention purge worker + legal-hold
   engagement on same scope must not both succeed.
3. **KMS-shred irreversibility**: once shred fires, the data is unrecoverable.
4. **eDiscovery scope leakage**: court-order scope must strictly match;
   under-collection misses evidence, over-collection violates Sedona.
5. **Engagement latency**: a court order that takes effect in minutes is a
   spoliation risk per FRCP 37(e); engagement must take effect within p99
   ≤ 1s.

## Decision

oyatie recordings ships a **per-pack retention model + load-bearing
legal-hold semantics + chain-of-custody-sealed eDiscovery export**:

### 1. Per-pack retention floor + ceiling

| Pack | Floor | Ceiling | Notes |
|---|---|---|---|
| pack-kr | 1y | 5y default / 7y configurable | KR PIPA Art. 21 + 전자문서법 |
| pack-eu | none (tenant) | 7y | GDPR Art. 5(1)(e) |
| pack-us | none (tenant) | 7y | varies state |
| pack-us-healthcare | 6y | 10y | HIPAA §164.530(j) |
| pack-us-financial | 3y (first 2y non-erasable) | 7y default / 10y configurable | SEC 17a-4(b)(4) + FINRA 4511 + MiFID II 16(7) |
| pack-jp | 2y | 7y | APPI |
| pack-sg | 1y | 7y | PDPA |
| pack-au | 7y | 10y | Privacy Act 1988 |
| pack-in | 3y | 7y | DPDPA 2023 |
| pack-br | 5y | 7y | LGPD |
| pack-ae | 1y | 7y | UAE PDPL |
| pack-ksa | 5y | 10y | PDPL + SAMA |

Tenant override allowed within `[floor, ceiling]`; outside the bounds is
refused by `oya-recordings-retention-policy-kernel`.

### 2. Legal-hold engagement (load-bearing 100 % correctness)

- Cedar PERMIT + four-eyes paired-approver required (`policy/cedar/legal-hold.cedar`).
- Court-order reference required (court_id + jurisdiction + case_no +
  order_date).
- Postgres advisory-lock on the hold scope; retention worker + KMS-shred
  worker observe the lock on next poll (max 100ms cycle).
- Audit-chain Ed25519 seal before the kernel returns success.
- **p99 ≤ 1s end-to-end** (load-bearing SLO).
- **No retention purge or KMS-shred on a held recording — ever** (load-
  bearing 100 % invariant).

### 3. Soft-grace before hard purge

7-day soft-grace period before any retention-purge fires hard. During the
grace window, `RetentionPurgePending` event is emitted; tenant can veto;
purge proceeds only at the end of the window if no veto.

### 4. KMS-shred ordering

KMS-shred fires only after:
(a) the retention purge has executed the soft-grace + hard-purge cycle,
(b) no legal-hold is engaged on the scope,
(c) audit-chain has sealed the purge event.

### 5. eDiscovery export workflow

Per `runbooks/ediscovery-export.md`:
- four-eyes paired-approver (compliance-officer × 2)
- Cedar PERMIT scope-match
- bundle includes media + transcript + redaction overlay + audit-chain
  seal + retention history + legal-hold history
- bundle signed by export-worker SPIFFE Ed25519
- Merkle root computed over the full bundle
- counsel verifies Ed25519 + Merkle root per ISO 27037:2012 §5.4

### 6. DSR Art. 17 cascade (right-to-erasure)

When erasure conflicts with retention floor:
- redact identifiers (handle → `«erased»`) via insert-only overlay row
- body remains in audit-protected form
- access bound to compliance-officer + four-eyes
- search index re-emit with redacted form
- audit-chain seal of the erasure event

### 7. Per-pack variants

- pack-us-financial: S3 object-lock (WORM) on the hot tier for first 2y;
  cold tier WORM for remaining retention.
- pack-us-healthcare: BAA-conditional ingest; PHI-aware Whisper redaction
  at transcription time.
- pack-eu: GDPR Art. 30 ROP entry on every hold + every export.
- pack-kr: 전자문서법 Art. 5 integrity-attestation on hold seal + export.
- pack-au: TIA Act + state Surveillance Devices Act consent verification.

## Alternatives Considered

### A. Per-tenant retention with no per-pack floor / ceiling

- Pros: maximum tenant flexibility.
- Cons: regulator audit fails (SEC 17a-4 floor not enforced; HIPAA 6y not
  enforced); compliance posture cannot be defended at scale.
- Rejected: regulators have hard floors; per-pack-default is the canonical
  enforcement layer.

### B. Eventual-consistency legal-hold (engage within 5 min)

- Pros: simpler implementation (batch worker; no advisory-lock).
- Cons: FRCP 37(e) spoliation risk; under-court-order delivery window of
  minutes, any in-flight purge in that window is a violation.
- Rejected: load-bearing 100 % + p99 ≤ 1s is the regulatory expectation.

### C. Hard-purge with no soft-grace

- Pros: simpler; matches GDPR storage-limitation cliff exactly.
- Cons: human-error blast-radius high (misconfigured retention policy → data
  loss); industry-leader practice is a grace window (Stripe / Linear /
  Palantir all emit a "pending" event).
- Rejected: soft-grace is the industry best practice.

### D. Built-in eDiscovery via cross-µservice call to a separate `ediscovery` µservice

- Pros: separation of concerns.
- Cons: recordings owns the chain-of-custody seal; moving the export bundle
  composition across a µservice boundary adds an extra audit-chain seal
  step + a residency concern (ediscovery µservice would have to be pack-
  pinned per residency, defeating the centralisation).
- Rejected: keep eDiscovery export inside recordings; cross-µservice handoff
  is only via Workflow events for downstream consumers.

### E. Per-tenant override of legal-hold semantics

- Pros: tenant flexibility.
- Cons: tenants cannot legally override court-order semantics; the
  load-bearing invariant cannot be negotiated.
- Rejected: invariant must hold for every tenant.

## Consequences

### Positive

- Court-order compliance verifiable + auditable.
- SEC 17a-4 WORM + HIPAA 6y + MiFID II 5y all natively supported.
- GDPR Art. 17 erasure works without breaking regulatory floors.
- Industry-leader-grade audit-chain Merkle seal on every export.

### Negative

- Postgres advisory-lock between retention + hold workers adds latency on
  contention (mitigated by < 100ms poll cycle).
- 7-day soft-grace means tenant can't immediately delete content (mitigated
  by tenant-tier-configurable, with starter-tier default 7d; pro-tier can
  override to immediate via explicit Cedar PERMIT).
- Load-bearing 100 % SLO is unforgiving; any process change must pass the
  retention-policy-correctness CI lane.

### Operational

- Cargo workspace adds `oya-recordings-retention-policy-*` (9 crates) +
  `oya-recordings-legal-hold-*` (9 crates) + `oya-recordings-ediscovery-*`
  (8 crates).
- CI: `retention-policy-correctness` + `legal-hold-chain-of-custody-
  correctness` are load-bearing 100 % lanes per ADR-0130.
- Per ADR-0130 SLO-gated promotion: legal-hold-engagement-latency.openslo.yaml
  gates every release.

### Regulatory

- **SEC Rule 17a-4(f)**: pack-us-financial first 2y non-erasable via S3
  object-lock (WORM).
- **FINRA Rule 4511**: 6y retention default.
- **MiFID II Art. 16(7)**: 5y default; on-request extension to 7y.
- **HIPAA §164.530(j)**: 6y clinical floor on pack-us-healthcare.
- **GDPR Art. 5(1)(e) + Art. 17**: storage-limitation + right-to-erasure
  with conflict-resolution per Section 6.
- **KR 전자문서법 Art. 5**: long-term integrity attestation via audit-chain
  Merkle seal.
- **FRCP Rule 26(f), Rule 34, Rule 37(e)**: spoliation-safe via p99 ≤ 1s
  engagement.
- **ISO 27037:2012**: chain-of-custody seal on eDiscovery export.
- **Sedona Conference**: scope strict matching at export time.

## References

- SEC Rule 17a-4(f); FINRA Rule 4511; MiFID II Art. 16(7); CFTC Rule 1.31.
- HIPAA 45 CFR §§164.502, 164.530(j); HITECH Act 13402.
- GDPR Arts. 5, 17, 30, 33, 34; ePrivacy Directive 2002/58 Art. 5(3).
- KR PIPA Art. 21, Art. 28, Art. 29; KR 전자문서법 Arts. 5, 6; KR 통신비밀보호법.
- APPI Arts. 17, 18; PDPA 2012; Privacy Act 1988 (AU); DPDPA 2023; LGPD
  Arts. 6, 15, 18; UAE PDPL; KSA PDPL + SAMA.
- FRCP Rule 26(f), Rule 34, Rule 37(e).
- Sedona Conference Commentary on Legal Holds + Commentary on Achieving
  Quality in the eDiscovery Process.
- ISO 27037:2012 — guidelines for identification, collection, acquisition,
  preservation of digital evidence.
- NIST SP 800-86 — guide to integrating forensic techniques into incident
  response.
- ADR-0117, ADR-0131, ADR-0133.
- ADR-RECORDINGS-0005 — storage tiering aligned with retention.
- `microservices/recordings/policy/data-residency.md`.
- `microservices/recordings/policy/cedar/legal-hold.cedar`.
- `microservices/recordings/runbooks/{legal-hold-court-order-receipt, retention-policy-rollback, ediscovery-export}.md`.
- `microservices/recordings/slos/{legal-hold-engagement-latency, retention-policy-correctness, legal-hold-chain-of-custody-correctness}.openslo.yaml`.
