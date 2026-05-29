---
doc_class: MultiRegionPosture
title: Multi-Region + DR Posture
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry
deciders: ops-sre-reliability, axis-foundry, council-architecture, ops-security
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/governance/policy/data-residency.md
  - microservices/governance/failure-modes.md
  - microservices/governance/capacity-model.md
review_cadence: per-pack onboarding + quarterly
doc_status: published
---

# Multi-Region + DR Posture: governance µservice

## Purpose

Document governance µservice's posture across packs, regions, and DR pairs. Defines per-pack region pinning (residency), failover topology, RTO/RPO targets, and the cross-region transfer rules that hold across the lifetime of the µservice.

## M01 baseline

- Single pack: **pack-kr** (OCI ap-seoul-1).
- Single region; no DR pair at M01 launch; RTO/RPO bounded by intra-region recovery only.
- Per-pack residency lock per ADR-0117 + `policy/data-residency.md`.
- All Findings + evidence + audit-chain seals live in pack-kr.
- Cross-border refused except auditor JIT reads.

## Post-M01 Expansion (per-pack onboarding sequence)

Per ADR-0117 §"pack onboarding order"; matches Bominal pack rollout sequence inherited per `feedback_bominal_inheritance_precedence.md`:

| Wave | Packs | OCI region | DR pair? | Target |
|---|---|---|---|---|
| 1 (M01) | pack-kr | ap-seoul-1 | no (single-region acceptable for KR) | M01 |
| 2 | pack-eu | eu-frankfurt-1 | yes (eu-amsterdam-1 warm) | M02 |
| 3 | pack-us | us-ashburn-1 | yes (us-phoenix-1 warm) | M02 |
| 4 | pack-jp | ap-tokyo-1 | no | M03 |
| 5 | pack-sg | ap-singapore-1 | no | M03 |
| 6 | pack-au | ap-sydney-1 | yes (ap-melbourne-1 warm) | M03 |
| 7 | pack-in | ap-mumbai-1 | yes (ap-hyderabad-1 warm) | M04 |
| 8 | pack-br | sa-saopaulo-1 | yes (sa-vinhedo-1 warm) | M04 |
| 9 | pack-ae | me-dubai-1 | yes (me-abudhabi-1 warm) | M04 |
| 10 | pack-ksa | me-jeddah-1 | yes (me-riyadh-1 warm) | M04 |
| 11 | pack-us-healthcare | us-ashburn-1 (HIPAA-eligible) | yes (us-phoenix-1 HIPAA-eligible) | M05 |

## Topology per pack

### Single-region packs (pack-kr, pack-jp, pack-sg)

```text
┌─ Pack region ───────────────────────────────────────────────┐
│                                                             │
│   Governance cluster                                        │
│   ├─ ARC runner pool                                        │
│   ├─ lane-runtime + policy-engine + evidence-emitter +      │
│   │  aggregation-indexer (each 2+ replicas)                 │
│   ├─ Postgres HA (1 primary + 2 sync replicas)             │
│   └─ S3 evidence bucket (multi-AZ within region)            │
│                                                             │
│   RTO ≤ 2h (intra-region failure modes per failure-modes.md)│
│   RPO = 0 (sync replication; outbox patterns)              │
│                                                             │
│   Cross-region failover: NOT AVAILABLE                      │
│   Acceptance: pack carries single-region acceptance per     │
│   ADR-0117 (KR + JP + SG regulators tolerate single-region) │
└─────────────────────────────────────────────────────────────┘
```

### DR-pair packs (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa, pack-us-healthcare)

```text
┌─ Primary region ─────────────┐        ┌─ DR region (warm) ──────────┐
│                              │        │                             │
│  Governance cluster (active) │        │  Governance cluster (warm)  │
│  ├─ ARC pool (full)          │        │  ├─ ARC pool (50% scale)    │
│  ├─ All workloads            │        │  ├─ Workloads idle          │
│  ├─ Postgres primary         │ ─────► │  ├─ Postgres async replica  │
│  └─ S3 primary bucket        │ ─────► │  └─ S3 cross-region rep'd   │
│                              │        │                             │
└──────────────────────────────┘        └─────────────────────────────┘
        │                                       │
        └─────────── DNS failover ──────────────┘
                 (Cloudflare DNS; 60s TTL)

RTO target: ≤ 15 min (DNS + Postgres replica promotion)
RPO target: ≤ 60 s (async logical replication lag)
Drill cadence: quarterly per pack
```

## RTO/RPO targets per pack class

| Pack class | RTO | RPO | Mechanism |
|---|---|---|---|
| Single-region | 2h | 0 (intra-region sync) | Intra-region HA |
| DR pair (standard) | 15 min | 60 s | DNS + Postgres promote + S3 replica |
| DR pair (HIPAA) | 15 min | 60 s | Same as standard + BAA-bound DR site |

## Cross-Region Replication Rules

| Asset | Replication mode | Notes |
|---|---|---|
| Postgres (Findings, lane-runs) | Async logical replication to DR | < 60 s lag p99 |
| Postgres (per-tenant identity tables) | Async; tenant residency overrides | refused if cross-residency-pack |
| S3 evidence bucket | Cross-region replication (CRR) within same pack only | refused if cross-pack |
| Audit-chain seals | Replicated globally (cryptographic; non-PII) | Merkle root publishable globally |
| Rule packs (git) | Globally replicated (code; pseudonymous) | no residency restriction |
| Industry-baseline pins | Globally replicated | no residency restriction |
| Aggregation indices | Globally replicated | no residency restriction (code/metadata only) |

