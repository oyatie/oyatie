---
id: ADR-RECORDINGS-0005
status: Accepted
date: 2026-05-17
microservice: recordings
deciders: axis-recordings, ops-sre-reliability, ops-finops, council-privacy
owner: axis-recordings
supersedes: []
superseded_by: []
related: [ADR-0117, ADR-0131, ADR-RECORDINGS-0002, ADR-RECORDINGS-0004]
related_artifacts:
  - microservices/recordings/PRD.md (Open Question 3 — hot/cold tiering)
  - microservices/recordings/policy/data-residency.md
  - microservices/recordings/cost-budget.md
  - microservices/recordings/capacity-model.md
purpose: |
  Fix the storage tier model: hot S3 for active access + cold S3-Glacier-
  class for retention-aged content + per-pack age-down policy. Pack-us-
  financial: WORM-class for SEC 17a-4 first-2y requirement. Aligned with
  SEC Rule 17a-4(f), HIPAA, KR 전자문서법 retention-with-integrity-attestation.
---

# ADR-RECORDINGS-0005: Storage substrate tiered — hot S3 + cold S3-Glacier-class + per-pack age-down + WORM where required

## Status

Accepted — 2026-05-17.

## Context

PRD-recordings demands durable storage for every recording across an
archive that can grow to 50 PB / pack and 500 PB / pack for cold tier per
`capacity-model.md`. Per ADR-RECORDINGS-0002, retention floors span 1y
(pack-kr / pack-ae) to 10y (pack-us-healthcare ceiling); SEC 17a-4(f)
mandates non-erasable WORM-class for the first 36 months of pack-us-
financial recordings.

Storage tier choices:

- **S3 Standard** ($0.023/GB/mo; ms-latency; high IOPS).
- **S3 Intelligent-Tiering** (auto-tier; modest savings; no WORM).
- **S3 Glacier Instant Retrieval** ($0.004/GB/mo; ms-latency; 90-day min
  storage; archive-class).
- **S3 Glacier Flexible Retrieval** ($0.0036/GB/mo; minute-latency; 90-day
  min; cheap archive).
- **S3 Glacier Deep Archive** ($0.00099/GB/mo; 12-h retrieve SLA; 180-day
  min).
- **Cross-cloud cold** (Azure Archive, GCS Coldline) — adds complexity
  without clear pack-residency benefit.
- **S3 object-lock (Governance / Compliance mode)** — WORM-class; required
  for SEC 17a-4(f) pack-us-financial.

Tiering policy choices:

- **Per-recording age-down based on last-access** — flexible but
  unpredictable cost.
- **Per-pack age-down based on age-since-ingest** — predictable; aligns
  with retention floor logic.
- **Per-tenant override** — gives flexibility but complicates ops.

## Decision

oyatie recordings ships a **hot S3 Standard + cold S3 Glacier Instant
Retrieval + per-pack age-down + S3 object-lock for pack-us-financial first
2y**:

1. **Hot tier**: S3 Standard for active recordings (≤ pack-defined hot-tier
   retention).
2. **Cold tier**: S3 Glacier Instant Retrieval (millisecond-class retrieve
   for compliance investigations; meets ediscovery export SLAs at p99 ≤ 1h
   per 100 hours of media).
3. **Per-pack age-down policy**:
   | Pack | Hot retention | Cold tier engaged after |
   |---|---|---|
   | pack-kr | 90 days | day 90 |
   | pack-eu | 90 days | day 90 |
   | pack-us | 90 days | day 90 |
   | pack-us-healthcare | 1 year (clinical frequent-access) | year 1 |
   | pack-us-financial | first 2 years in hot (SEC 17a-4 non-erasable, object-lock COMPLIANCE mode) | year 2 |
   | pack-jp | 90 days | day 90 |
   | pack-sg | 90 days | day 90 |
   | pack-au | 90 days | day 90 |
   | pack-in | 90 days | day 90 |
   | pack-br | 90 days | day 90 |
   | pack-ae | 90 days | day 90 |
   | pack-ksa | 90 days | day 90 |
4. **S3 object-lock (COMPLIANCE mode)** for pack-us-financial first 2y;
   prevents deletion even by root account; satisfies SEC 17a-4(f) "non-
   erasable, non-rewriteable" requirement.
5. **Cross-region replication**: within-pack DR-pair only (per
   `multi-region.md`); cross-pack replication forbidden.
6. **KMS-shred on retention expiry** (per ADR-RECORDINGS-0002): the
   tenant-DEK envelope-encrypts the recording at ingest; on retention
   expiry, the KMS key is shredded, rendering the object cryptographically
   unrecoverable even if the S3 object somehow survived.
7. **Hot-tier object-lock**: Governance mode by default (allows root
   override under audit-chain seal for emergency recovery); COMPLIANCE mode
   only for pack-us-financial first 2y.
8. **Tenant-tier override**: pro+ tenants may extend hot-tier retention
   (e.g., for clinical-review patterns where frequent access exceeds
   pack-default 90 days); refused if tenant-tier doesn't support it.
