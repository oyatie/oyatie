---
doc_class: MultiRegionPlan
title: Multi-Region Topology + BCDR
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-guardrails + cloud-iac + cloud-k8s
deciders: ops-sre-reliability, axis-foundry-guardrails, council-architecture, council-privacy
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/intelligence-guardrails/policy/data-residency.md
  - microservices/intelligence-guardrails/capacity-model.md
  - microservices/intelligence-guardrails/cost-budget.md
  - microservices/intelligence-guardrails/failure-modes.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Multi-Region Topology + BCDR (foundry-guardrails µservice)

## Purpose

Define the multi-region topology across the 11 oyatie packs: pack-pinning, in-pack DR pair (where applicable), cross-pack-replication-forbidden policy, BCDR posture, RPO/RTO per pack, failover procedures. Reference for ops-sre-reliability on-call during region outages + auditors verifying BC.

## Topology Per Pack

Inherited shape from observability (same pack matrix; same DR-pair pattern).

| Pack | Primary region | DR pair (warm-standby) | Single-region? | Activation status |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | — | YES | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | DR pair | Conditional (first EU SCC) |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | DR pair | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | DR pair; isolated from pack-us | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | — | YES | Conditional |
| pack-sg | OCI ap-singapore-1 | — | YES | Conditional |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | DR pair | Conditional |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | DR pair | Conditional |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | DR pair | Conditional |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | DR pair | Conditional |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | DR pair | Conditional |

## In-Pack DR-Pair Architecture (DR-pair packs)

```text
┌─ Pack <X> ───────────────────────────────────────────────────────────────┐
│                                                                          │
│  Primary region                          DR-pair region                  │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Classifier-serving pool  │            │ Classifier-serving pool  │    │
│  │ (active; per-model rep)  │  artifact  │ (warm; 0.6× capacity)    │    │
│  │  - Cosign-verified ONNX  │  replic    │  - Cosign-verified ONNX  │    │
│  │  - per-pack S3 model reg │ ◀────────▶ │  - replicated model reg  │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Postgres rule store      │   sync     │ Postgres warm-standby    │    │
│  │  - HA primary + 2 RR     │ ◀────────▶ │  - read replica          │    │
│  │  - mutation log          │   repl     │  - failover-promotable   │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ Cedar engine (in-proc +  │            │ Cedar engine (in-proc +  │    │
│  │  standalone)             │            │  standalone, warm)       │    │
│  │ Bundle from Postgres     │            │ Bundle synced            │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│  ┌──────────────────────────┐            ┌──────────────────────────┐    │
│  │ rest + worker + app      │            │ rest + worker + app      │    │
│  │ (active)                 │            │ (warm-standby)           │    │
│  └──────────────────────────┘            └──────────────────────────┘    │
│                                                                          │
│  Global Traffic Manager (per-pack DNS):                                  │
│  - Health-check on primary's rest endpoint                               │
│  - On failure: DNS failover → DR pair (≤ 60s TTL)                        │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Replication

| Component | Mode | RPO | Cross-region scope |
|---|---|---|---|
| Classifier-model artifacts | Async via S3 CRR (Cosign-signed; integrity verified pre-deploy) | ≤ 5 min | intra-pack only |
| Postgres rule store | Sync (within-pack); RR with ≤ 1s lag | ≤ 1 s | intra-pack only |
| Cedar fragment bundle | Re-derived from Postgres on failover | ≤ 1 s | intra-pack only |
| Audit-chain seal records | Replicated by audit-chain µservice per its own multi-region plan | per audit-chain | intra-pack only |
| GuardrailDecision events | Emitted at write-time; consumer (foundry-evidence) replicates per its policy | per foundry-evidence | intra-pack only |
| Recording rules (observability emit) | Per-pack; not replicated cross-pack | n/a | intra-pack only |

### Cross-pack policy

Forbidden by default. Tenant data, decisions, rules, models all pack-pinned. Exception: SCC-executed cross-border transfer per GDPR Arts. 44-46; documented in `legal/transfer-register.md` (Slice D).

## RPO / RTO Targets

| Pack type | RPO | RTO | Notes |
|---|---|---|---|
| Single-region (pack-kr, pack-jp, pack-sg) | ≤ 60s (rule mutations) | ≤ 30 min (region-recovery dependent) | provider-bounded |
| DR-pair (pack-eu, pack-us, pack-us-hc, pack-au, pack-in, pack-br, pack-ae, pack-ksa) | ≤ 1s (Postgres sync) | ≤ 15 min (DR failover) | per-pack DR pair |

## DR Failover Procedure (DR-pair packs)

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-1 incident; open `#inc-<id>`; assign IC | ≤ 5 min |
| 2 | Verify primary unavailable for ≥ 5 min via two-channel check (Grafana SLI + manual probe from neighbouring region) | ≤ 5 min |
| 3 | Promote Postgres warm-standby to primary (pg_failover script in `iac/postgres/failover.sh`) | ≤ 3 min |
| 4 | Update Global Traffic Manager DNS TTL → 30s; switch records to DR pair | ≤ 2 min |
| 5 | Scale DR-pair classifier-serving + rest from warm (0.6×) → full (1.0×); HPA-driven | ≤ 5 min |
| 6 | Verify gate decisions flowing (test invocation from foundry-runtime) | ≤ 1 min |
| 7 | Notify tenants per `incident-response.md` template | ≤ 30 min |
| 8 | Post-failover: investigate primary; plan failback when stable; postmortem within 5 business days | – |

