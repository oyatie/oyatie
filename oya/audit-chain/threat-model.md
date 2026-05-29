---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-audit-chain + ops-security
deciders: council-architecture, ops-security, axis-audit-chain, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0003, ADR-0056, ADR-0105, ADR-0117, ADR-0123, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/audit-chain-merkle-ed25519.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every HSM/cryptography library upgrade + after any verification-failed Sev-1 incident
enforced_frameworks:
  - "SOC 2 Type 2: CC4.1, CC4.2, CC6.1, CC6.2, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.27, A.5.28, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.24, A.8.25, A.8.28"
  - "GDPR Arts. 5, 17, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.5-§2.12", "KR PIPA Arts. 23/28/29/29-2/34", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b) (audit controls)", "§164.312(c)(1) (integrity)", "§164.316(b)(2) (6y retention)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35", "eIDAS 910/2014 Art. 26 (AdES)", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 20/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §8-10"]
  pack-br: ["LGPD Arts. 6 + 33 + 46"]
  pack-ae: ["UAE PDPL Art. 5/6/9/15"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: audit-chain µservice

## Purpose

Identify, classify, and mitigate threats to the audit-chain µservice's confidentiality, integrity, and availability. Because **this µservice is the evidence backbone for every other µservice's compliance posture**, a compromise here cascades to every product's audit trail. A successful tamper of audit-chain would: (a) invalidate every SOC 2 / ISO 27001 / GDPR / HIPAA / KR-ISMS-P claim oyatie makes; (b) provide a forged-event vector for any internal-or-external attacker; (c) cripple every µservice's promotion-readiness lane (which writes audit-emitted verdicts). This is therefore the highest-severity threat model in oyatie; tamper-detection time is the load-bearing operational metric.

## Scope

### In-scope

All components introduced by this phase + the existing crates `oya-audit-chain-{domain,file-adapter,usecase}` referenced as upstream (their physical migration is owned by a sibling phase):

| Layer-A (adopted OSS / managed) | Layer-B (oyatie-owned) |
|---|---|
| Postgres (HA primary + replica) | `oya-audit-chain-emission-*` (8 crates) |
| S3-compatible object storage (WORM-locked via Object Lock Compliance mode) | `oya-audit-chain-sealing-*` (10 crates) |
| OCI Cloud-HSM (per-pack signing-key partition) | `oya-audit-chain-verification-*` (7 crates) |
| Grafana Mimir (root-publication channel — co-owned with observability µservice) | `oya-audit-chain-query-*` (8 crates) |
| Cedar policy evaluator | `oya-audit-chain-retention-cascade-*` (6 crates) |

### Out-of-scope

- Threats to the underlying Kubernetes cluster — owned by `cloud-k8s`.
- Threats to OCI Cloud-HSM hardware itself — owned by `cloud-secrets` µservice's threat model; this document inherits Cloud-HSM threats as upstream.
- Threats to the source µservices that emit (each owns its own threat model for what it emits; audit-chain only owns what it accepts + seals).
- Threats to the `observability` µservice (sibling threat model; root-publication via Mimir crosses both surfaces and is therefore co-mitigated).

## Trust Boundaries

```text
┌─ Internal mesh ─────────────────────────────────────────────────────────────┐
│                                                                             │
│  Every workload µservice                                                    │
│      │                                                                      │
│      │ (mTLS; SPIFFE identity; per-µservice OTel API key NOT used here)     │
│      ▼                                                                      │
│  ┌─ Trust boundary 1: workload µservice → audit-chain-emission-rest ─┐      │
│  │  - mTLS                                                            │      │
│  │  - SPIFFE identity validation                                      │      │
│  │  - Per-tenant Cedar scope                                          │      │
│  └────────────────────────────────────────────────────────────────────┘      │
│                              │                                              │
│  ┌─ audit-chain pods ───────────────────────────────────────────────┐       │
│  │  emission-rest (stateless; horizontally scaled)                   │       │
│  │     │ writes to                                                   │       │
│  │     ▼                                                             │       │
│  │  Trust boundary 2: emission-rest → durable WAL                    │       │
│  │     ▼                                                             │       │
│  │  Postgres (per-pack primary + replica)                            │       │
│  │     │  (events index + SealRecord; never UPDATE/DELETE except    │        │
│  │     │   via retention-cascade RPC)                                 │       │
│  │     ▼                                                             │       │
│  │  Trust boundary 3: WAL → sealing-worker (leader-elected)          │       │
│  │     ▼                                                             │       │
│  │  sealing-worker builds Merkle tree per (tenant, period)           │       │
│  │     │                                                             │       │
│  │  Trust boundary 4: sealing-worker → OCI Cloud-HSM                 │       │
│  │     │ (PKCS#11 / KMIP; per-pack partition; SPIFFE-bound)          │       │
│  │     ▼                                                             │       │
│  │  HSM signs root with Ed25519 private key (private key never       │       │
│  │  leaves HSM; signing call is remote)                              │       │
│  │     ▼                                                             │       │
│  │  Trust boundary 5: sealing-worker → S3 WORM bucket                │       │
│  │     │  (Object Lock Compliance mode; immutable for retention)     │       │
│  │     ▼                                                             │       │
│  │  Trust boundary 6: sealing-worker → Mimir root-publication        │       │
│  │     │  (writes oya_audit_chain_root_* metrics to oya-self +       │       │
│  │     │   oya-aggregate tenants)                                    │       │
│  │     ▼                                                             │       │
│  │  Mimir + GitHub-pinned manifest                                   │       │
│  └───────────────────────────────────────────────────────────────────┘       │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘

┌─ External readers ─────────────────────────────────────────────────────────┐
│                                                                            │
│  Tenant operator / customer app / auditor                                  │
│      │                                                                      │
│  Trust boundary 7: external → query-rest                                   │
│      │ (OIDC + Cedar + per-tenant scope; auditor JIT tokens)               │
│      ▼                                                                      │
│  query-rest serves AuditEvent lookups + signed export bundles              │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Seven trust boundaries:
1. **Workload → emission-rest** (mTLS + SPIFFE).
2. **emission-rest → durable WAL** (Postgres write-only-append).
3. **WAL → sealing-worker** (leader-elected; pulls events for sealing).
4. **sealing-worker → HSM** (the load-bearing cryptographic boundary).
5. **sealing-worker → S3 WORM** (immutability boundary).
6. **sealing-worker → Mimir root-publication** (external-observability boundary; tamper-detection signal).
7. **External → query-rest** (tenant + auditor reads).

## Assets & Data Classification

Per Bominal ADR-0028 + the `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| AuditEvent records (per-event payload) | `AUDIT` + variable (per source event's `data_class`) | High | per-pack matrix (HIPAA 6y; KR PIPA 3y; pack-default 2y) | Postgres index + S3 WORM raw blob |
| SealRecord (Merkle root + signature + signer key handle) | `AUDIT` | Critical (load-bearing for non-repudiation) | indefinite for the period; never deleted | Postgres + S3 WORM |
| Merkle proofs (per-event inclusion proofs) | `AUDIT` | High | Derived from tree on demand; tree retained for retention window | computed from S3 WORM |
| HSM signing private keys | `SECRET` (HSM-bound — never appears outside HSM) | Critical | rotated 90d; rotation overlap 24h; old keys retained for verification | OCI Cloud-HSM partition |
| HSM signing public keys | `INTERNAL_ONLY` | Medium | retained across all rotation epochs (KeyResolver maps period → public key) | Postgres + S3 + GitHub-pinned manifest |
| Pack-scoped tenant partitioning salt | `SECRET` | Critical | rotated 12mo; rotation logged on-chain | OpenBao |
| RetentionPolicy (per-pack matrix) | `INTERNAL_ONLY` | Medium | git-versioned + Postgres replica | `microservices/audit-chain/policy/retention-matrix.yaml` + Postgres |
| DSR cascade tokens (subject_hash, dsr_id) | `PII_QUASI_IDENTIFIER` (subject_hash) + `AUDIT` | High | retained for the audit window | Postgres + S3 |
| Auditor engagement tokens | `SECRET` | Critical | TTL ≤ 4h; non-renewable without OpenBao + ops-security | OpenBao |
| Mimir root-publication series (`oya_audit_chain_root_*`) | `AUDIT` | High | aligned with Mimir retention | Mimir (`oya-self` + `oya-aggregate` tenants) |
| GitHub-pinned root manifest | `AUDIT` | High | indefinite; git history is the tamper-evident record | repo `evidence/audit-chain-roots/<pack>/<epoch>.json` |
| Cedar policy fragments | `INTERNAL_ONLY` | Medium | git history | `microservices/audit-chain/policy/*.cedar` |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| Workload µservice (emitter) | Semi-trusted internal | mTLS + SPIFFE | call `emit(event)` with own-tenant + own-µservice-source binding |
| audit-chain emission-rest | Trusted internal | SPIFFE | write to WAL |
| audit-chain sealing-worker | Trusted internal | SPIFFE; HSM-handle-bound | read WAL; build Merkle tree; sign root via HSM |
| audit-chain verification-rest | Trusted internal | SPIFFE (read-only roots + proofs) | serve verify reads |
| audit-chain query-rest | Trusted internal | OIDC + Cedar | serve tenant + auditor reads |
| Tenant operator (human) | Untrusted external | OIDC + MFA | read own tenant's audit events; verify own proofs |
| Customer application (machine) | Untrusted external | Per-tenant API key | call `emit` only via tenant's tenancy-µservice-mediated proxy (no direct external emit) |
| External auditor | Read-only external on time-boxed window | OIDC + MFA + JIT short-lived auditor token via OpenBao | read scoped-tenant subset; signed export bundle |
| OCI Cloud-HSM operator (Oracle) | Trusted external (vendor) | OCI IAM + dedicated partition | host HSM partition; oyatie owns key material via per-partition isolation; Oracle cannot read |
| ops-security operator (human) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao + 2-person rule for HSM admin ops | rotate keys; export evidence bundles; emergency redact |
| council-privacy operator (human) | Trusted internal | OIDC + MFA | DSR initiation + receipt of receipt |
| Attacker — opportunistic | Untrusted | none | scans + low-skill exploitation |
| Attacker — targeted (nation-state-level on the audit signal) | Untrusted | none | attempt verification-bypass; HSM-key extraction; supply-chain on cryptography lib |
| Insider — accidental | Trusted internal | OIDC | misconfigure retention policy or accidentally trigger DSR cascade (mitigated by 2-person rule for retention modifications) |
| Insider — malicious | Trusted internal | OIDC | worst-case actor for non-repudiation; the entire model exists to make a single insider unable to tamper without detection |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Workload µservice emits AuditEvent claiming foreign tenant_id**
- Asset: emission boundary; tenant attribution
- Likelihood: M / Impact: H (corrupts tenant's audit trail; false-flag attribution) / Risk: **H**
- Mitigations:
  - emission-rest validates SPIFFE identity → µservice_id binding; SPIFFE identity is the source of truth, NOT the tenant_id field in the event payload.
  - tenant_id in payload is cross-checked against the SPIFFE-resolved µservice's permitted tenant set via Cedar policy (`policy/tenant-scope.cedar`); mismatch → 403 + emit `audit_chain_tenant_spoofing_attempt_total`.
  - Per Bominal ADR-0003 §"Emission-source attribution": every event envelope carries the SPIFFE identity as the canonical source attribute, not the payload's source claim.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29

**T-S-02 — Attacker forges HSM signing call (impersonates sealing-worker)**
- Asset: HSM signing authority
- Likelihood: L / Impact: Critical (forged root signature; chain integrity destroyed) / Risk: **H**
- Mitigations:
  - HSM partition access is SPIFFE-identity-bound at the OCI Cloud-HSM IAM level; only the sealing-worker SA can request signing.
  - PKCS#11 session authentication via short-lived (≤24h) certificates rooted in OpenBao; certificates carry sealing-worker SPIFFE identity.
  - Per-call HSM audit log (Oracle-managed; oyatie reads as evidence); HSM-side audit emits independent stream cross-checked against on-chain SealRecord.
  - Network policy: only sealing-worker pods may reach HSM endpoint (Kubernetes NetworkPolicy + Istio AuthorizationPolicy).
  - Anomaly alert: signing-rate per partition deviates from sealing-cadence-expected ⇒ Sev-1 page.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.17, A.8.5, A.8.7, A.8.24; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29-2; HIPAA §164.312(a)(1)+(c)(1); eIDAS Art. 26 (AdES)

**T-S-03 — Attacker impersonates auditor JIT token**
- Asset: auditor read scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Auditor JIT tokens scoped per-(framework, tenant-subset, time-window) via Cedar (`policy/auditor-scope.cedar`).
  - mTLS pinning to auditor firm's gateway during engagement.
  - Every auditor read is itself audit-emitted (audit-of-audits).
  - Token TTL ≤ 4h; non-renewable without ops-security re-issue.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC8.1; ISO 27001 A.5.15, A.5.17, A.5.18; GDPR Art. 28; HIPAA §164.308(a)(4)(ii)(B)

**T-S-04 — Sealing-worker SPIFFE identity spoofing (attacker stands up workload with same SA name)**
- Asset: sealing-worker authority
- Likelihood: L / Impact: Critical / Risk: **M-H**
- Mitigations:
  - SPIFFE identity includes pod-identity binding (workload UUID + namespace + SA); spoofed workload in same namespace blocked by Kubernetes admission-controller policy.
  - OPA Gatekeeper / Kyverno rule: only the audit-chain operator may deploy pods claiming the sealing-worker SA.
  - HSM-side admission: PKCS#11 session must present a certificate signed by OpenBao's intermediate; OpenBao refuses to issue without the right cluster-admission-checked SVID.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.5, A.8.7; GDPR Art. 32(1)(b)

**T-S-05 — Replay of valid emission to fabricate duplicate audit-trail**
- Asset: emission idempotency
- Likelihood: M / Impact: M (duplicate events corrupt forensic queries) / Risk: **M**
- Mitigations:
  - emit() requires a caller-supplied `idempotency_key` (ULID or equivalent); emission-rest rejects duplicates within a 24h window (caller's UUID + content_sha is the primary index).
  - Per Bominal ADR-0003 §"Idempotency": idempotency-key + content_sha is the dedupe primitive.
  - Mimir metric `audit_chain_idempotent_reject_total` exposes attempt rate.
- Owner: axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.27, A.8.32; GDPR Art. 5(1)(d) accuracy

### Tampering (T)

**T-T-01 — Tampering of an unsealed AuditEvent in WAL before sealing**
- Asset: Postgres WAL between emission and sealing
- Likelihood: L (Postgres direct write requires DB superuser; not given to humans by default) / Impact: H / Risk: **M**
- Mitigations:
  - Postgres connection roles: emission-rest uses `audit_emitter` role with INSERT-only on the `events` table; no UPDATE/DELETE.
  - sealing-worker uses `audit_sealer` role with SELECT + INSERT on `sealed_periods`; no UPDATE on `events`.
  - DB superuser access is ops-security JIT only (2-person rule).
  - Postgres audit-log (pgaudit) captures every connection + statement; cross-checked against expected role traffic.
  - WAL → sealing-worker pipeline is one-way; sealing-worker SELECTs an event, computes the leaf, INSERTs the SealRecord; no reverse path.
  - Periodic integrity check: sealing-worker re-reads its own WAL inputs from S3 raw-blob storage before computing the Merkle leaf (defense-in-depth — S3 WORM blob is the source of truth, Postgres is the index).
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.3, A.8.5; GDPR Art. 32(1)(b); HIPAA §164.312(c)(1)

**T-T-02 — Tampering of the Merkle tree blob in S3 (corrupt or attacker-substituted)**
- Asset: S3 WORM raw-blob storage
- Likelihood: L (S3 Object Lock Compliance mode prevents writes after creation) / Impact: Critical (forged tree could verify against a forged signature; if attacker controls both, chain integrity destroyed) / Risk: **M-H**
- Mitigations:
  - S3 Object Lock in Compliance mode (write-once-read-many; retention ≥ pack retention window; cannot be shortened by anyone — including the bucket owner).
  - Bucket policy: deny `s3:DeleteObject` + `s3:PutObjectRetention` for everyone except a 2-person-rule break-glass principal.
  - Block-level SHA-256 stored in Postgres index; periodic Mimir-rule-driven `block_sha_mismatch_total` cross-check.
  - SSE-KMS encryption with per-pack KMS key; key access logged.
  - Server-side validation: sealing-worker reads back the blob it just wrote, recomputes the SHA, verifies match; mismatch quarantines + Sev-1 page.
  - **The Merkle root signed by the HSM commits to the tree's content; tampering the tree without re-signing makes the signature invalid (verification catches it deterministically).** This is the load-bearing mitigation per Bominal ADR-0028.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.8.11, A.8.12, A.8.24, A.8.25; GDPR Art. 32(1)(a)(b); HIPAA §164.312(c)(1); KR 전자문서법 Art. 5

**T-T-03 — Tampering of the GitHub-pinned root manifest**
- Asset: `evidence/audit-chain-roots/<pack>/<epoch>.json` repo file
- Likelihood: L (branch-protection on repo; signed commits) / Impact: H (false root reference) / Risk: **M**
- Mitigations:
  - branch-protection.yaml enforces signed commits + linear history + PR review + CODEOWNERS scoped to ops-security + axis-audit-chain on these paths.
  - Per-publication LEAN check: the published manifest's root + signature must match the latest SealRecord in Postgres at the time of publication.
  - The manifest is itself just a mirror; the canonical source is S3 WORM + Postgres; tampering the manifest doesn't tamper the chain — it gets detected at next-verifier read.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.32, A.8.32; GDPR Art. 32(1)(b)

**T-T-04 — Substitution of the published Mimir root series**
- Asset: Mimir `oya_audit_chain_root_*` metric series
- Likelihood: L / Impact: M (false root visibility; would be caught by S3 + GitHub cross-check) / Risk: **L-M**
- Mitigations:
  - Mimir multi-tenancy: only sealing-worker SA can write `oya_audit_chain_root_*` (per `policy/ci-scope.cedar` cross-references).
  - Three-channel cross-publication: S3 + Postgres + Mimir + GitHub-pinned manifest — tampering one without the others is detectable.
  - Recording rule `oya:root_publication_cross_check:rate` alerts on divergence.
- Owner: axis-audit-chain + axis-observability
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.8.15, A.8.16; GDPR Art. 32(1)(b)

**T-T-05 — Re-sign attack: attacker uses compromised HSM key to retroactively re-sign a forged tree**
- Asset: HSM signing-key + chain validity
- Likelihood: L / Impact: Critical / Risk: **M-H**
- Mitigations:
  - HSM keys never leave the HSM partition; "compromise" requires physical or vendor-supply-chain compromise (mitigated upstream by `cloud-secrets`).
  - **Key-rotation overlap**: every 90d a new key is created; the new key signs alongside the outgoing key for 24h; the KeyResolver maps `(period → key)`; verification refuses a signature from a key not active at that period. Thus a stolen current key cannot retroactively re-sign past periods.
  - Per Bominal ADR-0028 §"Chain-of-trust on rotation": every key rotation event is itself sealed by both keys; the chain-of-trust is auditable.
  - Public-key transparency: every rotation is published to the GitHub-pinned manifest + Mimir; tenant verifiers can detect a key being used outside its declared epoch.
- Owner: ops-security + axis-audit-chain
- Residual: L (defence-in-depth across 4 controls)
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.1; ISO 27001 A.5.17, A.8.5, A.8.7, A.8.24; GDPR Art. 32(1)(b)(c); eIDAS Art. 26 (AdES)

**T-T-06 — Tampering of retention-cascade decisions to silently delete events**
- Asset: retention-cascade authority
- Likelihood: L / Impact: H (would let an insider make events disappear) / Risk: **M**
- Mitigations:
  - retention-cascade-worker enforces declared policy from `policy/retention-matrix.yaml` (git-versioned; PR-reviewed).
  - Every retention application emits a `RetentionApplied` event WHICH IS ITSELF SEALED INTO THE CHAIN; deletion is recorded.
  - Per Bominal ADR-0028 §"Retention proof": every retention event carries a Merkle proof of the deleted event's presence at the moment of deletion (so verifiers can prove "this event existed and was deleted on this date").
  - Soft-delete is the default (mark for redaction; preserve the leaf but redact the payload); hard-delete only after pack's hard-delete threshold (typically 30d post soft-delete grace).
- Owner: council-privacy + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.27, A.5.28, A.8.16; GDPR Art. 17 (right to erasure with preservation of erasure-record); HIPAA §164.316(b)(2); KR PIPA Art. 36

### Repudiation (R)

**T-R-01 — Tenant denies emitting an event present in their chain**
- Asset: tenant non-repudiation
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - SPIFFE identity of the emitting µservice + tenant-binding is in the event envelope.
  - tenant_id + idempotency_key + content_sha + emitted_at + period_id form the canonical receipt.
  - Per Bominal ADR-0003 §"Receipt mandate": every emit returns a structured receipt the caller commits in its own audit; tenant has both their copy + the chain copy.
  - Ed25519 signature on the period root commits to the event; the signature is verifiable using the public key published in 3 channels (S3 + Mimir + GitHub).
- Owner: axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2); eIDAS Art. 26

**T-R-02 — Insider denies authoring a retention-cascade or key-rotation operation**
- Asset: operator non-repudiation
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Every operator-initiated retention or key-rotation requires OpenBao JIT elevation + 2-person rule.
  - Approval chain (requester + approver) recorded in OpenBao audit + emitted to audit-chain.
  - Signed commits on `policy/retention-matrix.yaml` changes; CODEOWNERS scoped.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34; GDPR Art. 5(2)

**T-R-03 — sealing-worker silently advances chain without publishing root (root-publication failure)**
- Asset: external visibility of chain advance
- Likelihood: L / Impact: M (would create a "shadow chain" no one can verify) / Risk: **L-M**
- Mitigations:
  - sealing-worker emits a SLI `audit_chain_root_publication_lag_seconds`; lag > 60s ⇒ Sev-2 page.
  - Three-channel cross-publication: missing one is operational; missing all three is Sev-1.
  - Per Bominal ADR-0028 §"Eventual consistency contract": sealing-worker MUST publish before declaring the period sealed; sealing without publication is a bug, not a feature.
- Owner: axis-audit-chain + axis-observability
- Residual: L
- Frameworks: SOC 2 CC7.2, CC8.1; ISO 27001 A.5.28, A.8.15, A.8.16; GDPR Art. 33

### Information Disclosure (I)

**T-I-01 — Cross-tenant query leak via Cedar policy misconfiguration**
- Asset: per-tenant query scope
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - query-rest enforces Cedar policy (`policy/tenant-scope.cedar`); default-deny.
  - SPIFFE identity → tenant_id binding validated independently of caller-supplied parameters.
  - Postgres queries always scoped with `WHERE tenant_id = $bound_tenant`; client-side filter is advisory; server enforces.
  - LEAN check `oya-check-cedar-fragment-coverage --microservice audit-chain` validates fragment shape on every PR.
  - Annual pen-test against the tenant boundary.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32; HIPAA §164.312(a)(1)

**T-I-02 — Audit-event payload contains PII / PHI that was supposed to be redacted by source µservice**
- Asset: audit-event raw payload
- Likelihood: M (source µservices may pass through unredacted payloads on bug) / Impact: H (GDPR / HIPAA / KR PIPA violation; long retention amplifies) / Risk: **H**
- Mitigations:
  - Per Bominal ADR-0003 §"Caller redaction obligation": the emitter is responsible for redacting `PII` / `PHI` from event payload before calling emit; the audit-chain treats the payload as opaque bytes.
  - emission-rest validates payload size + presence of mandatory `data_class` field; refuses unannotated payloads.
  - Synthetic-PII detector lane (cross-µservice) scans audit-event payloads in staging for PII patterns; production deployment blocked until lane green.
  - DSR cascade is the recovery: if PII leaks anyway, the affected events can be marked for redaction.
- Owner: each emitting µservice owner + council-privacy + axis-audit-chain
- Residual: M (engineering-discipline floor; never fully zero)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.11, A.8.12; GDPR Art. 5(1)(c) data-min, Art. 25, Art. 32; HIPAA §164.502(b) minimum-necessary; KR PIPA Art. 3

**T-I-03 — Auditor exfiltrates beyond their engagement scope**
- Asset: auditor JIT-token-scoped reads
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar policy (`policy/auditor-scope.cedar`) enforces `scoped_tenants` set + engagement-window.
  - Auditor exports are bundle-shaped — auditor receives a self-contained signed bundle; raw production access not granted.
  - All auditor reads are themselves audit-emitted; oyatie can detect anomalous read patterns.
  - mTLS pinning to auditor firm's gateway.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.34; GDPR Art. 28; HIPAA §164.308(a)(4)(ii)(B)

**T-I-04 — Anonymous schema-validation probe leaks chain-existence pattern**
- Asset: public verification endpoint
- Likelihood: L / Impact: L (only reveals "this event exists / not exist") / Risk: **L**
- Mitigations:
  - Public verification endpoint returns only `{verified: bool, reason?: string}`; never reveals event payload or tenant.
  - Rate-limited at WAF (10 req/s per IP).
  - Each probe is itself audit-emitted to detect scanning patterns.
- Owner: axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.5; GDPR Art. 32

**T-I-05 — Long retention amplifies surveillance footprint of the chain itself**
- Asset: retention-driven data growth
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Retention per pack matrix; minimum-necessary windows.
  - DSR cascade honoured within 30d.
  - Per Bominal ADR-0028 §"Retention proportionality": retention is bounded by purpose + legal minimum, not maximum possible.
  - Tenant DPA discloses retention defaults; tenant can request shorter retention (bounded by legal minimum).
- Owner: council-privacy + axis-audit-chain
- Residual: M (legal minima dominate)
- Frameworks: GDPR Art. 5(1)(e) storage-limitation; KR PIPA Art. 28; APPI Art. 20

### Denial of Service (D)

**T-D-01 — Emission flood from compromised workload µservice**
- Asset: emission-rest capacity
- Likelihood: H / Impact: H (one µservice DoS'ing the chain DoS's every µservice's compliance posture) / Risk: **H**
- Mitigations:
  - Per-(SPIFFE-source, tenant) rate limits at emission-rest; defaults sized in `capacity-model.md`.
  - HPA on emission-rest (CPU + queue depth).
  - Postgres WAL is partition-sharded by tenant; one tenant's flood does not block others' shards.
  - Sustained per-source rate-limit exceedance triggers Sev-2 page + tenant comms.
- Owner: ops-sre-reliability + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — HSM partition outage**
- Asset: signing capacity
- Likelihood: L (Oracle HSM SLA 99.95%+) / Impact: H (no new seals until restored) / Risk: **M**
- Mitigations:
  - emission-rest does NOT depend on HSM; emit() succeeds and queues for sealing.
  - sealing-worker enters degraded mode: events accumulate in unsealed buffer; once HSM restored, batch-seal catches up.
  - SLI `audit_chain_unsealed_buffer_depth_seconds`; > 60s ⇒ Sev-2; > 1h ⇒ Sev-1.
  - Per Bominal ADR-0028 §"Degraded mode": emission is decoupled from sealing; chain catches up.
  - DR pair packs have HSM partitions in both regions; failover ≤ 35min.
- Owner: ops-sre-reliability + cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC7.1, CC9.1; ISO 27001 A.5.30, A.8.14

**T-D-03 — Postgres outage**
- Asset: WAL + index store
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - HA primary + replica per pack.
  - emission-rest writes to local-WAL-on-disk as fallback; replays to Postgres on recovery.
  - DR pair packs have replicated Postgres + S3 WORM is the source of truth (Postgres is the index, recomputable from S3).
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14

**T-D-04 — S3 bucket throttling**
- Asset: S3 WORM raw-blob storage
- Likelihood: M / Impact: H / Risk: **M**
- Mitigations:
  - Per-tenant prefix sharding to distribute write load across S3 partitions.
  - Bucket-level rate limits monitored; alarm at 70% / 85% / 95% of bucket-published throughput.
  - Multi-bucket sharding when single-bucket approaches limit.
- Owner: ops-sre-reliability + cloud-secrets
- Residual: L

**T-D-05 — Resource exhaustion from oversized event payloads**
- Asset: emission-rest + WAL
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - emit() enforces payload size limit (default 1 MB; pack-overridable).
  - Oversized payloads return 413; emit a metric.
  - Per Bominal ADR-0003 §"Payload size limit": callers should externalise large payloads (e.g., document into object-storage; emit only the hash + reference).
- Owner: axis-audit-chain
- Residual: L

### Elevation of Privilege (E)

**T-E-01 — `audit_emitter` Postgres role escalated to allow UPDATE/DELETE**
- Asset: WAL append-only invariant
- Likelihood: L / Impact: Critical / Risk: **M-H**
- Mitigations:
  - Postgres role grants Terraform-managed; change requires PR + LEAN-check + 2-person rule for grant edits.
  - LEAN check `oya-check-audit-chain-postgres-role-conformance` validates live cluster permissions match git-declared.
  - Continuous validation: a CronJob runs `pg_role_audit` weekly; deviation pages ops-security.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.3, A.8.4

**T-E-02 — Sealing-worker role granted HSM admin (key-create / key-delete)**
- Asset: HSM key lifecycle
- Likelihood: L / Impact: Critical / Risk: **M-H**
- Mitigations:
  - HSM-side IAM separates sealing-worker SA (sign-only) from rotation-operator SA (create-and-retire).
  - Rotation operator requires 2-person rule + OpenBao JIT elevation.
  - HSM admin operations cross-audit-emitted; mass-key-deletion patterns trigger anomaly alert.
  - Oracle OCI side: dedicated HSM partition; no shared admin between partitions.
- Owner: ops-security + cloud-secrets + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.17, A.8.3, A.8.4

**T-E-03 — Operator-level access to retention-cascade-worker used to bypass retention defaults**
- Asset: retention-cascade authority
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - retention-cascade-worker enforces declared policy; no operator-bypass interface.
  - Emergency-redaction operations (e.g., privacy-team request for an out-of-cadence DSR) require OpenBao JIT + 2-person rule + audit-emit.
  - Per Bominal ADR-0028 §"Retention proof": every retention application emits a Merkle proof of the deleted event's pre-deletion presence; bypass attempts leave detectable gaps.
- Owner: council-privacy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4; GDPR Art. 17 + Art. 32

**T-E-04 — Cedar policy escape**
- Asset: Cedar policy evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4+; fragments fuzzed at CI time.
  - Field input lengths bounded at REST API.
  - LEAN check on fragment coverage.
- Owner: axis-audit-chain + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-05 — Supply-chain attack on cryptography library (ring / ed25519-dalek / etc.)**
- Asset: signing + verification correctness
- Likelihood: L / Impact: Critical / Risk: **M**
- Mitigations:
  - `cargo deny` enforces version pins on crypto crates.
  - Cosign-signed builds; reproducible builds (per `docs/standards/build-reproducibility.md`).
  - Crypto crates monitored by weekly `oya-governance-supply-chain` lane (RustSec advisory db cross-check).
  - HSM-side signing for the load-bearing path means a compromised Rust crypto crate cannot directly leak keys (HSM mediates).
  - Verification uses the same crypto crate; compromise would affect both; quarterly red-team drills include "verify against a known-bad signature" to validate verification correctness.
- Owner: ops-security + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC6.8, CC8.1; ISO 27001 A.5.21, A.8.28; GDPR Art. 32(1)(b)

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | event payload + tenant_id correlation | Multiple events for a subject can be linked across µservices via shared subject_hash. | DSR cascade honours subject erasure within 30d; subject_hash uses per-deployment salt rotated 12mo; long retention bounded by purpose. | M (inherent to audit purpose) |
| T-L-02 | Identifiability | subject_hash | Small-tenant subjects re-identifiable via auxiliary data. | Salted-hash + per-pack rotation; small-tenant rate-limit on cross-tenant aggregate queries. | L |
| T-L-03 | Non-repudiation | This is the *design goal* — non-repudiation IS the purpose. | n/a — designed-in property. | n/a |
| T-L-04 | Detectability | emit-rate timing | Burst emissions correlate with tenant business events. | Disclosed in DPA; tenant-onboarding consent; standard for any audit log. | M (inherent) |
| T-L-05 | Disclosure | export bundles | Auditor bundles could be leaked. | Bundles signed by oyatie's pack key; auditor firm bound by engagement contract; bundle content scoped per engagement; bundle exfil itself is an internal breach incident. | L |
| T-L-06 | Unawareness | End-users (the tenants' users) | Tenant's end-users may not know audit-chain captures their activity. | Joint-controllership disclosure required from tenant per DPA; oyatie's processing covered in tenant's notice. | M |
| T-L-07 | Non-compliance | Right-to-erasure | Subject requests erasure; chain retains a Merkle proof of redaction (not the data itself). | Soft-delete preserves proof while removing payload; per Bominal ADR-0028 §"Right-to-erasure with chain preservation". GDPR Art. 17 honoured via "right to redaction with audit-of-redaction"; recital-65 permits retention for legal-claims defence within chain integrity. | L-M |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Mimir multi-tenancy enforced for root-publication channel | Preventive | axis-audit-chain + axis-observability | `oya-check-mimir-tenancy-enforced` lane |
| SPIFFE identity-bound emission | Preventive | ops-security | SPIFFE attestation log + audit-chain emission log |
| Per-emitter Cedar policy | Preventive | ops-security | `oya-check-cedar-fragment-coverage` |
| HSM-backed signing keys | Preventive | cloud-secrets + axis-audit-chain | HSM-side audit log + signing-correctness e2e |
| Key-rotation overlap (24h) | Preventive | ops-security + axis-audit-chain | rotation drill quarterly |
| S3 Object Lock Compliance mode | Preventive | cloud-secrets + axis-audit-chain | bucket policy review |
| Postgres INSERT-only role for emitter | Preventive | ops-security | `oya-check-audit-chain-postgres-role-conformance` |
| Three-channel root publication (S3 + Mimir + GitHub) | Detective | axis-audit-chain | cross-channel divergence recording-rule alert |
| `idempotency_key` + content_sha dedupe | Preventive | axis-audit-chain | unit + integration tests |
| Soft-delete with Merkle proof of redaction | Preventive (compliance) | council-privacy + axis-audit-chain | DSR cascade integration test |
| 2-person rule for HSM admin + retention overrides | Preventive (insider) | ops-security | OpenBao JIT log |
| Per-tenant cardinality + emission rate limits | Preventive (DoS) | axis-audit-chain | emission-rest metrics |
| Reproducible cosign-signed builds for crypto path | Preventive (supply-chain) | ops-security | `oya-governance-supply-chain` |
| Cross-pack-replication-forbidden (default deny) | Preventive (residency) | axis-audit-chain | `oya-check-cross-pack-replication-forbidden` lane |
| External verifier reference implementation | Detective (transparency) | axis-audit-chain | quarterly drill: external verifier validates pack-kr |

## Residual Risk Acceptance

Residual risks above L (low) require explicit acceptance:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-02 (PII / PHI in payloads) | M | Caller-redaction floor is engineering-discipline-bound; mitigated to acceptable via detection + DSR cascade. | Quarterly |
| T-I-05 (long-retention surveillance) | M | Legal minima dominate (HIPAA 6y, KR-FSS 5y); minimum-necessary applied within those minima. | Annually |
| T-L-01 (linkability) | M | Inherent to audit purpose. | Annually |
| T-L-04 (detectability) | M | Inherent. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant-side responsibility; joint-controllership clause. | Annually |
| T-L-07 (right-to-erasure with chain) | L-M | Recital-65 permits retention for legal-claims defence; soft-delete model is the EDPB-recommended pattern. | Quarterly |

Sign-off (this document RW until council sign-off):

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

Compliance frameworks: KR-ISMS-P + KR PIPA + KR 전자문서법.

- **KR 전자문서법 Arts. 5–7**: audit-chain Ed25519 seal satisfies electronic-document integrity + storage + verification. This is one of the load-bearing legal mappings for pack-kr — Korean courts accept the chain as electronic-document evidence under these articles.
- **KR PIPA Art. 23 + 23-2**: sensitive personal info handling; salt rotation 12mo; cross-pack-replication forbidden.
- **KR PIPA Art. 28**: storage period limitation; default 3y; per KR-FSS sector tenants 5y.
- **KR PIPA Art. 29-2**: encryption requirement; AES-256 at rest + TLS 1.3 in transit + Ed25519 for chain integrity.
- **KR PIPA Art. 34**: breach notification chain integrated with `incident-response.md`.
- **KR-ISMS-P §2.5–§2.12**: cross-mapped in `compliance.md`.

### pack-us-healthcare (HIPAA)

- **§164.312(b) (audit controls)**: this entire µservice is the implementation.
- **§164.312(c)(1) (integrity)**: Ed25519 + Merkle tree + WORM storage; cross-channel publication.
- **§164.316(b)(2) (retention ≥ 6y)**: retention-cascade enforces; verified by retention-conformance lane.
- **§164.308(a)(1)(ii)(A) (risk analysis)**: this document + DPIA.
- **BAA**: per-tenant; `legal/baa-template.md`.

### pack-eu (GDPR + eIDAS + NIS2)

- **GDPR Art. 25 (privacy-by-design)**: pseudonymisation + per-pack residency + soft-delete-with-proof.
- **GDPR Art. 30 (records of processing)**: this µservice IS the records-of-processing register backbone.
- **GDPR Art. 17 (right to erasure)**: soft-delete with Merkle proof of redaction.
- **eIDAS 910/2014 Art. 26 (AdES)**: HSM-backed Ed25519 satisfies AdES requirements.
- **NIS2 (2022/2555)**: incident reporting timelines integrated.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/audit-chain-overlay.md`; each maps the local audit-trail-retention + integrity-of-electronic-records laws to T-T-01..T-T-06 + T-I-01..T-I-05.

## Re-review Triggers

- Any change to trust boundary diagram.
- Any HSM hardware or library upgrade.
- Any cryptography crate (`ring`, `ed25519-dalek`, etc.) version bump.
- Annual scheduled review (Q2).
- Post-incident (any Sev-1 or Sev-2 involving audit-chain or chain integrity).
- Pen-test or audit finding.

## References

- Bominal ADR-0028 (Audit chain — Merkle + Ed25519); inherited.
- Bominal ADR-0003 (Audit emission contract); inherited.
- ADR-0056; ADR-0105; ADR-0117; ADR-0123; ADR-0131; ADR-0140.
- `microservices/audit-chain/PRD.md`.
- `microservices/audit-chain/dpia.md`.
- `microservices/audit-chain/policy/{seal-integrity, data-residency, tenant-scope, ci-scope, auditor-scope, public-read}.{md,cedar}`.
- `microservices/audit-chain/compliance.md`.
- `/specs/audit-chain-merkle-ed25519.json`.
- RFC 6962 (Certificate Transparency, Merkle-tree-shape reference).
- Google SRE Workbook ch. 5–6.
- Grafana Mimir security model.
- OCI Cloud-HSM docs.
- KR 전자문서법 — `law.go.kr/lsInfoP.do?lsiSeq=233358`.
- eIDAS 910/2014 — `eur-lex.europa.eu/legal-content/EN/TXT/?uri=uriserv%3AOJ.L_.2014.257.01.0073.01.ENG`.
- HIPAA 45 CFR Part 164 Subpart C — `hhs.gov/hipaa/for-professionals/security/laws-regulations/`.