9. **Glacier Deep Archive** is **NOT** used by default; the 12h retrieve
   SLA breaks ediscovery export SLAs. Tenant-tier ≥ enterprise can opt in
   for deep-cold ≥ 7y retention.

## Alternatives Considered

### A. S3 Standard for everything (no tiering)

- Pros: simpler ops; instant retrieve for everything.
- Cons: cost runs 5-10× higher at the 50 PB cold-tier scale; ops-finops
  unhappy.
- Rejected.

### B. S3 Intelligent-Tiering

- Pros: auto-tier; no policy management.
- Cons: no WORM; cost-predictability worse; pack-us-financial cannot use.
- Rejected; manual per-pack age-down is more predictable.

### C. Glacier Deep Archive as cold tier

- Pros: cheapest ($0.00099/GB/mo).
- Cons: 12h retrieve SLA breaks ediscovery export SLAs; legal-hold
  engagement on cold-tier rows would face 12h retrieve before any export
  bundle can be assembled.
- Rejected for default; optional for tenant-tier ≥ enterprise.

### D. S3 object-lock GOVERNANCE mode for pack-us-financial first 2y

- Pros: allows admin override.
- Cons: SEC 17a-4(f) requires "non-erasable, non-rewriteable" — Governance
  mode allows override by privileged principals, which may not satisfy
  "non-erasable" under strict interpretation. SEC has clarified that
  COMPLIANCE mode is the safe choice.
- Rejected; COMPLIANCE mode is canonical for pack-us-financial.

### E. Cross-cloud cold (Azure Archive, GCS Coldline)

- Pros: cross-cloud redundancy.
- Cons: per-pack residency complicates; ops complexity grows; no clear
  benefit at oyatie's hyperscaler-comparator scale where OCI is the
  primary substrate.
- Rejected.

## Consequences

### Positive

- Cost-efficient at 50 PB cold-tier scale (≈ 80 % savings vs. all-hot).
- SEC 17a-4(f) WORM-class satisfied via object-lock COMPLIANCE mode.
- HIPAA + KR 전자문서법 retention floors honoured via per-pack age-down.
- KMS-shred on retention expiry gives cryptographic-erasure guarantee
  independent of S3 object lifecycle.
- Per-pack lifecycle policy is explicit and auditable.

### Negative

- Two tier classes to operate (hot + cold) per pack.
- pack-us-financial first 2y costs 5× more per byte than other packs
  (mitigated by tenant-pricing premium for financial tier).
- Cold-tier retrieve latency p99 is ~1s vs. hot-tier ~50ms; ediscovery
  export jobs must factor in retrieve cost.

### Operational

- IaC: per-pack S3 bucket policies pin Object-Lock + age-down rules per
  `iac/kustomize/overlays/pack-*`.
- CI lane `oya-check-recordings-pack-residency` asserts S3 bucket region
  matches pack region.
- CI lane `oya-check-recordings-tier-policy` asserts pack-us-financial
  hot-tier objects have ObjectLock=COMPLIANCE.
- KMS rotation policy: tenant-DEK rotated annually; KMS-shred on retention
  expiry per ADR-RECORDINGS-0002.

### Regulatory

- **SEC Rule 17a-4(f)(2)** — non-erasable, non-rewriteable: COMPLIANCE
  object-lock satisfies.
- **SEC 17a-4(b)(4)** — 3-year retention: pack-us-financial floor.
- **FINRA Rule 4511** — 6-year books + records: ceiling default.
- **MiFID II Art. 16(7)** — 5-year retention; on-request extension to 7y:
  configurable.
- **HIPAA §164.530(j)** — 6-year clinical retention: pack-us-healthcare
  floor.
- **KR 전자문서법 Art. 5** — long-term integrity attestation: KMS-shred +
  audit-chain Merkle seal satisfies.
- **GDPR Art. 5(1)(e)** — storage-limitation: per-pack ceiling enforces.
- **GDPR Art. 17** — right-to-erasure: KMS-shred satisfies cryptographic-
  erasure expectation per Bominal ADR-0028 inheritance.

## References

- SEC Rule 17a-4(f); SEC Rule 17a-4(b)(4).
- FINRA Rule 4511.
- MiFID II Art. 16(7).
- HIPAA 45 CFR §164.530(j).
- GDPR Arts. 5(1)(e), 17.
- KR 전자문서법 Art. 5.
- AWS S3 Glacier Instant Retrieval documentation.
- AWS S3 object-lock COMPLIANCE mode documentation.
- AWS KMS cryptographic-erasure pattern.
- ADR-0117, ADR-0131, ADR-RECORDINGS-0002 (retention floors), ADR-RECORDINGS-0004.
- microservices/recordings/cost-budget.md.
- microservices/recordings/capacity-model.md.
- microservices/recordings/policy/data-residency.md.
- microservices/recordings/multi-region.md.