## Cross-Pack Transfer Rules

Per `policy/data-residency.md` Cross-pack transfer matrix. Default: **REFUSED**. Permitted only with explicit per-tenant consent + appropriate mechanism (adequacy / SCCs + TIA / BAA chain). Cross-pack reads by external auditors permitted only under `auditor-scope.cedar` JIT scope.

| Transfer direction | Default | Override path |
|---|---|---|
| pack-A → pack-B (production data) | REFUSED | Tenant migration ADR + per-tenant consent + ops-compliance approval |
| pack-A → pack-B (auditor read) | PERMITTED if auditor scope claims both packs | Per-audit window only |
| pack-us-healthcare → other packs | REFUSED | BAA chain + ops-compliance + HHS notification if PHI |
| pack-eu → pack-non-adequate | REFUSED absent SCCs + TIA | SCCs 2021/914 + TIA in compliance.md |
| Any pack → audit-chain seal global root | PERMITTED | Cryptographic; non-PII; explicit per ADR-0028 |

## Failover Procedures

### Standard DR failover (15-min RTO)

Per `runbooks/migration-execution.md` §"Cross-region failover" (related runbook):

1. **Detect** primary unavailability (Postgres + S3 health-check fail; Grafana alert).
2. **Decision**: ops-sre-reliability on-call + axis-foundry on-call review failure type.
3. **Promote** DR Postgres replica to primary (Patroni promote; takes ≤30s).
4. **Switch** DNS to DR ingress (Cloudflare update; 60s TTL).
5. **Resume** ARC runner pool in DR (scale to full).
6. **Backfill** any in-flight lane runs from primary (via outbox replay).
7. **Verify** end-to-end: synthetic-PR + lane run + Finding emission.
8. **Notify** stakeholders + open postmortem incident record.

### Pack-onboarding procedure

Per `runbooks/migration-execution.md` §"Per-pack onboarding":

1. **ADR** per new pack approving residency + DR posture.
2. **Provision** OCI region (compute + network + IAM compartment).
3. **Deploy** governance cluster via `iac/helm/*` + `iac/kustomize/overlays/pack-<pack>/`.
4. **Cedar policy** review for pack-specific overrides.
5. **Compliance check**: `data-residency.md` + `compliance.md` updated; ROPA filed.
6. **Drill**: pack-onboarding drill executed before accepting production traffic.
7. **Tenant migration**: per-tenant consent + ADR + per-data-class migration plan.

## Pack-onboarding gate

Before a pack accepts production traffic:

| Gate | Owner | Status (per-pack) |
|---|---|---|
| ADR-####-pack-<pack>-onboarding accepted | council-architecture | required-before-onboarding; tracked per pack in `microservices/governance/IP-PACK-<pack>-onboarding.md` (templated; one IP per pack at fill time) |
| Residency overlay deployed | ops-sre-reliability | required-before-onboarding; verified by `oya-check-data-residency` lane green on overlay PR |
| Cedar fragments reviewed | ops-security | required-before-onboarding; logged in `microservices/governance/decisions/ADR-PACK-<pack>-cedar-review.md` |
| Compliance ROPA filed | ops-compliance | required-before-onboarding; ROPA entry in `microservices/governance/compliance.md` §ROPA before traffic shifts |
| DPIA addendum signed | council-privacy | required-before-onboarding; signed addendum at `microservices/governance/dpia-pack-<pack>.md` |
| DR drill executed | ops-sre-reliability | required-before-onboarding; drill record at `microservices/governance/runbooks/pack-<pack>-dr-drill-result-<date>.md` |
| Cost-budget projection added | ops-finops | required-before-onboarding; projection row in `microservices/governance/cost-budget.md` Forecast-vs-Actual table |
| `oya-check-data-residency` lane green per pack | axis-foundry | required-before-onboarding; gating PR cannot merge until lane is green |

## Drill cadence

| Drill type | Cadence | Owner |
|---|---|---|
| Per-pack DR failover | quarterly | ops-sre-reliability |
| Cross-pack auditor JIT scope review | quarterly | ops-security |
| Pack-onboarding rehearsal | per new pack | ops-sre-reliability + ops-compliance |
| End-to-end PR through DR-promoted primary | quarterly | axis-foundry |

## Roadmap

| Quarter | Pack additions | Notes |
|---|---|---|
| 2026-Q2 (M01) | pack-kr only | baseline |
| 2026-Q3 (M02) | + pack-eu + pack-us | first DR-pair packs |
| 2026-Q4 (M03) | + pack-jp + pack-sg + pack-au | APAC expansion |
| 2027-Q1 (M04) | + pack-in + pack-br + pack-ae + pack-ksa | global expansion |
| 2027-Q2 (M05) | + pack-us-healthcare | HIPAA-eligible pack |

## Verification

- `cargo run -p oya-dev-cli -- gate validate multi-region --microservice governance` — exit 0.
- Quarterly DR drill per pack; record at `evidence/audits/dr-drill/<quarter>/<pack>.json`.
- Cross-pack consistency check: pack-A's Postgres data does NOT appear in pack-B Postgres replicas.

## References

- ADR-0117 (data-residency + pack scheme).
- `policy/data-residency.md` (per-pack residency rules).
- `failure-modes.md` (RTO/RPO failure modes).
- `runbooks/migration-execution.md` (DR failover steps).
- `microservices/observability/multi-region.md` (shape reference).
- OCI Object Storage Cross-Region Replication — `docs.oracle.com/en-us/iaas/Content/Object/Tasks/usingreplication.htm`.