Total RTO budget: **≤ 15 min** failover; full tenant restoration ≤ 30 min.

## BCDR Drill Cadence

- **Quarterly** automated failover drill per DR-pair pack (non-prod tenants only; or shadow-mode drill).
- **Annual** full pack failover drill (production-tier traffic; tenants notified 7d in advance; ≤ 30 min window).
- **Post-incident** drills triggered after any actual failover; verify learnings.

## Single-Region Pack Posture

Packs without DR pair (pack-kr, pack-jp, pack-sg) lack in-pack hot failover. BC posture:
- Within-region multi-AZ: classifier-serving spread across ≥ 3 AZs; Postgres HA with sync replication across AZs.
- Region-outage tolerance: bounded by OCI provider-recovery time; for pack-kr at M01 ≤ 4h is the realistic ceiling (provider-dependent).
- Tenants in single-region packs are informed at onboarding of the RPO/RTO posture; tenants requiring cross-region DR must request a DR-pair pack (typically pack-eu or pack-us for SCC-eligible cases).

Future ADR: when business case justifies, single-region packs can be upgraded to DR-pair by activating a paired region; pack-kr → pack-kr DR-pair is a candidate.

## Audit + Compliance

Per `compliance.md`:
- SOC 2 CC9.1 (risk mitigation): DR + auto-rollback.
- ISO 27001 A.5.30 (BC): documented + drilled.
- HIPAA §164.308(a)(7) (contingency plan): DR runbook + drill cadence.
- GDPR Art. 32(1)(c) (availability + resilience).
- EU AI Act Art. 14 (human-oversight): manual failover supervised.

## Verification

- `cargo run -p oya-dev-cli -- gate validate dr-readiness --microservice foundry-guardrails --pack <pack>` — exit 0.
- Quarterly drill: scripted; success criteria documented in `runbooks/dr-failover-drill.md`.
- Annual auditor review of BC posture.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- `microservices/intelligence-guardrails/policy/data-residency.md`.
- `microservices/intelligence-guardrails/capacity-model.md`.
- `microservices/intelligence-guardrails/cost-budget.md`.
- `microservices/intelligence-guardrails/failure-modes.md`.
- `microservices/observability/multi-region.md` (inherited shape).
- OCI region documentation.
