---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-security
deciders: council-architecture, ops-security, axis-foundry, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0123, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/industry-best-practice-conformance.json]
review_cadence: quarterly + on every CI substrate or branch-protection change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.3, CC7.4, CC7.5, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.30, A.5.31, A.5.32, A.5.33, A.5.34, A.5.36, A.5.37, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28, A.8.30, A.8.31"
  - "SLSA v1.0 — Build L3, Source L3, Isolation L3"
  - "NIST SSDF SP 800-218 — PO.1, PO.3, PO.4, PS.1, PS.2, PS.3, PW.1, PW.4, PW.6, PW.7, PW.8, RV.1, RV.2"
  - "GDPR Arts. 5, 6, 13, 14, 17, 22, 25, 30, 32"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 23/24/25/29"]
  pack-us-healthcare: ["HIPAA §164.308/310/312/316"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35", "NIS2 Annex I/II when applicable"]
  pack-jp: ["APPI Arts. 20/21/23"]
  pack-sg: ["PDPA Protection Obligation", "MAS-TRM v2021"]
  pack-au: ["Privacy Act APP 11", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 33/46/48"]
  pack-ae: ["UAE PDPL Art. 15"]
  pack-ksa: ["PDPL Art. 9", "SAMA Cybersecurity Framework"]
doc_status: published
---

# Threat Model: governance µservice

## Purpose

Identify, classify, and mitigate threats to the governance µservice's confidentiality, integrity, availability, and privacy posture. Governance is the **gate authority** for every oyatie PR's admission to `dev` and every µservice's promotion path; a compromise here cascades to every product. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, SLSA assessors, and GDPR DPAs at first-tenant onboarding.

The governance µservice sits in a **trusted-internal-but-targeted** position: external attackers who breach a PR-author identity could weaponise it against every downstream µservice; insider misuse could silently disable fitness lanes; supply-chain attacks on lane crates could compromise the entire CI surface.

## Scope

### In-scope

All components introduced by ADR-0131 §"governance bundle" + ADR-0133 (6-axis program) for the governance µservice, deployed in a **dedicated governance cluster** within the observability cluster's trust domain (decision confirmed 2026-05-17; matches hyperscaler practice — GitHub Actions runners run in a hardened pool isolated from workload clusters; Codacy / SonarCloud run on dedicated infra):

| Layer-A (adopted OSS/external) | Layer-B (oyatie-owned) |
|---|---|
| GitHub Actions runner ARC pool | `oya-governance-lane-runtime-*` (9 crates) |
| Postgres for lane-state + Finding metadata | `oya-governance-policy-engine-*` (9 crates) |
| S3 / OCI Object Storage for evidence blobs | `oya-governance-evidence-emitter-*` (9 crates) |
| OpenBao for secrets (Ed25519 keys, GitHub PAT, Postgres creds) | `oya-governance-aggregation-indexer-*` (9 crates) |
| GitHub branch-protection (admission origin) | ~50 migrated `oya-check-*` lane crates |
| Audit-chain µservice (Ed25519 seals; upstream) | Lane workflow files at `.github/workflows/governance-suite.yml` |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS layer — owned by the `cloud-k8s` µservice's threat model.
- Threats to OpenBao secret-manager itself — owned by the `cloud-secrets` µservice's threat model. Inherited as upstream.
- Threats to the audit-chain µservice's Ed25519 signing infrastructure — owned by the `audit-chain` µservice's threat model. Inherited.
- Threats to the workload µservices themselves (tenancy, ontology, workflow, mail) — each owns its own threat-model.md.
- Threats to GitHub itself (vendor-managed) — out of scope; mitigated by SLSA L3 source-provenance enforcement.

## Trust Boundaries

```text
┌─ External (Public Internet) ───────────────────────────────────────────────┐
│                                                                            │
│   PR author (human or agent)        External auditor (JIT)                 │
│         │                                  │                               │
│         │ (HTTPS + signed-commits)         │ (JIT short-lived OIDC token)  │
│         ▼                                  ▼                               │
│  ┌─ GitHub.com (vendor-managed) ─────────────────────────────────────┐     │
│  │  - branch-protection (required_status_checks)                     │     │
│  │  - PR webhook to oyatie tenancy event bus                         │     │
│  │  - merge-queue projected-state per ADR-0111                       │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Dedicated governance cluster (within observability trust domain) ─────────┐
│                                                                            │
│  Trust boundary 1: External (GitHub webhook) → Cluster ingress             │
│    - mTLS terminated at Envoy ingress                                      │
│    - HMAC-SHA256 signed webhook payload                                    │
│                                                                            │
│  ┌─ governance-lane-runtime-rest ──────────┐                               │
│  │  - OIDC PR-author identity verification │                               │
│  │  - Per-lane rate-limit                  │                               │
│  └─────────────────────────────────────────┘                               │
│             │                                                              │
│  Trust boundary 2: Lane-runner ARC pool (hardened; ephemeral runners)      │
│             │                                                              │
│  ┌─ GitHub Actions Runner Controller (ARC) ──────────────────────────┐     │
│  │  - Ephemeral runner per lane invocation                            │    │
│  │  - Pod-level network policy: outbound only to allow-listed hosts   │    │
│  │  - No persistent disk; tmpfs only                                  │    │
│  │  - SPIFFE workload identity per runner                             │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│             │                                                              │
│  Trust boundary 3: Lane runner → Postgres (lane-state + Finding metadata)  │
│             │                                                              │
│  ┌─ Postgres (HA primary + 2 replicas) ──────────────────────────────┐     │
│  │  - mTLS only; per-runner identity-mapped role                     │     │
│  │  - Row-level security on Finding by microservice                  │     │
│  │  - WAL-G to S3 for PITR                                           │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│             │                                                              │
│  Trust boundary 4: Evidence-emitter → S3 (evidence blobs)                  │
│             │                                                              │
│  ┌─ OCI Object Storage (S3-compatible) ──────────────────────────────┐     │
│  │  - SSE-KMS (per-pack KMS keyring)                                 │     │
│  │  - Object-lock (compliance mode): append-only; 7y retention      │     │
│  │  - Content-addressed (SHA256 of canonical-JSON)                   │     │
│  │  - Per-microservice key prefix; IAM scoped                        │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│             │                                                              │
│  Trust boundary 5: Evidence-emitter → audit-chain µservice (Ed25519 seal)  │
│             │                                                              │
│  ┌─ audit-chain (upstream µservice) ─────────────────────────────────┐     │
│  │  - Merkle-tree aggregation; Ed25519 sign; HSM-backed where        │     │
│  │    available                                                       │     │
│  │  - Per-PR seal record; per-quarter root publication              │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│             │                                                              │
│  Trust boundary 6: Aggregation-indexer → Git push (central-index commits) │
│             │                                                              │
│  ┌─ Git refs PATCH (signed commits, GitHub API) ─────────────────────┐    │
│  │  - WORKFLOW_PAT scoped to: docs/prds/INDEX.md, registry/catalog/, │    │
│  │    /specs/microservices/ (and only these paths)                        │    │
│  │  - Signed commit (gpg-sign + Ed25519 verify)                      │    │
│  │  - Audit emission: every commit writes Ed25519 audit record       │    │
│  └───────────────────────────────────────────────────────────────────┘    │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. **External webhook ingress** (HMAC-SHA256 + mTLS).
2. **Ephemeral runner pool** (per-lane isolation; SPIFFE identity).
3. **Lane runner → Postgres** (mTLS + RLS).
4. **Evidence-emitter → S3** (SSE-KMS + object-lock).
5. **Evidence-emitter → audit-chain** (Ed25519 seal).
6. **Aggregation-indexer → Git refs PATCH** (PAT-scoped; signed commits).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `oya-check-data-class` LEAN lane (self-applied).

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Finding records (per-lane, per-PR violations) | `AUDIT` + `INTERNAL_ONLY` | High | Postgres 90d hot + S3 7y cold (SOC 2 + ISO 27001 retention) | Postgres + S3 |
| Lane-run metadata (lane_id, sha, duration, runner_profile) | `AUDIT` + `INTERNAL_ONLY` | Medium | Postgres 90d hot + S3 2y cold | Postgres + S3 |
| Evidence blobs (full lane output, including stderr/stdout transcripts) | `AUDIT` + occasionally `BEHAVIORAL_TENANT_PRODUCT` (when lane runs over tenant artifacts) | High | S3 7y cold; object-locked | S3 |
| Industry-baseline pins (`/specs/industry-best-practice-conformance.json`) | `INTERNAL_ONLY` | Low | git history; quarterly refresh | repo |
| Rule packs (per-lane rule definitions) | `INTERNAL_ONLY` | Low | git history | `microservices/governance/src/crates/oya-check-*/rules/` |
| Aggregation indices (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`) | `INTERNAL_ONLY` | Low | git history; generated | repo |
| Cedar policy fragments | `INTERNAL_ONLY` | Medium | git history | `microservices/governance/policy/*.cedar` |
| Ed25519 signing keys (evidence seal + index-commit signing) | `SECRET` | Critical | OpenBao with 90d rotation + HSM-backed where available | OpenBao |
| GitHub PAT (lane-runtime → GitHub Actions matrix dispatch + index-commit push) | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| Postgres credentials (per-runner role; per-replica reader) | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| S3 access keys (evidence-emitter; replay-query reader) | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| Per-PR diff metadata (source SHA, target branch, author identity) | `BEHAVIORAL_TENANT_PRODUCT` (tenant-author-attributable when tenant operator opens PR) | Medium | Postgres 90d hot + S3 2y cold | Postgres + S3 |
| Audit-chain seal records | `AUDIT` | High | append-only; immutable; published quarterly root | audit-chain µservice |
| PR-author identity (OIDC subject + IdP claims) | `PII_IDENTIFYING` (email + name); occasionally `PII_QUASI_IDENTIFIER` (IP + UA) | High | 90d hot + 2y cold; minimised | Postgres |
| External-auditor JIT tokens (short-lived OIDC) | `SECRET` | Critical | OpenBao-issued; ≤1h TTL | OpenBao |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| PR author (human or agent) | Untrusted external | GitHub OAuth + signed-commits | Open PR; lane set runs against their ref; cannot bypass gates |
| Tenant operator (when PR touches their tenant's µservice) | Untrusted external | OIDC + MFA via Application Shell | Read own µservice's Findings; cannot read others' |
| oyatie CI runner (GitHub Actions; ephemeral) | Semi-trusted internal | SPIFFE workload identity + scoped PAT | Execute lanes; write Findings; emit dispatch events; ephemeral filesystem only |
| Lane-runtime worker (long-lived service) | Trusted internal | OpenBao-issued service-account token | Dispatch matrix jobs; collect verdicts; admission-gate query API |
| Evidence-emitter worker (long-lived service) | Trusted internal | OpenBao-issued service-account token | Write Findings to Postgres + S3; seal via audit-chain; serve replay |
| Aggregation-indexer worker (long-lived service) | Trusted internal | OpenBao-issued service-account token | Read per-µservice sources; write central indices via scoped PAT |
| Reviewer agent (`oya-pr-review` lane) | Trusted internal | OIDC-bound CI identity | Read Findings; APPROVE / REQUEST_CHANGES on PRs |
| Council-architecture / ops-security operators (human) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Admin-level rule-pack edits via PR review; cannot bypass lanes (subject to lanes themselves) |
| External auditor (SOC 2 / ISO 27001 / SLSA) | Read-only external on a time-boxed window | OIDC + MFA + JIT short-lived token via OpenBao | Read-only on Findings + evidence + audit-chain export; cannot pivot |
| Attacker — opportunistic | Untrusted | none | Webhook scanning; PR-author identity attempts; assume always present |
| Attacker — targeted (supply-chain motivated) | Untrusted | none | Lane-crate supply-chain attack; rule-pack tampering attempt; assume present for prod-tier surfaces |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure rule packs, lane registry, or baseline pins (mitigated by PR-review + self-application) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor: silently disable lanes, forge Findings, escalate (mitigated by least-privilege + audit-chain + separation-of-duties + self-application bootstrap) |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score (likelihood × impact); mitigations; owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Attacker impersonates a lane runner to write false Findings or zero-violations**
- Asset: Finding write path
- Likelihood: M / Impact: H (could let bad-code merge or block good-code) / Risk: **H**
- Mitigations:
  - Lane runners authenticate to Postgres via SPIFFE workload identity (mTLS + per-runner role); not exposed to GitHub Actions runners under non-runner identities.
  - Postgres RLS: only the `lane-runtime-writer` role inserts into `findings`; insert requires matching `runner_id` claim.
  - Evidence-emitter cross-checks every Finding's `runner_id` against a known-active runner roster maintained by the lane-runtime worker.
  - Audit-chain seal includes runner identity claim; tampering visible at quarterly root publication.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC7.1, CC7.4; ISO 27001 A.5.15, A.5.16, A.8.2, A.8.3, A.8.7, A.8.16; SLSA Build L3; NIST SSDF PS.1, PS.2

**T-S-02 — Attacker forges GitHub webhook to trigger fake lane runs or skip lanes**
- Asset: Webhook ingress
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - HMAC-SHA256 signature validation on every webhook payload (X-Hub-Signature-256 header); secret rotated 30d via OpenBao.
  - mTLS termination at Envoy ingress; webhook source IP allow-list (GitHub's published ranges).
  - Replay protection: webhook payload `delivery-id` deduplicated for 10min.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.5.14, A.8.20; OWASP API01

**T-S-03 — Attacker impersonates the aggregation-indexer to push hand-crafted central-index commits**
- Asset: Aggregation-index Git push path
- Likelihood: L / Impact: H (could inject false PRDs, hide retired vocabulary, etc.) / Risk: **M**
- Mitigations:
  - WORKFLOW_PAT scoped to aggregation-index paths only (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`); IAM-enforced at GitHub level.
  - Signed commits (gpg-sign + Ed25519 verify on receive); commit signature key in OpenBao.
  - `oya-check-aggregation-index-generation` lane re-runs on every PR and refuses divergence between the index and the per-µservice sources.
- Owner: axis-foundry
- Residual: L
- Frameworks: ISO 27001 A.5.15, A.8.21, A.8.31; SLSA Source L3

### Tampering (T)

**T-T-01 — Lane-crate supply-chain attack: malicious dependency in `oya-check-*`**
- Asset: ~50 lane crates' transitive dependencies
- Likelihood: M / Impact: H (could silently disable lanes; could forge Findings) / Risk: **H**
- Mitigations:
  - `cargo deny check` runs against every workspace build (license + vulnerability + advisory).
  - `oya-check-supply-chain` lane (BLOCKER on dev) enforces SBOM coverage; SLSA L3 provenance on every dependency.
  - Renovate auto-PRs for security advisories; `oya-check-vendor-recency` lane refuses stale deps.
  - Per-runner network policy: outbound only to allow-listed hosts (crates.io, github.com, OpenBao); no arbitrary egress.
  - SPDX SBOM emitted with every governance release tag.
- Owner: ops-security
- Residual: M (transitive supply-chain risk is unfixable; residual is monitored)
- Frameworks: SLSA Build L3, Source L3; NIST SSDF PS.3; CIS SLSA framework; OpenSSF Best Practices Badge

**T-T-02 — Rule-pack tampering: insider edits a rule pack to silently soften a lane**
- Asset: `microservices/governance/src/crates/oya-check-*/rules/*.toml`
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Every rule-pack edit requires a PR (no direct push); PR runs the full ~50-lane set including `oya-check-quality-lane` (which validates rule-pack schema and forbids weakening without ADR).
  - Two-reviewer requirement for rule-pack edits via CODEOWNERS (axis-foundry + ops-security).
  - Self-application: any softening that would let the softening PR itself pass un-noticed is caught by the synthetic-probe fallback per PRD Open Q3.
  - Audit-chain seal on every rule-pack git blob; diff-replay shows historical posture.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.34, A.8.32; SLSA Source L3

**T-T-03 — Finding tampering: attacker mutates Postgres rows to mark BLOCKER → PASS**
- Asset: Postgres `findings` table
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Postgres role separation: `lane-runtime-writer` (insert-only), `replay-reader` (select-only), `admin` (no direct DML; structural only).
  - Every row is content-addressed: `finding_hash = SHA256(canonical_json(row))`; tamper detected at next read.
  - Audit-chain seal includes finding_hash; mismatch at quarterly root surfaces tampering.
  - WAL-G PITR enables forensic recovery.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.13, A.8.34; GDPR Art. 32(1)(b)

**T-T-04 — Industry-baseline pin tampering: insider points to a softer baseline**
- Asset: `/specs/industry-best-practice-conformance.json`
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - File is in repo; every edit requires PR + ADR-#### successor-IP + council-architecture review per ADR-0133 §"Operational".
  - `oya-check-industry-best-practice-conformance` lane self-applies: detects pin softening absent ADR.
  - Quarterly refresh PR is automated (`axis-foundry-bot`); manual edits flagged for human review.
- Owner: council-architecture
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.34

### Repudiation (R)

**T-R-01 — PR author denies opening a PR that triggered a Finding**
- Asset: PR-author identity → Finding linkage
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Signed-commits enforced via branch-protection.
  - Every Finding records `pr_number`, `pr_author_subject`, `commit_sha`, `signature`.
  - Audit-chain seal includes PR-author identity; quarterly root publication makes denial cryptographically infeasible.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.3; ISO 27001 A.5.16, A.8.34; SLSA Source L3

**T-R-02 — Insider denies modifying a rule pack**
- Asset: rule-pack edit history
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - All edits go through PR; git history with signed commits; CODEOWNERS approval recorded.
  - Audit-chain seal on every rule-pack blob.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.3; ISO 27001 A.5.16

### Information Disclosure (I)

**T-I-01 — Cross-tenant Finding read: tenant-A reads tenant-B's findings via replay-query**
- Asset: Postgres `findings` + S3 evidence
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Replay-query API requires OIDC + tenant-scope claim; Postgres RLS by `microservice → tenant` mapping in `ontology`.
  - S3 evidence bucket: per-microservice key prefix; IAM scoped to read only own prefix.
  - Cedar policy fragment `tenant-scope.cedar` enforces ABAC at API gateway.
- Owner: ops-security
- Residual: L
- Frameworks: GDPR Art. 25; ISO 27001 A.8.3; SOC 2 CC6.1, CC6.6

**T-I-02 — Evidence blob leakage: S3 bucket misconfiguration**
- Asset: S3 evidence bucket
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SSE-KMS encryption mandatory (per-pack KMS keyring).
  - Bucket policy: deny `s3:PublicAccess`; deny any external principal.
  - Object-lock in compliance mode prevents accidental deletion; 7y retention.
  - Quarterly bucket-policy review per `runbooks/aggregation-rebuild.md`.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.24, A.8.25; GDPR Art. 32(1)(a)

**T-I-03 — PII leakage in Finding evidence (e.g., user-id in test failure output)**
- Asset: evidence blobs
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Evidence-emitter sanitiser pass: redacts known PII patterns (email, phone, SSN, KR RRN) before write.
  - `oya-check-data-class` lane refuses unannotated test fixtures containing real-looking PII.
  - Cedar `data-residency.md` policy scopes evidence reads to same-pack auditors.
- Owner: ops-privacy + ops-security
- Residual: M (regex-based redaction is imperfect; signal of false-negatives reviewed monthly)
- Frameworks: GDPR Arts. 5(1)(c), 25, 32; pack-kr KR PIPA Art. 23

### Denial of Service (D)

**T-D-01 — PR-bomb: attacker opens hundreds of PRs to exhaust lane-runner pool**
- Asset: lane-runner ARC pool
- Likelihood: H / Impact: M / Risk: **H**
- Mitigations:
  - Per-author concurrent-PR rate limit (max 10 in-flight) enforced at admission gate.
  - ARC autoscaling with max-replicas cap; pre-warmed pool of 8 standbys.
  - Per-µservice fairness queue: no µservice can monopolize > 30% of runner pool.
  - Cost-budget lane (`oya-check-cost-budget`) on the governance µservice itself: monthly spend cap; PR-bomb breach alerts ops-finops + ops-sre-reliability.
- Owner: ops-sre-reliability
- Residual: M
- Frameworks: ISO 27001 A.5.30, A.8.6; SOC 2 A1.1; OWASP API04

**T-D-02 — Lane-runner OOM: pathological PR (e.g., 100k-file diff) exhausts runner memory**
- Asset: individual lane runner
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-runner memory ceiling: 8 GB; OOM-kill within timeout.
  - Lane timeout: 60s p99 per lane; SIGKILL on overrun.
  - Per-lane input-size cap: refuses PRs with diff > 10k files (returns BLOCKER with `pr-too-large` finding; author must split).
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 A1.1; ISO 27001 A.8.6

**T-D-03 — Aggregation-indexer regen storm: every PR triggers full regen, saturates Postgres**
- Asset: aggregation-indexer worker
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Coalescing: regen requests within 15min window collapse to one execution.
  - Incremental regen: only re-process µservices whose source files changed (Merkle-tree based).
  - Background full regen on 5min cron (independent of PR signals).
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 A1.1

### Elevation of Privilege (E)

**T-E-01 — Lane bypass via admin-merge without break-glass record**
- Asset: branch-protection
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Branch-protection: `enforce_admins = true` so even admins cannot bypass `required_status_checks` without explicit dismissal.
  - When dismissal happens, GitHub audit log fires; `oya-check-protection-context-match` lane detects retroactively on next PR.
  - Break-glass procedure documented in `runbooks/lane-bypass-emergency.md`: requires two ops-security signatures + recorded justification + post-incident review.
  - Quarterly review by ops-security of all bypass records.
- Owner: ops-security + council-architecture
- Residual: M (admin override is unavoidable for true emergencies)
- Frameworks: SOC 2 CC6.3; ISO 27001 A.5.34, A.8.34

**T-E-02 — Policy-engine privilege escalation: rule-pack execution as a CLI subprocess**
- Asset: rule-pack execution sandbox
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Rule packs are declarative TOML/YAML; no arbitrary code execution. Cedar fragments parse-and-eval only.
  - Lane crates run in seccomp-restricted containers (default Docker seccomp profile + ARC hardening).
  - Per-runner SPIFFE identity is least-privilege; no kubectl, no AWS credentials, no Postgres admin.
- Owner: ops-security
- Residual: L
- Frameworks: ISO 27001 A.8.22; CIS Kubernetes Benchmark §5.7

**T-E-03 — Aggregation-indexer overreaches its scoped PAT (writes outside permitted paths)**
- Asset: WORKFLOW_PAT path scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - PAT scoped at GitHub level to specific paths (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`).
  - Pre-push hook: aggregation-indexer asserts every modified path is in its scope-allow-list before push.
  - On scope-overrun: refuse push + emit `aggregation-scope-overrun` Finding + audit-chain seal.
- Owner: axis-foundry
- Residual: L
- Frameworks: ISO 27001 A.5.15, A.8.3; SLSA Source L3

## LINDDUN Privacy Threat Catalog

| ID | Category | Threat | Likelihood | Impact | Risk | Mitigations | Frameworks |
|---|---|---|---|---|---|---|---|
| P-L-01 | Linkability | PR-author identity cross-linked across tenants via Findings | L | M | L | Hashed identity in non-AUDIT views; AUDIT view restricted to ops-security | GDPR Art. 25; ISO 27001 A.5.34 |
| P-I-01 | Identifiability | Test failure transcripts expose real user emails | M | M | M | Sanitiser pass per T-I-03; data-class lane refuses real-PII fixtures | GDPR Arts. 5(1)(c), 25 |
| P-N-01 | Non-repudiation | PR-author cannot deny actions (intended) | — | — | — | This is intentional per T-R-01; documented for transparency | GDPR Art. 22 (not a Solely-Automated decision) |
| P-D-01 | Detectability | Long-term retention enables author behaviour profiling | L | M | L | 7y AUDIT retention bounded by SOC 2 + ISO 27001 minimums; profiling refused at policy level | GDPR Arts. 5(1)(e), 17 |
| P-D-02 | Disclosure | External auditor JIT scope leaks tenant evidence | L | H | M | JIT scope is bound to a single audit window; tenant-scope claim in OIDC token enforced at API gateway | SOC 2 CC6.6; GDPR Art. 28 |
| P-U-01 | Unawareness | PR-author unaware that PR-author identity is sealed in audit-chain | M | M | M | Public CLAUDE.md + docs/AGENTS.md disclosure; PR template includes consent footer | GDPR Arts. 13, 14 |
| P-NC-01 | Non-compliance | Per-pack retention violation (KR commercial-code 5y vs HIPAA 6y vs general 7y) | M | H | H | Per-pack retention overlay at `iac/kustomize/overlays/pack-<pack>/`; tested at deploy time | GDPR Art. 5(1)(e); pack-kr KR PIPA Art. 21; HIPAA §164.316(b)(2) |

## OWASP Top 10 (2021) Mapping

| OWASP | Risk for governance | Mitigation reference |
|---|---|---|
| A01 Broken Access Control | T-I-01 (cross-tenant Finding read); T-E-01 (lane bypass) | T-I-01, T-E-01 |
| A02 Cryptographic Failures | T-I-02 (S3 misconfig); Ed25519 key rotation | T-I-02; `runbooks/key-rotation.md` successor-IP |
| A03 Injection | Rule-pack execution sandbox (T-E-02) | T-E-02 |
| A04 Insecure Design | 6-axis program per ADR-0133; first-class threat model | this document |
| A05 Security Misconfiguration | T-I-02 (bucket policy); branch-protection drift (`oya-check-protection-context-match`) | T-I-02; lane self-application |
| A06 Vulnerable + Outdated Components | T-T-01 (supply-chain) | T-T-01 |
| A07 Identification + Authentication Failures | T-S-01 (lane impersonation); T-S-02 (webhook forgery) | T-S-01, T-S-02 |
| A08 Software + Data Integrity Failures | T-T-02 (rule-pack tampering); T-T-03 (Finding tampering); T-T-04 (baseline-pin tampering) | T-T-02, T-T-03, T-T-04 |
| A09 Security Logging + Monitoring Failures | Audit-chain seal; quarterly root; Grafana OnCall on BLOCKER findings | `runbooks/lane-failure-triage.md` |
| A10 Server-Side Request Forgery | Outbound-allow-list per runner; baseline-diff client allow-listed hosts only | T-T-01 mitigation |

## Residual Risk Posture

| Risk class | Residual after mitigations | Acceptance | Review trigger |
|---|---|---|---|
| Supply-chain (T-T-01) | M (transitive; bounded by SLSA L3) | Accepted by ops-security; quarterly review | Any CVE in transitive dep |
| PII leakage in evidence (T-I-03) | M (regex-based redaction imperfect) | Accepted by ops-privacy; monthly false-negative review | Any new data-class added to taxonomy |
| Lane bypass (T-E-01) | M (admin override unavoidable) | Accepted by council-architecture; per-incident review | Any bypass event |
| Per-pack retention violation (P-NC-01) | M (operator burden) | Accepted by ops-compliance; per-pack deploy verification | New pack added |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=threat-model-coverage --microservice governance` exit 0 (asserts every kernel struct has at least one threat reference; lane will be implemented in IP-006).
- Quarterly STRIDE + LINDDUN review by ops-security; record at `evidence/audits/threat-model/<quarter>.json`.
- Annual external-auditor review (SOC 2 Type 2 + ISO 27001).

## References

- ADR-0028 (Bominal; audit-chain + data-class taxonomy).
- ADR-0117 (data-residency).
- ADR-0123 (HG-GOV registers here).
- ADR-0131 (per-microservice flat layout).
- ADR-0133 (industry-best-practice conformance).
- SLSA v1.0 — `slsa.dev`.
- NIST SSDF SP 800-218 — `csrc.nist.gov`.
- OWASP Top 10 (2021) — `owasp.org/Top10`.
- OWASP ASVS v4 — `owasp.org/www-project-application-security-verification-standard/`.
- CIS Kubernetes Benchmark — `cisecurity.org`.
- LINDDUN — `linddun.org`.
- `microservices/observability/threat-model.md` (shape reference).
